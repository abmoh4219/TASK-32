//! Store HTTP handlers — products, promotions, checkout, orders.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::AuditAction;

use crate::error::AppResult;
use crate::middleware::require_role::{AuthenticatedUser, RequireStore};
use crate::models::store::{Order, OrderItem, Product, Promotion};
use crate::services::audit_service::AuditService;
use crate::services::store_service::{
    apply_best_promotion, CartItem, CheckoutResult, CreateProductInput, CreatePromotionInput,
    StoreService,
};
use crate::AppState;

pub async fn list_products(State(state): State<AppState>) -> AppResult<Json<Vec<Product>>> {
    let svc = StoreService::new(state.db.clone());
    Ok(Json(svc.list_products().await?))
}

pub async fn create_product(
    State(state): State<AppState>,
    RequireStore(user): RequireStore,
    Json(input): Json<CreateProductInput>,
) -> AppResult<Json<Product>> {
    let svc = StoreService::new(state.db.clone());
    let row = svc.create_product(input, &user.id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Create,
            "product",
            Some(&row.id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&row)?)),
            None,
        )
        .await?;
    Ok(Json(row))
}

pub async fn list_promotions(State(state): State<AppState>) -> AppResult<Json<Vec<Promotion>>> {
    let svc = StoreService::new(state.db.clone());
    Ok(Json(svc.list_promotions().await?))
}

pub async fn create_promotion(
    State(state): State<AppState>,
    RequireStore(user): RequireStore,
    Json(input): Json<CreatePromotionInput>,
) -> AppResult<Json<Promotion>> {
    let svc = StoreService::new(state.db.clone());
    let row = svc.create_promotion(input, &user.id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Create,
            "promotion",
            Some(&row.id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&row)?)),
            None,
        )
        .await?;
    Ok(Json(row))
}

pub async fn deactivate_promotion(
    State(state): State<AppState>,
    RequireStore(user): RequireStore,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = StoreService::new(state.db.clone());
    svc.deactivate_promotion(&id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "promotion",
            Some(&id),
            None,
            Some(AuditService::compute_hash("deactivated")),
            None,
        )
        .await?;
    Ok(Json(json!({"success": true})))
}

#[derive(Deserialize)]
pub struct CheckoutRequest {
    pub items: Vec<CartItem>,
}

#[derive(Serialize)]
pub struct CheckoutResponse {
    pub order: Order,
    pub result: CheckoutResult,
}

pub async fn checkout(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CheckoutRequest>,
) -> AppResult<Json<CheckoutResponse>> {
    let svc = StoreService::new(state.db.clone());
    let (order, result) = svc.create_order(&user.id, req.items).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Checkout,
            "order",
            Some(&order.id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&order)?)),
            None,
        )
        .await?;
    Ok(Json(CheckoutResponse { order, result }))
}

pub async fn preview_checkout(
    State(state): State<AppState>,
    Json(req): Json<CheckoutRequest>,
) -> AppResult<Json<CheckoutResult>> {
    let svc = StoreService::new(state.db.clone());
    let promos = svc.list_promotions().await?;
    Ok(Json(apply_best_promotion(&req.items, &promos)))
}

pub async fn list_orders(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> AppResult<Json<Vec<Order>>> {
    let svc = StoreService::new(state.db.clone());
    Ok(Json(svc.list_orders(Some(&user.id), 100).await?))
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<OrderWithItems>> {
    let svc = StoreService::new(state.db.clone());
    let (order, items) = svc.get_order_with_items(&id).await?;
    Ok(Json(OrderWithItems { order, items }))
}

#[derive(Serialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}
