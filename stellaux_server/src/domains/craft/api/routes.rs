use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde_json::{Value, json};

use crate::{
    common::{app_state::AppState, error::AppResult},
    domains::craft::dto::{AccessoriesQuery, BasesQuery},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bases", get(list_bases))
        .route("/accessories", get(list_accessories))
        .route("/compatibility/{base_handle}", get(get_compatibility))
}

async fn list_bases(
    State(_state): State<AppState>,
    Query(_q): Query<BasesQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "items": [],
        "_todo": "products where craft_role='base' [and craft_base_type=$type]"
    })))
}

async fn list_accessories(
    State(_state): State<AppState>,
    Query(_q): Query<AccessoriesQuery>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "items": [],
        "_todo": "products where craft_role='accessory', joined with craft_compatibility"
    })))
}

async fn get_compatibility(
    State(_state): State<AppState>,
    Path(base_handle): Path<String>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "base_handle": base_handle,
        "accessory_handles": [],
        "_todo": "join craft_compatibility + products to return handle list"
    })))
}
