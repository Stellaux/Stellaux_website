//! sea-orm entities. Hand-written for catalog tables (we control the schema).
//!
//! When commerce tables are migrated, generate their entities with:
//!
//! ```ignore
//! cargo install sea-orm-cli --version "^1.1"
//! sea-orm-cli generate entity \
//!     -u "$DATABASE_URL" \
//!     -o src/entity \
//!     --with-serde both \
//!     --serde-skip-deserializing-primary-key
//! ```
//!
//! Then re-export the new modules below.

pub mod craft_compatibility;
pub mod inventory_levels;
pub mod product_images;
pub mod product_variants;
pub mod products;

pub mod prelude {
    pub use super::craft_compatibility::Entity as CraftCompatibility;
    pub use super::inventory_levels::Entity as InventoryLevels;
    pub use super::product_images::Entity as ProductImages;
    pub use super::product_variants::Entity as ProductVariants;
    pub use super::products::Entity as Products;
}
