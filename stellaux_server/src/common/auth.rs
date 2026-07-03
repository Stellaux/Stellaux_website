//! JWT auth middleware + extractor.
//!
//! Three middleware flavors:
//!
//! - `require_auth`           — accepts an HS256 token issued by this service.
//! - `require_admin`          — same + enforces `role == Admin`.
//! - `require_supabase_auth`  — accepts an RS256 token issued by Supabase and
//!   resolves the caller's authorization role from the `user_roles` table.
//! - `require_supabase_admin` — same + enforces `role == Admin`. This is the
//!   gate for the `/api/v1/admin/*` group under the unified Supabase identity.
//!
//! Supabase is the single identity provider for every human user (customers and
//! staff alike); *authorization* roles live in our own `user_roles` table and
//! are stamped onto `Claims` per request (see `common::roles`). A user with no
//! row is a plain `Customer`.
//!
//! All middlewares insert `Claims` into request extensions; handlers extract via
//! the `AuthUser` extractor regardless of which middleware ran.

use axum::{
    extract::{FromRequestParts, Request, State},
    http::{HeaderMap, StatusCode, header, request::Parts},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use uuid::Uuid;

use crate::common::{
    app_state::AppState,
    error::{AppError, AppResult},
    jwt::{self, Claims, Role},
    roles,
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

const HEADER_ORIGINAL_USER: &str = "x-original-user";

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

/// Verify a Supabase-issued RS256 token via the JWKS cache and resolve the
/// caller's authorization role from `user_roles`. Returns fully-populated
/// internal `Claims`. Shared by `require_supabase_auth` and
/// `require_supabase_admin`.
///
/// Takes the bearer token by reference rather than `&Request`: `Request<Body>`
/// is `!Sync`, so holding `&Request` across the `.await` points below would make
/// this future `!Send` and the middleware would not satisfy Axum's `from_fn`
/// `Service` bound. The caller extracts the token (an owned `String`) up front.
async fn supabase_claims(state: &AppState, token: &str) -> AppResult<Claims> {
    let cache = state.jwks.as_ref().ok_or_else(|| {
        tracing::error!("Supabase auth hit but SUPABASE_JWKS_URL is not configured");
        AppError::Internal(anyhow::anyhow!("Supabase JWKS not configured"))
    })?;

    let supa = jwt::verify_supabase(
        cache,
        token,
        &state.config.auth.supabase_audience,
        state.config.auth.supabase_issuer.as_deref(),
    )
    .await?;

    let user_id = supa.sub.parse().map_err(|_| AppError::Unauthorized)?;
    // Identity comes from Supabase; authorization role comes from our DB.
    let role = roles::lookup(&state.db, user_id).await?;

    Ok(Claims {
        sub: user_id,
        iat: supa.iat,
        exp: supa.exp,
        iss: supa.iss.unwrap_or_else(|| state.config.auth.issuer.clone()),
        aud: supa.aud,
        role,
    })
}

/// Verify a Supabase-issued RS256 token; stash role-resolved `Claims`.
pub async fn require_supabase_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    // Own the token before any `.await` so we never hold `&Request` (which is
    // `!Send`) across an await point — see `supabase_claims`.
    let token = extract_bearer(&req)?.to_owned();
    let claims = supabase_claims(&state, &token).await?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// Like `require_supabase_auth` plus a `role == Admin` gate.
pub async fn require_supabase_admin(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let token = extract_bearer(&req)?.to_owned();
    let claims = supabase_claims(&state, &token).await?;
    if claims.role != Role::Admin {
        return Err(AppError::Forbidden);
    }
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn service_admin_claims(state: &AppState, token: &str, headers: &HeaderMap) -> AppResult<Claims> {
    let expected = state
        .config
        .auth
        .internal_admin_token
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or(AppError::Unauthorized)?;
    if token != expected {
        return Err(AppError::Unauthorized);
    }

    let original_user = headers
        .get(HEADER_ORIGINAL_USER)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    let sub = Uuid::parse_str(original_user).map_err(|_| AppError::Unauthorized)?;
    let now = Utc::now().timestamp();

    Ok(Claims {
        sub,
        iat: now,
        exp: now + 300,
        iss: "internal-admin".to_string(),
        aud: "stellaux-core-admin".to_string(),
        role: Role::Admin,
    })
}

/// Verify the static service token used by the internal control plane and synthesize
/// admin claims from the propagated `X-Original-User` header.
pub async fn require_internal_admin(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let token = extract_bearer(&req)?.to_owned();
    let claims = service_admin_claims(&state, &token, req.headers())?;
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
    /// The authenticated user's id (Supabase `auth.users.id`). Use this to scope
    /// every user-owned query — e.g. `where user_id = user.user_id()`.
    pub fn user_id(&self) -> uuid::Uuid {
        self.0.sub
    }

    pub fn role(&self) -> Role {
        self.0.role
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.0.role, Role::Admin)
    }

    /// Guard a resource fetched by id against horizontal access: the caller must
    /// either own the resource or be an admin. Returns `Forbidden` otherwise.
    ///
    /// Call this in every handler that loads a resource by a path/query id
    /// rather than by the caller's own id, so customer A can never read or
    /// mutate customer B's data.
    pub fn ensure_owns(&self, owner_id: uuid::Uuid) -> AppResult<()> {
        if self.0.sub == owner_id || self.is_admin() {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::config::{
        AuthConfig, Config, CorsConfig, DatabaseConfig, ResendConfig, ServerConfig, ShippoConfig,
        StorageBackend, StorageConfig, StripeConfig, WarehouseConfig,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn user(sub: Uuid, role: Role) -> AuthUser {
        AuthUser(Claims {
            sub,
            iat: 0,
            exp: 0,
            iss: "test".into(),
            aud: "test".into(),
            role,
        })
    }

    fn app_state_with_internal_token(token: Option<&str>) -> AppState {
        let local_path = std::env::temp_dir()
            .join("stellaux-auth-tests-storage")
            .display()
            .to_string();
        let storage = crate::common::storage::build(&StorageConfig {
            backend: StorageBackend::Local,
            public_base_url: String::new(),
            local_path: local_path.clone(),
            s3_bucket: None,
            s3_endpoint: None,
            s3_region: String::new(),
            s3_access_key_id: None,
            s3_secret_access_key: None,
        })
        .unwrap();
        AppState {
            db: sea_orm::DatabaseConnection::Disconnected,
            config: Arc::new(Config {
                server: ServerConfig {
                    host: "127.0.0.1".into(),
                    port: 8080,
                    environment: "test".into(),
                    request_timeout_secs: 30,
                    body_limit_bytes: 1024,
                    webhook_body_limit_bytes: 1024,
                },
                database: DatabaseConfig {
                    url: "postgres://example".into(),
                    pool_size: 1,
                },
                auth: AuthConfig {
                    jwt_secret: "secret".into(),
                    jwt_expiry_seconds: 3600,
                    issuer: "issuer".into(),
                    audience: "aud".into(),
                    internal_admin_token: token.map(str::to_string),
                    supabase_jwks_url: None,
                    supabase_audience: "authenticated".into(),
                    supabase_issuer: None,
                    jwks_ttl_seconds: 3600,
                },
                cors: CorsConfig { origins: vec![] },
                storage: StorageConfig {
                    backend: StorageBackend::Local,
                    public_base_url: String::new(),
                    local_path,
                    s3_bucket: None,
                    s3_endpoint: None,
                    s3_region: String::new(),
                    s3_access_key_id: None,
                    s3_secret_access_key: None,
                },
                stripe: StripeConfig::default(),
                shippo: ShippoConfig::default(),
                resend: ResendConfig::default(),
                warehouse: WarehouseConfig::default(),
            }),
            http: reqwest::Client::new(),
            storage,
            jwks: None,
        }
    }

    #[test]
    fn owner_passes_ensure_owns() {
        let id = Uuid::new_v4();
        assert!(user(id, Role::Customer).ensure_owns(id).is_ok());
    }

    #[test]
    fn non_owner_customer_is_forbidden() {
        let caller = user(Uuid::new_v4(), Role::Customer);
        let err = caller.ensure_owns(Uuid::new_v4()).unwrap_err();
        assert!(matches!(err, AppError::Forbidden));
    }

    #[test]
    fn admin_can_access_other_users_resources() {
        // Vertical privilege: admins bypass the ownership check entirely.
        let admin = user(Uuid::new_v4(), Role::Admin);
        assert!(admin.ensure_owns(Uuid::new_v4()).is_ok());
    }

    #[test]
    fn non_admin_roles_do_not_bypass_ownership() {
        // Only Admin escalates; Staff/Support must still own the resource.
        for role in [Role::Staff, Role::Support] {
            let caller = user(Uuid::new_v4(), role);
            assert!(
                matches!(caller.ensure_owns(Uuid::new_v4()), Err(AppError::Forbidden)),
                "role {role:?} must not bypass ownership"
            );
        }
    }

    #[test]
    fn internal_admin_token_mints_admin_claims_for_original_user() {
        let state = app_state_with_internal_token(Some("svc-token"));
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, "Bearer svc-token".parse().unwrap());
        headers.insert(
            HEADER_ORIGINAL_USER,
            Uuid::nil().to_string().parse().unwrap(),
        );

        let claims = service_admin_claims(&state, "svc-token", &headers).unwrap();
        assert_eq!(claims.sub, Uuid::nil());
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn internal_admin_token_requires_matching_bearer_and_user_header() {
        let state = app_state_with_internal_token(Some("svc-token"));
        let headers = HeaderMap::new();
        assert!(matches!(
            service_admin_claims(&state, "wrong-token", &headers),
            Err(AppError::Unauthorized)
        ));
    }
}
