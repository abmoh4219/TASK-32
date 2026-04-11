//! Store API client — products, promotions, checkout, orders.

use serde::{Deserialize, Serialize};

use crate::api::client::{get_json, post_json, ApiError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub status: String,
    pub subtotal: f64,
    pub discount_applied: f64,
    pub total: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub order: Order,
    pub result: CheckoutResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckoutRequest {
    pub items: Vec<CartItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePromotionInput {
    pub name: String,
    pub description: String,
    pub discount_value: f64,
    pub discount_type: String,
    pub effective_from: String,
    pub effective_until: String,
    pub mutual_exclusion_group: Option<String>,
    pub priority: i64,
}

pub async fn list_products() -> Result<Vec<Product>, ApiError> {
    get_json("/api/store/products").await
}

pub async fn list_promotions() -> Result<Vec<Promotion>, ApiError> {
    get_json("/api/store/promotions").await
}

pub async fn create_promotion(input: CreatePromotionInput) -> Result<Promotion, ApiError> {
    post_json("/api/store/promotions", &input).await
}

pub async fn preview_checkout(items: Vec<CartItem>) -> Result<CheckoutResult, ApiError> {
    post_json("/api/store/checkout/preview", &CheckoutRequest { items }).await
}

pub async fn checkout(items: Vec<CartItem>) -> Result<CheckoutResponse, ApiError> {
    post_json("/api/store/checkout", &CheckoutRequest { items }).await
}

pub async fn list_orders() -> Result<Vec<Order>, ApiError> {
    get_json("/api/store/orders").await
}
