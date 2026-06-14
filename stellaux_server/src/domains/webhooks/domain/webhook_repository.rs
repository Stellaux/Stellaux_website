use async_trait::async_trait;

use crate::domains::webhooks::domain::{
    error::WebhookContractError,
    webhook_event::{WebhookEventRecord, WebhookSource},
};

#[async_trait]
pub trait WebhookEventRepository: Send + Sync {
    async fn find_by_source_and_external_id(
        &self,
        source: WebhookSource,
        external_event_id: &str,
    ) -> Result<Option<WebhookEventRecord>, WebhookContractError>;

    async fn record_receipt(&self, event: &WebhookEventRecord) -> Result<(), WebhookContractError>;

    async fn mark_processed(
        &self,
        source: WebhookSource,
        external_event_id: &str,
    ) -> Result<(), WebhookContractError>;

    async fn mark_failed(
        &self,
        source: WebhookSource,
        external_event_id: &str,
        reason: &str,
    ) -> Result<(), WebhookContractError>;
}
