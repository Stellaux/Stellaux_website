//! Public auth endpoints — issuing our own HS256 tokens.
//!
//! Mounted at `/api/v1/auth/*` in the public route group (no auth on the
//! endpoints themselves; the *result* is a token used elsewhere).
//!
//! Customer authentication primarily flows through Supabase Auth on the
//! frontend; the Rust API verifies those Supabase RS256 tokens via
//! `require_supabase_auth`. These endpoints exist for admin sign-in and any
//! non-Supabase service accounts that need our HS256 tokens.

use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::common::{app_state::AppState, error::AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

async fn login(
    State(_state): State<AppState>,
    Json(_body): Json<LoginRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "token": null,
        "_todo": "argon2 verify against profiles.password_hash, then jwt::issue with role"
    })))
}

async fn signup(
    State(_state): State<AppState>,
    Json(_body): Json<SignupRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "user_id": null,
        "_todo": "argon2 hash + insert auth.users / profiles + jwt::issue"
    })))
}

async fn forgot_password(
    State(_state): State<AppState>,
    Json(_body): Json<ForgotPasswordRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "sent": true,
        "_todo": "create signed reset token, email via Resend"
    })))
}

async fn reset_password(
    State(_state): State<AppState>,
    Json(_body): Json<ResetPasswordRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "_todo": "verify reset token + argon2 hash + update profiles"
    })))
}
