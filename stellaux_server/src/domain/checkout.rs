//! Checkout flow: Shippo rate quotes + Stripe Checkout Session creation.
//!
//! Mounted at `/api/v1/checkout/*` in the protected route group.

use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::common::{app_state::AppState, error::AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/shipping-rates", post(get_shipping_rates))
        .route("/session", post(create_session))
}

#[derive(Debug, Deserialize)]
pub struct Address {
    pub recipient: String,
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShippingRatesRequest {
    pub cart_id: Uuid,
    pub address: Address,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub cart_id: Uuid,
    pub shippo_rate_id: String,
    pub address: Address,
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
