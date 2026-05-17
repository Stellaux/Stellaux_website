//! Domain modules. Each aggregate gets its own submodule that exposes
//! `pub fn routes() -> Router<AppState>` (mounted in `server::build_router`).
//!
//! These are scaffolds — handlers return stub JSON. Replace each handler's
//! body with real sea-orm queries / service calls as modules land.
//!
//! As a module grows, split it from `<name>.rs` into a directory:
//!
//! ```ignore
//! <name>/
//! ├── mod.rs
//! ├── model.rs    — sea-orm entities + DTOs
//! ├── repo.rs     — data access
//! ├── service.rs  — business logic
//! ├── handler.rs  — axum handlers
//! └── routes.rs   — pub fn routes()
//! ```

pub mod account;
pub mod admin;
pub mod auth;
pub mod cart;
pub mod catalog;
pub mod checkout;
pub mod craft;
pub mod webhooks;
