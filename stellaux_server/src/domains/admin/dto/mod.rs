use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

/// Admin orders-table row. **Unified with `internal_api::shared::domain::OrderListItem`** — keep the
/// field set (names + types) identical on both sides so the ICP can deserialize this verbatim.
/// `channel` has no column yet (always `None`); everything else is DB truth.
#[derive(Debug, Serialize)]
pub struct OrderListItem {
    pub id: Uuid,
    pub number: String,
    pub status: String,
    pub customer: Option<String>,
    pub email: Option<String>,
    pub channel: Option<String>,
    pub total_cents: i64,
    pub item_count: i64,
    pub placed_at: Option<DateTime<Utc>>,
    pub ship_city: Option<String>,
    pub ship_country: Option<String>,
}

/// One line of an order (`public.order_items`). Unified with the ICP `OrderLineItem`.
#[derive(Debug, Serialize)]
pub struct OrderLineItem {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub total_cents: i64,
}

/// Full order detail (list fields + money breakdown, addresses, timestamps, line items).
/// **Unified with `internal_api::shared::domain::OrderDetail`.** Superset of `OrderListItem`, so a
/// consumer expecting the list shape can read a detail payload unchanged.
#[derive(Debug, Serialize)]
pub struct OrderDetail {
    pub id: Uuid,
    pub number: String,
    pub status: String,
    pub customer: Option<String>,
    pub email: Option<String>,
    pub channel: Option<String>,
    pub total_cents: i64,
    pub item_count: i64,
    pub placed_at: Option<DateTime<Utc>>,
    pub ship_city: Option<String>,
    pub ship_country: Option<String>,
    pub subtotal_cents: i64,
    pub tax_cents: i64,
    pub shipping_cents: i64,
    pub currency: String,
    pub user_id: Option<Uuid>,
    pub paid_at: Option<DateTime<Utc>>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub shipping_address: Value,
    pub billing_address: Value,
    pub line_items: Vec<OrderLineItem>,
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
