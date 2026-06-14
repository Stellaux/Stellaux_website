/// Adapter-layer representation of an incoming webhook before signature
/// verification and domain-level normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawWebhookRequest {
    pub source: String,
    pub signature_header: Option<String>,
    pub payload: Vec<u8>,
}
