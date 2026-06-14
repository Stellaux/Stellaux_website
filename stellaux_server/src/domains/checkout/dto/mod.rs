use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Address {
    pub recipient: String,
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShippingRatesRequest {
    pub cart_id: Uuid,
    pub address: Address,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub cart_id: Uuid,
    pub shippo_rate_id: String,
    pub address: Address,
}
