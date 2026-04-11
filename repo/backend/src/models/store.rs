//! Storefront row mappings: products, promotions, orders, line items.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock_quantity: i64,
    pub is_active: i64,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Promotion {
    pub id: String,
    pub name: String,
    pub description: String,
    pub discount_value: f64,
    pub discount_type: String,
    pub effective_from: String,
    pub effective_until: String,
    pub mutual_exclusion_group: Option<String>,
    pub priority: i64,
    pub is_active: i64,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub status: String,
    pub subtotal: f64,
    pub discount_applied: f64,
    pub total: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: String,
    pub order_id: String,
    pub product_id: String,
    pub product_name: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub discount_amount: f64,
    pub promotion_applied: Option<String>,
    pub promotion_trace: Option<String>,
}
