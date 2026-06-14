use axum::{Json, Router, body::Bytes, extract::State, http::HeaderMap, routing::post};
use serde_json::{Value, json};

use crate::common::{app_state::AppState, error::AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/stripe", post(stripe))
        .route("/shippo", post(shippo))
}

async fn stripe(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<Value>> {
    let _sig = headers.get("stripe-signature");
    let _ = body;
    Ok(Json(json!({ "received": true })))
}

async fn shippo(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<Value>> {
    let _sig = headers.get("x-shippo-signature");
    let _ = body;
    Ok(Json(json!({ "received": true })))
}
