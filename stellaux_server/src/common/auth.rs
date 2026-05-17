//! JWT auth middleware + extractor.
//!
//! Three middleware flavors:
//!
//! - `require_auth`           — accepts an HS256 token issued by this service.
//! - `require_admin`          — same + enforces `role == Admin`.
//! - `require_supabase_auth`  — accepts an RS256 token issued by Supabase.
//!   Maps Supabase claims into our internal `Claims` with `Role::Customer`
//!   (admin elevation is a separate DB check inside handlers).
//!
//! All three insert `Claims` into request extensions; handlers extract via
//! the `AuthUser` extractor regardless of which middleware ran.

use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, header, request::Parts},
    middleware::Next,
    response::Response,
};

use crate::common::{
    app_state::AppState,
    error::{AppError, AppResult},
    jwt::{self, Claims, Role},
};

// ─── Bearer extraction ──────────────────────────────────────────────────────

fn extract_bearer(req: &Request) -> AppResult<&str> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .filter(|s| !s.is_empty())
        .ok_or(AppError::Unauthorized)
}

// ─── HS256 (internal tokens) ────────────────────────────────────────────────

/// Verify an HS256 token issued by this service; stash `Claims` in extensions.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let claims = verify_internal(&state, &req)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// Like `require_auth` plus a `role == Admin` gate.
pub async fn require_admin(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let claims = verify_internal(&state, &req)?;
    if claims.role != Role::Admin {
        return Err(AppError::Forbidden);
    }
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn verify_internal(state: &AppState, req: &Request) -> AppResult<Claims> {
    let token = extract_bearer(req)?;
    jwt::verify(
        state.config.auth.jwt_secret.as_bytes(),
        token,
        &state.config.auth.issuer,
        &state.config.auth.audience,
    )
    .map_err(|_| AppError::Unauthorized)
}

// ─── RS256 (Supabase tokens) ────────────────────────────────────────────────

/// Verify a Supabase-issued RS256 token via the JWKS cache. Translates the
/// Supabase claims into our internal `Claims` (always `Role::Customer` — admin
/// elevation is a separate DB check).
pub async fn require_supabase_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let cache = state.jwks.as_ref().ok_or_else(|| {
        tracing::error!("require_supabase_auth hit but SUPABASE_JWKS_URL is not configured");
        AppError::Internal(anyhow::anyhow!("Supabase JWKS not configured"))
    })?;

    let token = extract_bearer(&req)?;
    let supa = jwt::verify_supabase(
        cache,
        token,
        &state.config.auth.supabase_audience,
        state.config.auth.supabase_issuer.as_deref(),
    )
    .await?;

    let user_id = supa.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let claims = Claims {
        sub: user_id,
        iat: supa.iat,
        exp: supa.exp,
        iss: supa.iss.unwrap_or_else(|| state.config.auth.issuer.clone()),
        aud: supa.aud,
        role: Role::Customer,
    };

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

// ─── Handler extractor ──────────────────────────────────────────────────────

/// Pulls the `Claims` previously inserted by any of the auth middlewares.
/// Returns 500 if the route wasn't mounted behind an auth layer — that's a
/// programmer error, not a user error.
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthUser)
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "auth middleware not mounted for this route",
            ))
    }
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        matches!(self.0.role, Role::Admin)
    }
}
