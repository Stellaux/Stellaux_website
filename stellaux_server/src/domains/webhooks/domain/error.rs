#[derive(Debug, thiserror::Error)]
pub enum WebhookContractError {
    #[error("invalid webhook signature")]
    InvalidSignature,

    #[error("duplicate webhook event")]
    DuplicateEvent,

    #[error("persistence failure")]
    PersistenceFailure,

    #[error("{0}")]
    Other(String),
}
