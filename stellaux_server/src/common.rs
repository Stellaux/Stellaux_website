//! Cross-cutting concerns: config, logging, errors, auth helpers, shared state.

pub mod app_state;
pub mod audit;
pub mod auth;
pub mod bootstrap;
pub mod config;
pub mod dto;
pub mod error;
pub mod hash_util;
pub mod jwt;
pub mod metrics;
pub mod multipart_helper;
pub mod opentelemetry;
pub mod storage;
pub mod ts_format;
