//! Modular craft builder: bases, accessories, compatibility lookup.
//!
//! Mounted at `/api/v1/craft/*` in the public route group (no auth).

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::common::{app_state::AppState, error::AppResult};


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bases", get(list_bases))
        .route("/accessories", get(list_accessories))
        .route("/compatibility/{base_handle}", get(get_compatibility))
}

#[derive(Debug, Deserialize)]
pub struct BasesQuery {
    /// "pendant" | "chain" | "trunk"
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccessoriesQuery {
    /// Filter accessories compatible with this base handle.
    pub base_handle: Option<String>,
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
