//! Per-request authorization-role resolution.
//!
//! Identity is owned by Supabase (`auth.users`); *authorization* lives in our
//! own `user_roles` table. The auth middleware calls [`lookup`] after verifying
//! a Supabase token to stamp the real role onto the request's `Claims`.
//!
//! A user with no `user_roles` row resolves to [`Role::Customer`] — the default
//! is least privilege, so granting access is always an explicit DB write and a
//! missing row can never escalate.

use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

use crate::common::{error::AppResult, jwt::Role};
use crate::entity::prelude::UserRoles;

/// Resolve `user_id`'s authorization role. Absent row → [`Role::Customer`].
pub async fn lookup(db: &DatabaseConnection, user_id: Uuid) -> AppResult<Role> {
    let row = UserRoles::find_by_id(user_id).one(db).await?;
    Ok(row
        .map(|m| Role::from_db_str(&m.role))
        .unwrap_or(Role::Customer))
}
