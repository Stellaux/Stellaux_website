//! HTTP server: router build, middleware stack, route-group composition,
//! graceful shutdown.
//!
//! Routing groups:
//!   - `public`     — open to anyone (catalog, health, metrics)
//!   - `webhooks`   — open (signature-verified inside handler), large body limit
//!   - `protected`  — JWT required (cart, orders, account)
//!   - `admin`      — JWT required + role == "admin" (admin CRUD, channel sync)

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context;
use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use axum_prometheus::PrometheusMetricLayer;
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

use crate::common::{
    app_state::AppState,
    auth::{require_admin, require_auth},
    config::StorageBackend,
    error::AppResult,
};

const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub async fn run(state: AppState) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{}:{}", state.config.server.host, state.config.server.port)
        .parse()
        .context("parsing server address")?;

    let app = build_router(state.clone());

    let listener = TcpListener::bind(addr).await.context("binding listener")?;
    tracing::info!(
        %addr,
        env = %state.config.server.environment,
        cors_origins = ?state.config.cors.origins,
        request_timeout_s = state.config.server.request_timeout_secs,
        body_limit_bytes = state.config.server.body_limit_bytes,
        webhook_body_limit_bytes = state.config.server.webhook_body_limit_bytes,
        "stellaux_server listening",
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    tracing::info!("server stopped cleanly");
    Ok(())
}

fn build_router(state: AppState) -> Router {
    let (prom_layer, prom_handle) = PrometheusMetricLayer::pair();
    let metrics_handle = prom_handle.clone();
    // Recorder is installed by ::pair() — safe to register descriptions now.
    crate::common::metrics::install_descriptions();

    let timeout = Duration::from_secs(state.config.server.request_timeout_secs);
    let body_limit = state.config.server.body_limit_bytes;
    let webhook_body_limit = state.config.server.webhook_body_limit_bytes;
    let in_prod = state.config.server.is_prod();

    // ── Public: no auth, standard body limit ─────────────────────────────
    let mut public: Router<AppState> = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/metrics",
            get(move || {
                let handle = metrics_handle.clone();
                async move { handle.render() }
            }),
        )
        .nest("/api/v1/catalog", crate::domain::catalog::routes())
        .nest("/api/v1/craft", crate::domain::craft::routes())
        .nest("/api/v1/auth", crate::domain::auth::routes());

    // Local backend: serve uploaded assets from this server. S3/R2 backends
    // resolve via the bucket's public URL, so no route is mounted.
    if state.config.storage.backend == StorageBackend::Local {
        public = public.nest_service(
            "/storage",
            ServeDir::new(&state.config.storage.local_path),
        );
    }

    let public = public.layer(RequestBodyLimitLayer::new(body_limit));

    // ── Webhooks: no auth (handler verifies signature), large body limit ─
    let webhooks: Router<AppState> = Router::new()
        .nest("/api/v1/webhooks", crate::domain::webhooks::routes())
        .layer(RequestBodyLimitLayer::new(webhook_body_limit));

    // ── Protected: any valid JWT ─────────────────────────────────────────
    // NOTE: `require_auth` accepts HS256 tokens we issue. For Supabase-signed
    // tokens from the frontend, swap to `require_supabase_auth` (also imported
    // from `crate::common::auth`) and ensure SUPABASE_JWKS_URL is set.
    let protected: Router<AppState> = Router::new()
        .nest("/api/v1/cart", crate::domain::cart::routes())
        .nest("/api/v1/checkout", crate::domain::checkout::routes())
        .nest("/api/v1/account", crate::domain::account::routes())
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .layer(RequestBodyLimitLayer::new(body_limit));

    // ── Admin: valid JWT + role == "admin" ───────────────────────────────
    let admin: Router<AppState> = Router::new()
        .nest("/api/v1/admin", crate::domain::admin::routes())
        .layer(middleware::from_fn_with_state(state.clone(), require_admin))
        .layer(RequestBodyLimitLayer::new(body_limit));

    // ── Global middleware (applied to all merged routes) ─────────────────
    Router::new()
        .merge(public)
        .merge(webhooks)
        .merge(protected)
        .merge(admin)
        .layer(middleware::from_fn_with_state(timeout, request_timeout))
        .layer(
            ServiceBuilder::new()
                // outermost: catch handler panics so the server stays up
                .layer(CatchPanicLayer::new())
                // mask Authorization in trace output
                .layer(SetSensitiveRequestHeadersLayer::new(std::iter::once(
                    header::AUTHORIZATION,
                )))
                // generate x-request-id if absent, propagate it back
                .layer(SetRequestIdLayer::new(REQUEST_ID.clone(), MakeRequestUuid))
                .layer(TraceLayer::new_for_http())
                .layer(PropagateRequestIdLayer::new(REQUEST_ID.clone()))
                // OWASP response headers
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::X_CONTENT_TYPE_OPTIONS,
                    HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::X_FRAME_OPTIONS,
                    HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::REFERRER_POLICY,
                    HeaderValue::from_static("strict-origin-when-cross-origin"),
                ))
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::X_XSS_PROTECTION,
                    HeaderValue::from_static("0"),
                ))
                .layer(SetResponseHeaderLayer::if_not_present(
                    HeaderName::from_static("permissions-policy"),
                    HeaderValue::from_static(
                        "accelerometer=(), camera=(), geolocation=(), gyroscope=(), \
                         magnetometer=(), microphone=(), payment=(), usb=()",
                    ),
                ))
                // HSTS only when we know we're behind TLS (typically in prod)
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::STRICT_TRANSPORT_SECURITY,
                    if in_prod {
                        HeaderValue::from_static("max-age=31536000; includeSubDomains")
                    } else {
                        HeaderValue::from_static("max-age=0")
                    },
                ))
                .layer(CompressionLayer::new())
                .layer(prom_layer)
                .layer(build_cors(&state.config.cors.origins)),
        )
        .with_state(state)
}

fn build_cors(origins: &[String]) -> CorsLayer {
    let mut parsed = Vec::with_capacity(origins.len());
    for raw in origins {
        match HeaderValue::from_str(raw) {
            Ok(v) => parsed.push(v),
            Err(err) => tracing::warn!(origin = %raw, %err, "ignoring invalid CORS origin"),
        }
    }
    if origins.is_empty() {
        tracing::warn!(
            "CORS_ORIGINS is empty — all cross-origin requests will be blocked. \
             Set CORS_ORIGINS to a comma-separated allowlist for browser callers."
        );
    }

    let allow_origin = AllowOrigin::list(parsed);

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

// ─── liveness / readiness / metrics ─────────────────────────────────────────

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(State(state): State<AppState>) -> AppResult<&'static str> {
    state.db.ping().await?;
    Ok("ready")
}

async fn request_timeout(
    State(timeout): State<Duration>,
    req: Request,
    next: middleware::Next,
) -> Response {
    match tokio::time::timeout(timeout, next.run(req)).await {
        Ok(response) => response,
        Err(_) => StatusCode::REQUEST_TIMEOUT.into_response(),
    }
}

// ─── graceful shutdown ──────────────────────────────────────────────────────

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c   => tracing::info!("ctrl-c received, shutting down"),
        _ = terminate => tracing::info!("SIGTERM received, shutting down"),
    }
}
