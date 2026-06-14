use serde::Deserialize;

use crate::common::dto::Pagination;

#[derive(Debug, Deserialize)]
pub struct ProductFilter {
    pub category: Option<String>,
    pub material: Option<String>,
    pub collection: Option<String>,
    pub sort: Option<String>,
    #[serde(flatten)]
    pub page: Pagination,
}
