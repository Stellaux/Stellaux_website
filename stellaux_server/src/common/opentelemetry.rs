//! OpenTelemetry exporter wiring. Opt-in — enable the opentelemetry deps in
//! `Cargo.toml`, then call `init_otel(&config)` from `bootstrap::init_tracing`
//! and attach the layer to the registry.
//!
//! Sketch:
//!
//! ```ignore
//! use opentelemetry_otlp::WithExportConfig;
//! use opentelemetry_sdk::trace as sdktrace;
//! use opentelemetry::trace::TracerProvider;
//!
//! pub fn init_otel(endpoint: &str) -> anyhow::Result<sdktrace::Tracer> {
//!     let exporter = opentelemetry_otlp::SpanExporter::builder()
//!         .with_tonic()
//!         .with_endpoint(endpoint)
//!         .build()?;
//!     let provider = sdktrace::SdkTracerProvider::builder()
//!         .with_batch_exporter(exporter)
//!         .build();
//!     let tracer = provider.tracer("stellaux_server");
//!     opentelemetry::global::set_tracer_provider(provider);
//!     Ok(tracer)
//! }
//! ```
//!
//! Then in `bootstrap::init_tracing`:
//!
//! ```ignore
//! let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
//! tracing_subscriber::registry()
//!     .with(env_filter)
//!     .with(fmt_layer)
//!     .with(otel_layer)
//!     .init();
//! ```
