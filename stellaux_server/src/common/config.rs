//! Environment-driven configuration. Loaded once at boot via `Config::from_env`.
//!
//! Convention: secrets live in env vars / Docker secrets; this struct never logs
//! itself via `Debug` in production (we hand-pick what's safe to surface).

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub cors: CorsConfig,
    pub storage: StorageConfig,
    pub stripe: StripeConfig,
    pub shippo: ShippoConfig,
    pub resend: ResendConfig,
    pub warehouse: WarehouseConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    Local,
    S3,
}

impl StorageBackend {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "s3" | "r2" | "minio" => StorageBackend::S3,
            _ => StorageBackend::Local,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub backend: StorageBackend,
    /// Base URL clients hit to fetch stored assets.
    /// Local: `http://localhost:8080/storage` (served by this server)
    /// R2:    `https://cdn.themaisonaure.com` (CDN in front of bucket)
    pub public_base_url: String,
    pub local_path: String,
    pub s3_bucket: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_region: String,
    pub s3_access_key_id: Option<String>,
    pub s3_secret_access_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String, // "dev" | "staging" | "prod"
    pub request_timeout_secs: u64,
    pub body_limit_bytes: usize,
    pub webhook_body_limit_bytes: usize,
}

impl ServerConfig {
    pub fn is_prod(&self) -> bool {
        self.environment.eq_ignore_ascii_case("prod")
            || self.environment.eq_ignore_ascii_case("production")
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_seconds: i64,
    pub issuer: String,                    // `iss` baked into our HS256 tokens
    pub audience: String,                  // `aud` baked into our HS256 tokens
    pub supabase_jwks_url: Option<String>, // None disables Supabase verification
    pub supabase_audience: String,         // typically "authenticated"
    pub supabase_issuer: Option<String>,   // e.g. https://<proj>.supabase.co/auth/v1
    pub jwks_ttl_seconds: i64,             // how long to trust cached JWKS
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub origins: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Clone, Default)]
pub struct ShippoConfig {
    pub api_token: String,
    pub webhook_secret: String,
}

#[derive(Debug, Clone, Default)]
pub struct ResendConfig {
    pub api_key: String,
    pub from_email: String,
}

#[derive(Debug, Clone, Default)]
pub struct WarehouseConfig {
    pub name: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub phone: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // `.env` is best-effort: useful for local dev, no-op in container/prod
        // where vars come from the platform.
        let _ = dotenvy::dotenv();

        Ok(Self {
            server: ServerConfig {
                host: env_or("SERVER_HOST", "0.0.0.0"),
                port: env_parse_or("SERVER_PORT", 8080u16)?,
                environment: env_or("APP_ENV", "dev"),
                request_timeout_secs: env_parse_or("REQUEST_TIMEOUT_SECS", 30u64)?,
                body_limit_bytes: env_parse_or("BODY_LIMIT_BYTES", 2 * 1024 * 1024usize)?,
                webhook_body_limit_bytes: env_parse_or(
                    "WEBHOOK_BODY_LIMIT_BYTES",
                    10 * 1024 * 1024usize,
                )?,
            },
            database: DatabaseConfig {
                url: env_required("DATABASE_URL")?,
                pool_size: env_parse_or("DATABASE_POOL_SIZE", 10u32)?,
            },
            auth: AuthConfig {
                jwt_secret: env_required("JWT_SECRET")?,
                jwt_expiry_seconds: env_parse_or("JWT_EXPIRY_SECONDS", 3600i64)?,
                issuer: env_or("JWT_ISSUER", "stellaux-api"),
                audience: env_or("JWT_AUDIENCE", "stellaux-clients"),
                supabase_jwks_url: env::var("SUPABASE_JWKS_URL").ok().filter(|s| !s.is_empty()),
                supabase_audience: env_or("SUPABASE_AUDIENCE", "authenticated"),
                supabase_issuer: env::var("SUPABASE_ISSUER").ok().filter(|s| !s.is_empty()),
                jwks_ttl_seconds: env_parse_or("JWKS_TTL_SECONDS", 3600i64)?,
            },
            cors: CorsConfig {
                origins: env_or("CORS_ORIGINS", "")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            },
            storage: StorageConfig {
                backend: StorageBackend::parse(&env_or("STORAGE_BACKEND", "local")),
                public_base_url: env_or("STORAGE_PUBLIC_URL", "http://localhost:8080/storage"),
                local_path: env_or("STORAGE_LOCAL_PATH", "./storage"),
                s3_bucket: env::var("STORAGE_S3_BUCKET").ok().filter(|s| !s.is_empty()),
                s3_endpoint: env::var("STORAGE_S3_ENDPOINT")
                    .ok()
                    .filter(|s| !s.is_empty()),
                s3_region: env_or("STORAGE_S3_REGION", "auto"),
                s3_access_key_id: env::var("STORAGE_S3_ACCESS_KEY_ID")
                    .ok()
                    .filter(|s| !s.is_empty()),
                s3_secret_access_key: env::var("STORAGE_S3_SECRET_ACCESS_KEY")
                    .ok()
                    .filter(|s| !s.is_empty()),
            },
            stripe: StripeConfig {
                secret_key: env_or("STRIPE_SECRET_KEY", ""),
                webhook_secret: env_or("STRIPE_WEBHOOK_SECRET", ""),
                success_url: env_or("STRIPE_SUCCESS_URL", ""),
                cancel_url: env_or("STRIPE_CANCEL_URL", ""),
            },
            shippo: ShippoConfig {
                api_token: env_or("SHIPPO_API_TOKEN", ""),
                webhook_secret: env_or("SHIPPO_WEBHOOK_SECRET", ""),
            },
            resend: ResendConfig {
                api_key: env_or("RESEND_API_KEY", ""),
                from_email: env_or("RESEND_FROM_EMAIL", ""),
            },
            warehouse: WarehouseConfig {
                name: env_or("WAREHOUSE_NAME", ""),
                street: env_or("WAREHOUSE_STREET", ""),
                city: env_or("WAREHOUSE_CITY", ""),
                state: env_or("WAREHOUSE_STATE", ""),
                postal_code: env_or("WAREHOUSE_POSTAL_CODE", ""),
                country: env_or("WAREHOUSE_COUNTRY", "US"),
                phone: env_or("WAREHOUSE_PHONE", ""),
            },
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse_or<T>(key: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(v) => v
            .parse()
            .map_err(|e: T::Err| anyhow::anyhow!("invalid {key}: {e}")),
        Err(_) => Ok(default),
    }
}

fn env_required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("missing required env var: {key}"))
}
