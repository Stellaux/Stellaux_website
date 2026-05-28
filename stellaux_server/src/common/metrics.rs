//! Custom business metrics emitted via the `metrics` facade.
//!
//! The Prometheus recorder installed by `axum_prometheus::PrometheusMetricLayer::pair()`
//! collects and exposes these on `GET /metrics` alongside HTTP defaults.
//!
//! `install_descriptions()` must be called *after* the recorder is installed —
//! that happens in `server::build_router` right after `PrometheusMetricLayer::pair()`.

/// Register Prometheus help/type descriptions. Idempotent; calling more than
/// once is harmless (later calls overwrite earlier descriptions).
pub fn install_descriptions() {
    metrics::describe_counter!(
        "stellaux_orders_total",
        "Total orders placed, labeled by source (website/etsy/ebay)."
    );
    metrics::describe_counter!(
        "stellaux_payments_total",
        "Stripe payment events, labeled by status (succeeded/failed)."
    );
    metrics::describe_counter!(
        "stellaux_refunds_total",
        "Refunds issued, labeled by reason."
    );
    metrics::describe_counter!(
        "stellaux_webhooks_total",
        "Webhooks received, labeled by source/event/success."
    );
    metrics::describe_counter!(
        "stellaux_inventory_adjustments_total",
        "Inventory adjustments, labeled by reason."
    );
    metrics::describe_counter!(
        "stellaux_inventory_units_consumed_total",
        "Total inventory units removed (orders, shrinkage)."
    );
    metrics::describe_counter!(
        "stellaux_inventory_units_restocked_total",
        "Total inventory units added (restocks, returns)."
    );
    metrics::describe_counter!(
        "stellaux_auth_events_total",
        "Auth events, labeled by kind (login/signup/reset) and success."
    );
    metrics::describe_counter!(
        "stellaux_external_calls_total",
        "Outbound API calls, labeled by service/operation/success."
    );
    metrics::describe_histogram!(
        "stellaux_order_amount_cents",
        "Order total amounts in cents."
    );
    metrics::describe_histogram!(
        "stellaux_external_call_duration_ms",
        "Outbound API call latency in milliseconds."
    );
}

// ─── Order + payment ────────────────────────────────────────────────────────

pub fn order_placed(amount_cents: u64, source: &str) {
    let source = source.to_string();
    metrics::counter!("stellaux_orders_total", "source" => source.clone()).increment(1);
    metrics::histogram!("stellaux_order_amount_cents", "source" => source)
        .record(amount_cents as f64);
}

pub fn payment_event(status: &str) {
    metrics::counter!("stellaux_payments_total", "status" => status.to_string()).increment(1);
}

pub fn refund_issued(reason: &str) {
    metrics::counter!("stellaux_refunds_total", "reason" => reason.to_string()).increment(1);
}

// ─── Webhooks ───────────────────────────────────────────────────────────────

pub fn webhook_received(source: &str, event: &str, success: bool) {
    metrics::counter!(
        "stellaux_webhooks_total",
        "source" => source.to_string(),
        "event" => event.to_string(),
        "success" => success.to_string(),
    )
    .increment(1);
}

// ─── Inventory ──────────────────────────────────────────────────────────────

pub fn inventory_adjusted(reason: &str, delta: i32) {
    metrics::counter!(
        "stellaux_inventory_adjustments_total",
        "reason" => reason.to_string()
    )
    .increment(1);
    if delta < 0 {
        metrics::counter!("stellaux_inventory_units_consumed_total").increment((-delta) as u64);
    } else if delta > 0 {
        metrics::counter!("stellaux_inventory_units_restocked_total").increment(delta as u64);
    }
}

// ─── Auth ───────────────────────────────────────────────────────────────────

pub fn auth_event(kind: &str, success: bool) {
    metrics::counter!(
        "stellaux_auth_events_total",
        "kind" => kind.to_string(),
        "success" => success.to_string(),
    )
    .increment(1);
}

// ─── External calls (Stripe/Shippo/Resend) ──────────────────────────────────

/// Record an outbound API call's outcome + latency.
pub fn external_call(service: &str, operation: &str, duration_ms: u64, success: bool) {
    let service = service.to_string();
    let operation = operation.to_string();
    metrics::counter!(
        "stellaux_external_calls_total",
        "service" => service.clone(),
        "operation" => operation.clone(),
        "success" => success.to_string(),
    )
    .increment(1);
    metrics::histogram!(
        "stellaux_external_call_duration_ms",
        "service" => service,
        "operation" => operation,
    )
    .record(duration_ms as f64);
}
