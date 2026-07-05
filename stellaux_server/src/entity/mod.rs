//! sea-orm entities. Only `user_roles` remains — it backs the authz role lookup in
//! `common::roles` (`UserRoles::find_by_id`), and its shape matches the authoritative schema
//! (`user_roles` in the Supabase migrations / `docs/migrations/db_schema.md`).
//!
//! Catalog/commerce tables are NOT modelled here: the schema is owned by the Supabase SQL
//! migrations (the single source of truth), and the server reads those tables via raw SQL in
//! `domains/*/api/routes.rs`. The old hand-written catalog entities + the sea-orm migrator were
//! retired when Supabase migrations replaced the abandoned sea-orm migration plan.

pub mod user_roles;

pub mod prelude {
    pub use super::user_roles::Entity as UserRoles;
}
