//! Server-side cart for both guests (anon cookie) and authenticated users.
//!
//! Mounted at `/api/v1/cart/*` in the protected route group.

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::common::{app_state::AppState, error::AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_cart).delete(clear_cart))
        .route("/items", post(add_item))
        .route("/items/{item_id}", axum::routing::patch(update_item).delete(remove_item))
}

#[derive(Debug, Deserialize)]
pub struct AddItem {
    pub variant_id: Uuid,
    pub qty: i32,
    /// When adding a Craft assembly, all component lines share this id.
    pub assembly_id: Option<Uuid>,
    /// "base" | "accessory" — only meaningful when `assembly_id` is set.
    pub assembly_role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItem {
    pub qty: i32,
}

async fn get_cart(State(_state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "id": null,
        "items": [],
        "subtotal_cents": 0,
        "_todo": "load cart by user_id or anonymous_token cookie"
    })))
}

async fn add_item(
    State(_state): State<AppState>,
    Json(_body): Json<AddItem>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "_todo": "insert cart_items + reserve inventory_levels.reserved += qty"
    })))
}

async fn update_item(
    State(_state): State<AppState>,
    Path(item_id): Path<Uuid>,
    Json(_body): Json<UpdateItem>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "item_id": item_id,
        "_todo": "adjust qty + re-reserve delta"
    })))
}

async fn remove_item(
    State(_state): State<AppState>,
    Path(item_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "removed": item_id,
        "_todo": "delete cart_item + release reserved inventory"
    })))
}

async fn clear_cart(State(_state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "cleared": true,
        "_todo": "delete all cart_items + release all reservations"
    })))
}
