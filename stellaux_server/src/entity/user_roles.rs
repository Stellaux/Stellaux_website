//! Authorization roles, keyed by the Supabase `auth.users.id`.
//!
//! Identity is owned by Supabase; this table records only *elevated* roles.
//! One row per user (the user id is the primary key). Absence of a row means
//! the user is a plain `customer` — see `common::roles::lookup`.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    /// 'customer' | 'support' | 'staff' | 'admin'
    pub role: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
