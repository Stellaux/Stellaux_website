use serde::{Deserialize, Serialize};
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
    #[serde(alias = "handle")]
    pub slug: String,
    #[serde(alias = "name")]
    pub title: String,
    pub description: Option<String>,
    #[serde(alias = "category")]
    pub category_slug: Option<String>,
    #[serde(alias = "material")]
    pub default_material: Option<String>,
    #[serde(alias = "collection")]
    pub collection_slug: Option<String>,
    #[serde(default)]
    pub collection_slugs: Vec<String>,
    pub status: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertProductVariant {
    pub material: String,
    pub design: Option<String>,
    pub type_label: Option<String>,
    pub size_value: Option<f64>,
    pub sku: String,
    pub price_cents: i64,
    pub cost_cents: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RefundOrderRequest {
    pub amount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct KpiQuery {
    pub range: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductSummary {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct ProductVariant {
    pub id: Uuid,
    pub sku: String,
    pub price_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct Product {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub variants: Vec<ProductVariant>,
}

#[derive(Debug, Serialize)]
pub struct Order {
    pub id: Uuid,
    pub number: String,
    pub status: String,
    pub channel: Option<String>,
    pub total_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct OrderKpis {
    pub revenue_cents: i64,
    pub previous_revenue_cents: i64,
    pub new_orders: u64,
}

#[derive(Debug, Serialize)]
pub struct CountResponse {
    pub count: u64,
}
