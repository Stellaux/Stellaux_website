//! Process-level initialization. Called once at start of `main`.
//!
//! Order matters:
//!   1. tracing (so subsequent steps can emit logs)
//!   2. config (everything else needs it)
//!   3. db connect + ping (fail fast if Postgres is unreachable)
//!   4. outbound http client (Stripe/Shippo/Resend share one)

use std::io::IsTerminal;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::common::{
    app_state::AppState,
    config::{Config, DatabaseConfig},
    jwt::JwksCache,
};

pub async fn init() -> anyhow::Result<AppState> {
    init_tracing();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "starting stellaux_server"
    );

    let config = Config::from_env().context("loading config")?;
    let db = connect_db(&config.database)
        .await
        .context("connecting to database")?;
    let http = build_http_client().context("building http client")?;

    let jwks = match config.auth.supabase_jwks_url.as_ref() {
        Some(url) => {
            tracing::info!(%url, "Supabase JWKS verification enabled");
            Some(Arc::new(JwksCache::new(
                http.clone(),
                url.clone(),
                config.auth.jwks_ttl_seconds,
            )))
        }
        None => {
            tracing::info!("Supabase JWKS verification disabled (SUPABASE_JWKS_URL unset)");
            None
        }
    };

    tracing::info!(
        env = %config.server.environment,
        host = %config.server.host,
        port = config.server.port,
        cors_origins = config.cors.origins.len(),
        request_timeout_s = config.server.request_timeout_secs,
        body_limit_bytes = config.server.body_limit_bytes,
        webhook_body_limit_bytes = config.server.webhook_body_limit_bytes,
        "bootstrap complete"
    );

    Ok(AppState {
        db,
        config: Arc::new(config),
        http,
        jwks,
    })
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_ansi(std::io::stdout().is_terminal());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

async fn connect_db(cfg: &DatabaseConfig) -> anyhow::Result<DatabaseConnection> {
    let mut opts = ConnectOptions::new(&cfg.url);
    opts.max_connections(cfg.pool_size)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(true);

    let db = Database::connect(opts).await?;
    db.ping().await.context("postgres ping failed")?;
    tracing::info!(pool_size = cfg.pool_size, "database connected");
    Ok(db)
}

fn build_http_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(90))
        .user_agent(concat!("stellaux-server/", env!("CARGO_PKG_VERSION")))
        .build()?)
}
