use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "inventory_levels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub variant_id: Uuid,
    pub on_hand: i32,
    pub reserved: i32,
    pub updated_at: DateTimeWithTimeZone,
}

impl Model {
    /// Units actually orderable right now.
    pub fn available(&self) -> i32 {
        (self.on_hand - self.reserved).max(0)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product_variants::Entity",
        from = "Column::VariantId",
        to = "super::product_variants::Column::Id",
        on_delete = "Cascade"
    )]
    Variant,
}

impl Related<super::product_variants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Variant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
