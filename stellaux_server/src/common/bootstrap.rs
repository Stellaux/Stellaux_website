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
    storage,
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

    // Schema ownership: the Supabase SQL migrations (`supabase/migrations/`, applied out-of-band via
    // `supabase db push`, split across stellaux_server and the internal dashboard) are the single
    // source of truth. The original sea-orm migrator (`src/migration/`) was abandoned in that switch
    // and is intentionally NOT run at boot: its schema diverged (old `craft_role` catalog model) and
    // would collide with the live schema. The server reads via raw SQL against the authoritative
    // tables (see `domains/*/api/routes.rs`); it does not depend on the sea-orm entities/migrator.
    tracing::info!("skipping sea-orm migrator; schema owned by Supabase migrations");

    let http = build_http_client().context("building http client")?;

    let storage = storage::build(&config.storage).context("initializing storage")?;
    tracing::info!(
        backend = ?config.storage.backend,
        public_url = %config.storage.public_base_url,
        "storage ready"
    );

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
        storage,
        jwks,
    })
}

fn init_tracing() {
    // APP_ENV is read directly here (Config isn't loaded yet — tracing must
    // come first so subsequent steps can emit logs).
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".into());
    let is_prod = env.eq_ignore_ascii_case("prod") || env.eq_ignore_ascii_case("production");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let registry = tracing_subscriber::registry().with(env_filter);

    if is_prod {
        // Structured JSON — one log object per line, ready for Loki/CloudWatch/etc.
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_current_span(true)
                    .with_span_list(false),
            )
            .init();
    } else {
        // Human-readable with optional ANSI when attached to a TTY.
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_line_number(false)
                    .with_ansi(std::io::stdout().is_terminal()),
            )
            .init();
    }
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
