//! Shared idempotency primitives for requests, webhooks, and background jobs.
//!
//! Phase 0 only defines the contract. Persistence and enforcement live in
//! domain-specific infra code once order, webhook, and marketplace flows land.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdempotencyScope {
    HttpRequest,
    WebhookEvent,
    BackgroundJob,
}

impl IdempotencyScope {
    pub fn as_str(self) -> &'static str {
        match self {
            IdempotencyScope::HttpRequest => "http_request",
            IdempotencyScope::WebhookEvent => "webhook_event",
            IdempotencyScope::BackgroundJob => "background_job",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Parse and normalize an idempotency key from headers or external event ids.
    pub fn parse(raw: &str) -> Option<Self> {
        let normalized = raw.trim();
        if normalized.is_empty() || normalized.len() > 128 {
            return None;
        }
        if normalized.chars().any(|ch| ch.is_control()) {
            return None;
        }

        Some(Self(normalized.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyRecord {
    pub key: IdempotencyKey,
    pub scope: IdempotencyScope,
    pub source: String,
    pub fingerprint: String,
}

impl IdempotencyRecord {
    pub fn new(
        key: IdempotencyKey,
        scope: IdempotencyScope,
        source: impl Into<String>,
        fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            key,
            scope,
            source: source.into(),
            fingerprint: fingerprint.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IdempotencyKey, IdempotencyScope};

    #[test]
    fn key_parse_trims_whitespace() {
        let key = IdempotencyKey::parse("  evt_123  ").expect("key should parse");
        assert_eq!(key.as_str(), "evt_123");
    }

    #[test]
    fn key_parse_rejects_empty_values() {
        assert!(IdempotencyKey::parse("   ").is_none());
    }

    #[test]
    fn key_parse_rejects_control_characters() {
        assert!(IdempotencyKey::parse("evt_\n123").is_none());
    }

    #[test]
    fn scope_names_are_stable() {
        assert_eq!(IdempotencyScope::HttpRequest.as_str(), "http_request");
        assert_eq!(IdempotencyScope::WebhookEvent.as_str(), "webhook_event");
        assert_eq!(IdempotencyScope::BackgroundJob.as_str(), "background_job");
    }
}
