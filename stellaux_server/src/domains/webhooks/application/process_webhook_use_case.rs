use crate::common::idempotency::IdempotencyKey;
use crate::domains::webhooks::domain::webhook_event::WebhookSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessWebhookCommand {
    pub source: WebhookSource,
    pub external_event_id: String,
    pub idempotency_key: IdempotencyKey,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessWebhookOutcome {
    Recorded,
    Duplicate,
    Processed,
    Failed,
}
