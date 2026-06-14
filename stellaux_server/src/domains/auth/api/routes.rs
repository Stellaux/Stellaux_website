use axum::{Json, Router, extract::State, routing::post};
use serde_json::{Value, json};

use crate::{
    common::{app_state::AppState, error::AppResult},
    domains::auth::dto::{
        ForgotPasswordRequest, LoginRequest, ResetPasswordRequest, SignupRequest,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
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
