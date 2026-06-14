//! JWT issuance and verification.
//!
//! Two verification paths are supported:
//!
//! - **HS256** (`verify`): tokens issued by *this* service (`issue`). Symmetric
//!   secret, validated against configured `iss` + `aud`.
//! - **RS256 + JWKS** (`verify_supabase`): tokens issued by Supabase. Asymmetric;
//!   the public key set is fetched once and cached with a TTL.
//!
//! Future hardening (not in this revision):
//!   - Wrap the HS256 secret in `secrecy::Zeroizing` to defeat memory scrapers.
//!   - Add `kid` to our own headers + a key map for rotation.
//!   - Add `jti` + a revocation list (Redis) for explicit logout.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode,
    jwk::JwkSet,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::error::{AppError, AppResult};

// ─── Role enum (typo-proof) ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Customer,
    Support,
    Staff,
    Admin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Customer => "customer",
            Role::Support => "support",
            Role::Staff => "staff",
            Role::Admin => "admin",
        }
    }

    /// Parse a role string stored in `user_roles.role`. Unknown or absent values
    /// degrade to `Customer` — the default is least privilege, so a missing or
    /// malformed row can never silently escalate access.
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "admin" => Role::Admin,
            "staff" => Role::Staff,
            "support" => Role::Support,
            _ => Role::Customer,
        }
    }
}

// ─── Internal HS256 claims ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,   // user id
    pub iat: i64,    // issued-at (unix seconds)
    pub exp: i64,    // expiry (unix seconds)
    pub iss: String, // issuer
    pub aud: String, // audience
    pub role: Role,
}

/// Issue an HS256 token signed by our shared secret.
pub fn issue(
    secret: &[u8],
    user_id: Uuid,
    role: Role,
    ttl_seconds: i64,
    issuer: &str,
    audience: &str,
) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: (now + Duration::seconds(ttl_seconds)).timestamp(),
        iss: issuer.to_string(),
        aud: audience.to_string(),
        role,
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;
    Ok(token)
}

/// Verify an HS256 token issued by `issue()`. Validates algorithm, expiry,
/// issuer, audience, and required-claim presence.
pub fn verify(secret: &[u8], token: &str, issuer: &str, audience: &str) -> AppResult<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 5; // seconds of clock-skew tolerance
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);
    validation.required_spec_claims = required_claims(&["exp", "iat", "iss", "aud", "sub"]);

    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(data.claims)
}

// ─── Supabase RS256 verification ────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupabaseClaims {
    pub sub: String, // user UUID as string
    pub aud: String, // "authenticated"
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub iss: Option<String>, // typically https://<project>.supabase.co/auth/v1
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub role: Option<String>, // Postgres role: "authenticated" | "anon" | "service_role"
}

/// Verify a Supabase-issued RS256 token using the cached JWKS.
pub async fn verify_supabase(
    cache: &JwksCache,
    token: &str,
    audience: &str,
    issuer: Option<&str>,
) -> AppResult<SupabaseClaims> {
    let header = decode_header(token).map_err(|_| AppError::Unauthorized)?;
    let kid = header.kid.ok_or(AppError::Unauthorized)?;
    let key = cache.decoding_key(&kid).await?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.leeway = 5;
    validation.set_audience(&[audience]);
    if let Some(iss) = issuer {
        validation.set_issuer(&[iss]);
    }
    validation.required_spec_claims = required_claims(&["exp", "iat", "aud", "sub"]);

    let data = decode::<SupabaseClaims>(token, key.as_ref(), &validation)
        .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

fn required_claims(names: &[&str]) -> HashSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod role_tests {
    use super::Role;

    #[test]
    fn known_roles_parse_exactly() {
        assert_eq!(Role::from_db_str("admin"), Role::Admin);
        assert_eq!(Role::from_db_str("staff"), Role::Staff);
        assert_eq!(Role::from_db_str("support"), Role::Support);
        assert_eq!(Role::from_db_str("customer"), Role::Customer);
    }

    #[test]
    fn unknown_or_malformed_degrades_to_customer() {
        // Least privilege: anything we don't recognize can never escalate.
        for s in ["", "ADMIN", "Admin", "root", "superuser", " admin"] {
            assert_eq!(
                Role::from_db_str(s),
                Role::Customer,
                "{s:?} must degrade to Customer"
            );
        }
    }

    #[test]
    fn as_str_roundtrips_through_from_db_str() {
        for role in [Role::Customer, Role::Support, Role::Staff, Role::Admin] {
            assert_eq!(Role::from_db_str(role.as_str()), role);
        }
    }
}

// ─── JWKS cache ─────────────────────────────────────────────────────────────

/// Thread-safe TTL-cached JWKS. Lives on `AppState` (wrapped in `Arc`) so the
/// inner state is shared across all handlers.
pub struct JwksCache {
    http: reqwest::Client,
    url: String,
    ttl: Duration,
    state: tokio::sync::RwLock<JwksState>,
}

struct JwksState {
    keys: HashMap<String, Arc<DecodingKey>>,
    last_refresh: Option<chrono::DateTime<Utc>>,
}

impl JwksCache {
    pub fn new(http: reqwest::Client, url: String, ttl_seconds: i64) -> Self {
        Self {
            http,
            url,
            ttl: Duration::seconds(ttl_seconds),
            state: tokio::sync::RwLock::new(JwksState {
                keys: HashMap::new(),
                last_refresh: None,
            }),
        }
    }

    /// Look up a decoding key by `kid`. Refreshes the cache if the key is
    /// missing OR the cache is older than `ttl`.
    pub async fn decoding_key(&self, kid: &str) -> AppResult<Arc<DecodingKey>> {
        // Fast path: in-cache and fresh.
        {
            let state = self.state.read().await;
            if let Some(last) = state.last_refresh {
                let fresh = Utc::now().signed_duration_since(last) < self.ttl;
                if fresh && let Some(key) = state.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        // Slow path: refresh, then re-check.
        self.refresh().await?;
        let state = self.state.read().await;
        state.keys.get(kid).cloned().ok_or_else(|| {
            tracing::warn!(kid, "JWT kid not present in JWKS after refresh");
            AppError::Unauthorized
        })
    }

    async fn refresh(&self) -> AppResult<()> {
        tracing::debug!(url = %self.url, "refreshing JWKS");
        let resp = self.http.get(&self.url).send().await?.error_for_status()?;
        let jwks: JwkSet = resp.json().await?;

        let mut new_keys: HashMap<String, Arc<DecodingKey>> = HashMap::new();
        for jwk in &jwks.keys {
            let Some(kid) = jwk.common.key_id.as_ref() else {
                continue;
            };
            match DecodingKey::from_jwk(jwk) {
                Ok(key) => {
                    new_keys.insert(kid.clone(), Arc::new(key));
                }
                Err(err) => {
                    tracing::warn!(%kid, %err, "skipping unparseable JWK");
                }
            }
        }

        tracing::info!(count = new_keys.len(), "JWKS refreshed");

        let mut state = self.state.write().await;
        state.keys = new_keys;
        state.last_refresh = Some(Utc::now());
        Ok(())
    }
}
