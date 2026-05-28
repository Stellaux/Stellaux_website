//! Audit logging for business events. Records *what happened* in a way that is
//! independently queryable from technical logs.
//!
//! Events emit to the `audit` tracing target — filterable via:
//!
//! ```ignore
//! RUST_LOG=info,audit=info
//! ```
//!
//! and routable to a separate sink (file, S3, datalake) via a dedicated
//! subscriber layer. In v2, events will additionally be inserted into a
//! `public.audit_log` Postgres table for in-app search and compliance.
//!
//! Usage:
//!
//! ```ignore
//! use crate::common::audit::{Actor, Subject, events};
//!
//! events::order_created(order_id, user_id, total_cents, "website");
//! events::order_refunded(order_id, admin_user_id, 4200);
//! events::inventory_adjusted(variant_id, Actor::User { id, role: "admin".into() }, -1, "shrinkage");
//! ```

use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

/// Who performed the action.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Actor {
    User { id: Uuid, role: String },
    System,
    Webhook { source: String },
    Anonymous,
}

/// What the action targeted.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Subject {
    Order { id: Uuid },
    Product { id: Uuid },
    Variant { id: Uuid },
    Cart { id: Uuid },
    User { id: Uuid },
    InventoryVariant { id: Uuid },
    Webhook { external_id: String },
    Other { name: String },
}

/// Emit an audit event. Non-fatal — never blocks the request path.
///
/// `action` should be dot-namespaced (`order.created`, `inventory.adjusted`).
pub fn record(action: &str, actor: Actor, subject: Subject, metadata: Option<Value>) {
    let actor_json = serde_json::to_string(&actor).unwrap_or_else(|_| "{}".into());
    let subject_json = serde_json::to_string(&subject).unwrap_or_else(|_| "{}".into());
    let metadata_json = metadata
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok())
        .unwrap_or_else(|| "{}".into());

    // Tracing fields: avoid the reserved name `target`; use `subject` instead.
    tracing::info!(
        target: "audit",
        action,
        actor = %actor_json,
        subject = %subject_json,
        metadata = %metadata_json,
        "audit"
    );

    // TODO v2: also INSERT into public.audit_log via sea-orm. Best done from
    // a background task so the request path doesn't pay the DB round-trip.
}

/// Convenience constructors for the events we'll emit most often.
pub mod events {
    use super::*;
    use serde_json::json;

    // ── Orders ──────────────────────────────────────────────────────────────

    pub fn order_created(order_id: Uuid, user_id: Uuid, total_cents: i64, source: &str) {
        record(
            "order.created",
            Actor::User {
                id: user_id,
                role: "customer".into(),
            },
            Subject::Order { id: order_id },
            Some(json!({ "total_cents": total_cents, "source": source })),
        );
    }

    pub fn order_paid(order_id: Uuid, stripe_payment_intent: &str, amount_cents: i64) {
        record(
            "order.paid",
            Actor::System,
            Subject::Order { id: order_id },
            Some(json!({
                "stripe_payment_intent": stripe_payment_intent,
                "amount_cents": amount_cents,
            })),
        );
    }

    pub fn order_fulfilled(order_id: Uuid, tracking_number: &str, carrier: &str) {
        record(
            "order.fulfilled",
            Actor::System,
            Subject::Order { id: order_id },
            Some(json!({ "tracking_number": tracking_number, "carrier": carrier })),
        );
    }

    pub fn order_refunded(order_id: Uuid, admin_user_id: Uuid, amount_cents: i64) {
        record(
            "order.refunded",
            Actor::User {
                id: admin_user_id,
                role: "admin".into(),
            },
            Subject::Order { id: order_id },
            Some(json!({ "amount_cents": amount_cents })),
        );
    }

    pub fn order_cancelled(order_id: Uuid, admin_user_id: Uuid, reason: &str) {
        record(
            "order.cancelled",
            Actor::User {
                id: admin_user_id,
                role: "admin".into(),
            },
            Subject::Order { id: order_id },
            Some(json!({ "reason": reason })),
        );
    }

    // ── Inventory ───────────────────────────────────────────────────────────

    pub fn inventory_adjusted(variant_id: Uuid, actor: Actor, delta: i32, reason: &str) {
        record(
            "inventory.adjusted",
            actor,
            Subject::InventoryVariant { id: variant_id },
            Some(json!({ "delta": delta, "reason": reason })),
        );
    }

    pub fn inventory_reserved(variant_id: Uuid, cart_id: Uuid, qty: i32) {
        record(
            "inventory.reserved",
            Actor::System,
            Subject::InventoryVariant { id: variant_id },
            Some(json!({ "cart_id": cart_id, "qty": qty })),
        );
    }

    pub fn inventory_released(variant_id: Uuid, cart_id: Uuid, qty: i32, reason: &str) {
        record(
            "inventory.released",
            Actor::System,
            Subject::InventoryVariant { id: variant_id },
            Some(json!({ "cart_id": cart_id, "qty": qty, "reason": reason })),
        );
    }

    // ── Webhooks ────────────────────────────────────────────────────────────

    pub fn webhook_received(source: &str, external_id: &str, event_type: &str) {
        record(
            "webhook.received",
            Actor::Webhook {
                source: source.into(),
            },
            Subject::Webhook {
                external_id: external_id.into(),
            },
            Some(json!({ "event_type": event_type })),
        );
    }

    pub fn webhook_processed(source: &str, external_id: &str, event_type: &str, success: bool) {
        record(
            if success {
                "webhook.processed"
            } else {
                "webhook.failed"
            },
            Actor::Webhook {
                source: source.into(),
            },
            Subject::Webhook {
                external_id: external_id.into(),
            },
            Some(json!({ "event_type": event_type, "success": success })),
        );
    }

    // ── Auth ────────────────────────────────────────────────────────────────

    pub fn auth_login(user_id: Uuid, method: &str, success: bool) {
        record(
            "auth.login",
            if success {
                Actor::User {
                    id: user_id,
                    role: "unknown".into(),
                }
            } else {
                Actor::Anonymous
            },
            Subject::User { id: user_id },
            Some(json!({ "method": method, "success": success })),
        );
    }

    pub fn auth_signup(user_id: Uuid) {
        record(
            "auth.signup",
            Actor::Anonymous,
            Subject::User { id: user_id },
            None,
        );
    }

    pub fn auth_password_changed(user_id: Uuid) {
        record(
            "auth.password_changed",
            Actor::User {
                id: user_id,
                role: "self".into(),
            },
            Subject::User { id: user_id },
            None,
        );
    }
}
