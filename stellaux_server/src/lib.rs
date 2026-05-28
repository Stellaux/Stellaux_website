//! stellaux_server — Axum + sea-orm backend for the Polished Standard storefront.
//!
//! Library crate: re-exports modules so `main.rs` and integration tests share the
//! same entry points.

pub mod common;
pub mod domain;
pub mod entity;
pub mod migration;
pub mod server;
