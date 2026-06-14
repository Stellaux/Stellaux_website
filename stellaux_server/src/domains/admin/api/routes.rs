use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    common::{app_state::AppState, auth::AuthUser, dto::Pagination, error::AppResult},
    domains::admin::dto::{CompatibilityPair, InventoryAdjust, OrdersFilter, UpsertProduct},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/orders", get(list_orders))
        .route("/orders/{order_id}", get(get_order))
        .route("/orders/{order_id}/refund", post(refund_order))
        .route("/orders/{order_id}/cancel", post(cancel_order))
        .route("/orders/{order_id}/labels", post(reprint_label))
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/{product_id}",
            get(get_product)
                .patch(update_product)
                .delete(delete_product),
        )
        .route("/inventory", get(list_inventory).post(adjust_inventory))
        .route(
            "/craft/compatibility",
            get(list_compatibility).post(add_compatibility),
        )
        .route(
            "/craft/compatibility/{base_id}/{accessory_id}",
            delete(remove_compatibility),
        )
        .route("/channel-listings", get(list_channel_listings))
}

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
    Ok(Json(
        json!({ "order_id": order_id, "_todo": "void label + refund" }),
    ))
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

async fn list_channel_listings(
    _admin: AuthUser,
    State(_state): State<AppState>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [] })))
}
