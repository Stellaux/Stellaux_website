//! Embedded migrations. Run automatically at boot via `Migrator::up(&db, None)`
//! in `bootstrap::init`.
//!
//! Adding a new migration:
//!   1. Create `src/migration/m<YYYYMMDD>_<NNNNNN>_<slug>.rs`.
//!   2. `mod` it below.
//!   3. Append `Box::new(...)` to the `migrations()` vec.
//!
//! The vec order *is* the apply order; sea-orm-migration tracks state in the
//! `seaql_migrations` table so each migration runs exactly once per DB.

pub use sea_orm_migration::prelude::*;

mod m20260516_000001_initial_catalog;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260516_000001_initial_catalog::Migration)]
    }
}
