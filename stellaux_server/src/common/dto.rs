//! Shared request/response shapes used across domain modules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

fn default_limit() -> u64 {
    20
}

impl Pagination {
    /// Returns `(limit, offset)` clamped to safe bounds.
    pub fn clamped(&self) -> (u64, u64) {
        (self.limit.clamp(1, 100), self.offset)
    }
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, limit: u64, offset: u64) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }
}
