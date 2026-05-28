//! OpenTelemetry exporter wiring. Opt-in.
//!
//! ## Why opt-in
//!
//! OTel pulls ~30 transitive crates and adds ~60s of first-build compile time.
//! Most local dev doesn't need it — `tracing-subscriber` JSON output is enough.
//! Enable when you stand up a collector (Tempo / Jaeger / Honeycomb / Datadog).
//!
//! ## Step 1 — add deps to `Cargo.toml` (behind a feature)
//!
//! ```toml
//! [features]
//! default = []
//! otel = [
//!   "dep:opentelemetry",
//!   "dep:opentelemetry_sdk",
//!   "dep:opentelemetry-otlp",
//!   "dep:tracing-opentelemetry",
//! ]
//!
//! [dependencies]
//! opentelemetry        = { version = "0.27", features = ["trace"], optional = true }
//! opentelemetry_sdk    = { version = "0.27", features = ["rt-tokio"], optional = true }
//! opentelemetry-otlp   = { version = "0.27", features = ["grpc-tonic", "trace"], optional = true }
//! tracing-opentelemetry = { version = "0.28", optional = true }
//! ```
//!
//! Build with `cargo build --features otel`.
//!
//! ## Step 2 — implementation (replace the stub below)
//!
//! ```ignore
//! #[cfg(feature = "otel")]
//! pub fn init_otel(endpoint: &str, service_name: &str) -> anyhow::Result<opentelemetry_sdk::trace::Tracer> {
//!     use opentelemetry::trace::TracerProvider;
//!     use opentelemetry_otlp::SpanExporter;
//!     use opentelemetry_sdk::{Resource, trace as sdktrace};
//!
//!     let exporter = SpanExporter::builder()
//!         .with_tonic()
//!         .with_endpoint(endpoint)
//!         .build()?;
//!
//!     let provider = sdktrace::SdkTracerProvider::builder()
//!         .with_batch_exporter(exporter)
//!         .with_resource(
//!             Resource::builder()
//!                 .with_service_name(service_name.to_string())
//!                 .build(),
//!         )
//!         .build();
//!
//!     let tracer = provider.tracer(service_name.to_string());
//!     opentelemetry::global::set_tracer_provider(provider);
//!     Ok(tracer)
//! }
//! ```
//!
//! ## Step 3 — wire into `bootstrap::init_tracing`
//!
//! ```ignore
//! #[cfg(feature = "otel")]
//! if let Ok(endpoint) = std::env::var("OTLP_ENDPOINT") {
//!     match crate::common::opentelemetry::init_otel(&endpoint, "stellaux_server") {
//!         Ok(tracer) => {
//!             let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
//!             // attach `otel_layer` to the registry alongside the fmt layer
//!         }
//!         Err(e) => eprintln!("OTel init failed (continuing without it): {e}"),
//!     }
//! }
//! ```
//!
//! ## Step 4 — shut down on drop
//!
//! Call `opentelemetry::global::shutdown_tracer_provider()` after `axum::serve`
//! returns in `server::run`, so buffered spans flush before the process exits.

// Stub no-op so callers compile without the `otel` feature.
pub fn init_otel(_endpoint: &str, _service_name: &str) -> anyhow::Result<()> {
    Ok(())
}
