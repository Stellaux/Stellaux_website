use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub handle: String,
    pub name: String,
    pub description: Option<String>,
    pub collection: Option<String>,
    pub category: String,
    pub material: String,
    pub status: String,
    pub popularity: i32,
    pub spec: Option<String>,
    pub craft_role: Option<String>,
    pub craft_base_type: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product_variants::Entity")]
    Variants,
    #[sea_orm(has_many = "super::product_images::Entity")]
    Images,
}

impl Related<super::product_variants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Variants.def()
    }
}

impl Related<super::product_images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Images.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
