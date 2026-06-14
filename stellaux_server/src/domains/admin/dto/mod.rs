use serde::Deserialize;
use uuid::Uuid;

use crate::common::dto::Pagination;

#[derive(Debug, Deserialize)]
pub struct OrdersFilter {
    pub status: Option<String>,
    pub source: Option<String>,
    #[serde(flatten)]
    pub page: Pagination,
}

#[derive(Debug, Deserialize)]
pub struct InventoryAdjust {
    pub variant_id: Uuid,
    pub delta: i32,
    pub reason: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompatibilityPair {
    pub base_product_id: Uuid,
    pub accessory_product_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpsertProduct {
    pub handle: String,
    pub name: String,
    pub category: String,
    pub material: String,
    pub collection: Option<String>,
    pub craft_role: Option<String>,
    pub craft_base_type: Option<String>,
}
