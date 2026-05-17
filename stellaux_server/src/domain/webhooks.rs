//! Stripe + Shippo webhooks.
//!
//! Mounted at `/api/v1/webhooks/*` in the webhooks route group (no auth on
//! the route — handlers verify HMAC signatures inline). Body limit is the
//! larger `webhook_body_limit_bytes` to accommodate label PDFs.

use axum::{
    Json, Router, body::Bytes, extract::State, http::HeaderMap, routing::post,
};
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
    // TODO:
    //   1. Read raw body bytes (already have them).
    //   2. Verify HMAC with STRIPE_WEBHOOK_SECRET via stripe Node SDK
    //      (or the `stripe-webhook` crate) using rustls/subtle crypto.
    //   3. Insert into webhook_events with ON CONFLICT DO NOTHING.
    //   4. Match on event type → dispatch to handlers (order, refund, dispute…).
    //   5. Set processed_at on success; return 500 on failure for Stripe retries.
    let _ = body;
    Ok(Json(json!({ "received": true })))
}

async fn shippo(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<Value>> {
    let _sig = headers.get("x-shippo-signature");
    // TODO:
    //   1. HMAC-SHA256(raw_body, SHIPPO_WEBHOOK_SECRET); constant-time compare.
    //   2. Dispatch on event type (mainly `track_updated`) → update orders.status.
    //   3. On `delivered`, schedule the DeliveredFollowup email (+3 days).
    let _ = body;
    Ok(Json(json!({ "received": true })))
}
