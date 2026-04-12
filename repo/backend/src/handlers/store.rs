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

pub async fn list_products(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<Vec<Product>>> {
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
            Some(crate::services::audit_service::HASH_ENTITY_CREATED.to_string()),
            Some(AuditService::compute_hash(&serde_json::to_string(&row)?)),
            None,
        )
        .await?;
    Ok(Json(row))
}

pub async fn list_promotions(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<Vec<Promotion>>> {
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
            Some(crate::services::audit_service::HASH_ENTITY_CREATED.to_string()),
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
    // Capture before-state for audit hash.
    let before = svc.get_promotion(&id).await?;
    let before_hash = AuditService::compute_hash(&serde_json::to_string(&before)?);
    svc.deactivate_promotion(&id).await?;
    let after = svc.get_promotion(&id).await?;
    let after_hash = AuditService::compute_hash(&serde_json::to_string(&after)?);
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "promotion",
            Some(&id),
            Some(before_hash),
            Some(after_hash),
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
            Some(crate::services::audit_service::HASH_ENTITY_CREATED.to_string()),
            Some(AuditService::compute_hash(&serde_json::to_string(&order)?)),
            None,
        )
        .await?;
    Ok(Json(CheckoutResponse { order, result }))
}

pub async fn preview_checkout(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Json(req): Json<CheckoutRequest>,
) -> AppResult<Json<CheckoutResult>> {
    let svc = StoreService::new(state.db.clone());
    // Resolve server-side prices — same trust boundary as real checkout.
    let server_cart = svc.resolve_cart_from_db(&req.items).await?;
    let promos = svc.list_promotions().await?;
    Ok(Json(apply_best_promotion(&server_cart, &promos)))
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
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> AppResult<Json<OrderWithItems>> {
    use crate::error::AppError;
    use shared::UserRole;
    let svc = StoreService::new(state.db.clone());
    let (order, items) = svc.get_order_with_items(&id).await?;
    // Object-level authorization: administrators and store managers may read any
    // order; all other users may only read orders they placed themselves.
    let role = UserRole::from_str(&user.role);
    let privileged = matches!(
        role,
        Some(UserRole::Administrator) | Some(UserRole::StoreManager)
    );
    if !privileged && order.user_id != user.id {
        return Err(AppError::Forbidden);
    }
    Ok(Json(OrderWithItems { order, items }))
}

#[derive(Serialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}
