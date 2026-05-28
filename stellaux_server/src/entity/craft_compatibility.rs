use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Composite-PK join table: which accessory product can be paired with which
/// base product.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "craft_compatibility")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub base_product_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub accessory_product_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
