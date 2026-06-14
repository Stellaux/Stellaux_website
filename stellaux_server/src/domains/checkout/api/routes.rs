use axum::{Json, Router, extract::State, routing::post};
use serde_json::{Value, json};

use crate::{
    common::{app_state::AppState, error::AppResult},
    domains::checkout::dto::{CreateSessionRequest, ShippingRatesRequest},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/shipping-rates", post(get_shipping_rates))
        .route("/session", post(create_session))
}

async fn get_shipping_rates(
    State(_state): State<AppState>,
    Json(_body): Json<ShippingRatesRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "rates": [],
        "_todo": "compute total weight from cart_items, call Shippo POST /shipments"
    })))
}

async fn create_session(
    State(_state): State<AppState>,
    Json(_body): Json<CreateSessionRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "checkout_url": null,
        "_todo": "reserve inventory + create Stripe Checkout Session with metadata"
    })))
}
