//! Store + promotion engine.
//!
//! Implements the SPEC promotion algorithm:
//!
//! 1. **Filter by effective window** — drop any promotion whose
//!    `effective_from..effective_until` does not contain the current time, or
//!    whose `is_active` flag is false.
//! 2. **Mutual exclusion resolution** — within each `mutual_exclusion_group`,
//!    keep **only the highest-priority** promotion. Promotions outside any
//!    group survive automatically.
//! 3. **Best discount selection** — across the surviving set, pick the single
//!    promotion that produces the greatest total cart discount and apply it.
//!    The discount is distributed proportionally across line items so each
//!    `LineItemResult` has a traceable `discount_amount` and `promotion_applied`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::store::{Order, OrderItem, Product, Promotion};

#[derive(Clone)]
pub struct StoreService {
    pub db: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductInput {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock_quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePromotionInput {
    pub name: String,
    pub description: String,
    pub discount_value: f64,
    pub discount_type: String, // "percent" | "fixed"
    pub effective_from: String,
    pub effective_until: String,
    pub mutual_exclusion_group: Option<String>,
    pub priority: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CartItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i64,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemResult {
    pub item: CartItem,
    pub line_subtotal: f64,
    pub discount_amount: f64,
    pub line_total: f64,
    pub promotion_applied: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResult {
    pub line_items: Vec<LineItemResult>,
    pub subtotal: f64,
    pub total_discount: f64,
    pub total: f64,
    pub best_promotion: Option<Promotion>,
}

impl StoreService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn list_products(&self) -> AppResult<Vec<Product>> {
        let rows = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE is_active = 1 ORDER BY name",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    pub async fn create_product(
        &self,
        input: CreateProductInput,
        actor_id: &str,
    ) -> AppResult<Product> {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("product name required".into()));
        }
        if input.price < 0.0 {
            return Err(AppError::Validation("price must be ≥ 0".into()));
        }
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO products (id, name, description, price, stock_quantity, is_active, created_by, created_at)
             VALUES (?, ?, ?, ?, ?, 1, ?, ?)",
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.price)
        .bind(input.stock_quantity)
        .bind(actor_id)
        .bind(&now)
        .execute(&self.db)
        .await?;
        let row = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.db)
            .await?;
        Ok(row)
    }

    pub async fn list_promotions(&self) -> AppResult<Vec<Promotion>> {
        let rows = sqlx::query_as::<_, Promotion>(
            "SELECT * FROM promotions ORDER BY priority DESC, created_at DESC",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    pub async fn create_promotion(
        &self,
        input: CreatePromotionInput,
        actor_id: &str,
    ) -> AppResult<Promotion> {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("promotion name required".into()));
        }
        if !["percent", "fixed"].contains(&input.discount_type.as_str()) {
            return Err(AppError::Validation(
                "discount_type must be 'percent' or 'fixed'".into(),
            ));
        }
        if input.discount_value < 0.0 {
            return Err(AppError::Validation("discount_value must be ≥ 0".into()));
        }
        let from = DateTime::parse_from_rfc3339(&input.effective_from)
            .map_err(|e| AppError::Validation(format!("effective_from: {e}")))?;
        let until = DateTime::parse_from_rfc3339(&input.effective_until)
            .map_err(|e| AppError::Validation(format!("effective_until: {e}")))?;
        if from >= until {
            return Err(AppError::Validation(
                "effective_from must be strictly before effective_until".into(),
            ));
        }
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO promotions (id, name, description, discount_value, discount_type, effective_from, effective_until, mutual_exclusion_group, priority, is_active, created_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)",
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.discount_value)
        .bind(&input.discount_type)
        .bind(&input.effective_from)
        .bind(&input.effective_until)
        .bind(&input.mutual_exclusion_group)
        .bind(input.priority)
        .bind(actor_id)
        .bind(&now)
        .execute(&self.db)
        .await?;
        let row = sqlx::query_as::<_, Promotion>("SELECT * FROM promotions WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.db)
            .await?;
        Ok(row)
    }

    pub async fn deactivate_promotion(&self, id: &str) -> AppResult<()> {
        sqlx::query("UPDATE promotions SET is_active = 0 WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Run a cart through the promotion engine and persist the resulting order
    /// + per-line trace.
    pub async fn create_order(
        &self,
        user_id: &str,
        cart: Vec<CartItem>,
    ) -> AppResult<(Order, CheckoutResult)> {
        if cart.is_empty() {
            return Err(AppError::Validation("cart is empty".into()));
        }
        let promos = self.list_promotions().await?;
        let result = apply_best_promotion(&cart, &promos);

        let order_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.begin().await?;
        sqlx::query(
            "INSERT INTO orders (id, user_id, status, subtotal, discount_applied, total, created_at)
             VALUES (?, ?, 'completed', ?, ?, ?, ?)",
        )
        .bind(&order_id)
        .bind(user_id)
        .bind(result.subtotal)
        .bind(result.total_discount)
        .bind(result.total)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
        for line in &result.line_items {
            let item_id = Uuid::new_v4().to_string();
            let trace_json = serde_json::to_string(line).unwrap_or_else(|_| "{}".into());
            sqlx::query(
                "INSERT INTO order_items
                 (id, order_id, product_id, product_name, quantity, unit_price, discount_amount, promotion_applied, promotion_trace)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&item_id)
            .bind(&order_id)
            .bind(&line.item.product_id)
            .bind(&line.item.product_name)
            .bind(line.item.quantity)
            .bind(line.item.unit_price)
            .bind(line.discount_amount)
            .bind(&line.promotion_applied)
            .bind(&trace_json)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = ?")
            .bind(&order_id)
            .fetch_one(&self.db)
            .await?;
        Ok((order, result))
    }

    pub async fn list_orders(&self, user_id: Option<&str>, limit: i64) -> AppResult<Vec<Order>> {
        let rows = if let Some(u) = user_id {
            sqlx::query_as::<_, Order>(
                "SELECT * FROM orders WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(u)
            .bind(limit)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Order>("SELECT * FROM orders ORDER BY created_at DESC LIMIT ?")
                .bind(limit)
                .fetch_all(&self.db)
                .await?
        };
        Ok(rows)
    }

    pub async fn get_order_with_items(&self, id: &str) -> AppResult<(Order, Vec<OrderItem>)> {
        let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(AppError::NotFound)?;
        let items = sqlx::query_as::<_, OrderItem>("SELECT * FROM order_items WHERE order_id = ?")
            .bind(id)
            .fetch_all(&self.db)
            .await?;
        Ok((order, items))
    }
}

// ─── Pure promotion-engine functions — no DB access — easy to unit-test ───

/// Select and apply the best eligible promotion at checkout.
/// Step 1: Filter by effective time window.
/// Step 2: Within each mutual_exclusion_group, keep only highest-priority promotion.
/// Step 3: Apply the promotion producing the greatest total discount.
/// Returns per-line-item traceable discount details.
pub fn apply_best_promotion(cart: &[CartItem], promotions: &[Promotion]) -> CheckoutResult {
    let now = Utc::now();

    let eligible: Vec<&Promotion> = promotions
        .iter()
        .filter(|p| p.is_active == 1 && in_window(p, now))
        .collect();

    let resolved = resolve_exclusion_groups(&eligible);

    let best = resolved
        .iter()
        .copied()
        .max_by(|a, b| {
            let da = total_cart_discount(cart, a);
            let db = total_cart_discount(cart, b);
            da.partial_cmp(&db).unwrap_or(Ordering::Equal)
        });

    let subtotal: f64 = cart
        .iter()
        .map(|i| i.unit_price * i.quantity as f64)
        .sum();
    let best_total = best.map(|p| total_cart_discount(cart, p)).unwrap_or(0.0);

    let line_items: Vec<LineItemResult> = cart
        .iter()
        .map(|item| {
            let line_subtotal = item.unit_price * item.quantity as f64;
            let discount = if subtotal > 0.0 {
                (line_subtotal / subtotal) * best_total
            } else {
                0.0
            };
            LineItemResult {
                item: item.clone(),
                line_subtotal,
                discount_amount: discount,
                line_total: line_subtotal - discount,
                promotion_applied: best.map(|p| p.name.clone()),
            }
        })
        .collect();

    CheckoutResult {
        line_items,
        subtotal,
        total_discount: best_total,
        total: subtotal - best_total,
        best_promotion: best.cloned(),
    }
}

/// Within each mutual_exclusion_group keep only the highest-priority promotion.
/// Promotions with `mutual_exclusion_group = None` always survive.
pub fn resolve_exclusion_groups<'a>(eligible: &[&'a Promotion]) -> Vec<&'a Promotion> {
    let mut by_group: HashMap<String, &Promotion> = HashMap::new();
    let mut ungrouped: Vec<&Promotion> = Vec::new();
    for p in eligible {
        match &p.mutual_exclusion_group {
            Some(g) if !g.is_empty() => {
                let entry = by_group.entry(g.clone()).or_insert(*p);
                if p.priority > entry.priority {
                    *entry = *p;
                }
            }
            _ => ungrouped.push(*p),
        }
    }
    let mut out: Vec<&Promotion> = by_group.into_values().collect();
    out.extend(ungrouped);
    out
}

/// Total discount that the given promotion would apply to the entire cart.
pub fn total_cart_discount(cart: &[CartItem], p: &Promotion) -> f64 {
    let subtotal: f64 = cart
        .iter()
        .map(|i| i.unit_price * i.quantity as f64)
        .sum();
    match p.discount_type.as_str() {
        "percent" => subtotal * (p.discount_value / 100.0),
        "fixed" => p.discount_value.min(subtotal),
        _ => 0.0,
    }
}

/// Check whether `now` lies inside the promotion's effective window.
pub fn in_window(p: &Promotion, now: DateTime<Utc>) -> bool {
    let from = DateTime::parse_from_rfc3339(&p.effective_from);
    let until = DateTime::parse_from_rfc3339(&p.effective_until);
    matches!(
        (from, until),
        (Ok(f), Ok(u)) if f.with_timezone(&Utc) <= now && now <= u.with_timezone(&Utc)
    )
}
