use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde_json::{Value, json};

use crate::{
    common::{app_state::AppState, error::AppResult},
    domains::catalog::dto::ProductFilter,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(list_products))
        .route("/products/{handle}", get(get_product))
        .route("/collections", get(list_collections))
        .route("/categories", get(list_categories))
}

async fn list_products(
    State(_state): State<AppState>,
    Query(_filter): Query<ProductFilter>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "items": [],
        "total": 0,
        "_todo": "select from products where status='active' with filters + pagination"
    })))
}

async fn get_product(
    State(_state): State<AppState>,
    Path(handle): Path<String>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "handle": handle,
        "_todo": "join products + variants + images by handle"
    })))
}

async fn list_collections(State(_state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": ["Vol. I", "Vol. II", "Atelier"] })))
}

async fn list_categories(State(_state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(
        json!({ "items": ["rings", "necklaces", "earrings", "bracelets"] }),
    ))
}
