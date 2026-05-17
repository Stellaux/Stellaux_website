//! Shared state injected into every handler via `axum::extract::State<AppState>`.
//!
//! `Arc` semantics are implicit: `DatabaseConnection` and `reqwest::Client` are
//! already cheap-to-clone (internal Arc); `Config` and `JwksCache` are wrapped
//! in `Arc` ourselves so cloning never copies strings or HashMap contents.

use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::common::{config::Config, jwt::JwksCache};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
    pub http: reqwest::Client,
    /// Present iff `SUPABASE_JWKS_URL` is configured. `None` means Supabase
    /// JWT verification is disabled; routes behind `require_supabase_auth`
    /// will return 500 if hit.
    pub jwks: Option<Arc<JwksCache>>,
}
