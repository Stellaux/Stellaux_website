use crate::common::idempotency::IdempotencyKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebhookSource {
    Stripe,
    Shippo,
    Ebay,
    Etsy,
}

impl WebhookSource {
    pub fn as_str(self) -> &'static str {
        match self {
            WebhookSource::Stripe => "stripe",
            WebhookSource::Shippo => "shippo",
            WebhookSource::Ebay => "ebay",
            WebhookSource::Etsy => "etsy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookProcessingStatus {
    Received,
    Processed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebhookEventRecord {
    pub source: WebhookSource,
    pub external_event_id: String,
    pub event_type: String,
    pub idempotency_key: IdempotencyKey,
    pub status: WebhookProcessingStatus,
}
