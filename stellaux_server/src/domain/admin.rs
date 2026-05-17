//! Admin operations: orders, products, inventory, channels, craft compatibility.
//!
//! Mounted at `/api/v1/admin/*` in the admin route group (require_admin).

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::common::{
    app_state::AppState, auth::AuthUser, dto::Pagination, error::AppResult,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Orders
        .route("/orders", get(list_orders))
        .route("/orders/{order_id}", get(get_order))
        .route("/orders/{order_id}/refund", post(refund_order))
        .route("/orders/{order_id}/cancel", post(cancel_order))
        .route("/orders/{order_id}/labels", post(reprint_label))
        // Products
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/{product_id}",
            get(get_product)
                .patch(update_product)
                .delete(delete_product),
        )
        // Inventory
        .route("/inventory", get(list_inventory).post(adjust_inventory))
        // Craft compatibility
        .route(
            "/craft/compatibility",
            get(list_compatibility).post(add_compatibility),
        )
        .route(
            "/craft/compatibility/{base_id}/{accessory_id}",
            delete(remove_compatibility),
        )
        // Channel listings (Etsy / eBay sync state)
        .route("/channel-listings", get(list_channel_listings))
}

// ─── DTOs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OrdersFilter {
    pub status: Option<String>,
    pub source: Option<String>, // "website" | "etsy" | "ebay"
    #[serde(flatten)]
    pub page: Pagination,
}

#[derive(Debug, Deserialize)]
pub struct InventoryAdjust {
    pub variant_id: Uuid,
    pub delta: i32,
    pub reason: String, // "restock" | "shrinkage" | "return" | "manual" | "channel_sync"
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompatibilityPair {
    pub base_product_id: Uuid,
    pub accessory_product_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpsertProduct {
    pub handle: String,
    pub name: String,
    pub category: String,
    pub material: String,
    pub collection: Option<String>,
    pub craft_role: Option<String>, // "base" | "accessory" | null
    pub craft_base_type: Option<String>,
}

// ─── Orders ─────────────────────────────────────────────────────────────────

async fn list_orders(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Query(_f): Query<OrdersFilter>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [], "total": 0 })))
}

async fn get_order(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "order_id": order_id })))
}

async fn refund_order(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "order_id": order_id,
        "_todo": "stripe refund + restock + email RefundConfirmation"
    })))
}

async fn cancel_order(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "order_id": order_id, "_todo": "void label + refund" })))
}

async fn reprint_label(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "order_id": order_id,
        "_todo": "re-fetch shippo transaction or void + re-buy"
    })))
}

// ─── Products ───────────────────────────────────────────────────────────────

async fn list_products(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Query(_p): Query<Pagination>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [], "total": 0 })))
}

async fn create_product(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Json(_body): Json<UpsertProduct>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "_todo": "insert product row + return id" })))
}

async fn get_product(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "id": id })))
}

async fn update_product(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_body): Json<UpsertProduct>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "id": id })))
}

async fn delete_product(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "deleted": id })))
}

// ─── Inventory ──────────────────────────────────────────────────────────────

async fn list_inventory(
    _admin: AuthUser,
    State(_state): State<AppState>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [] })))
}

async fn adjust_inventory(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Json(_body): Json<InventoryAdjust>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "_todo": "update inventory_levels.on_hand + insert inventory_adjustments"
    })))
}

// ─── Craft compatibility ────────────────────────────────────────────────────

async fn list_compatibility(
    _admin: AuthUser,
    State(_state): State<AppState>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [] })))
}

async fn add_compatibility(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Json(_pair): Json<CompatibilityPair>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "_todo": "insert into craft_compatibility" })))
}

async fn remove_compatibility(
    _admin: AuthUser,
    State(_state): State<AppState>,
    Path((base_id, accessory_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "removed": [base_id, accessory_id] })))
}

// ─── Channels ───────────────────────────────────────────────────────────────

async fn list_channel_listings(
    _admin: AuthUser,
    State(_state): State<AppState>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [], "_todo": "select from channel_listings" })))
}
