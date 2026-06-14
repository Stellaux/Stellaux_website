//! Authorization roles, keyed by the Supabase user id (`auth.users.id`).
//!
//! Identity stays in Supabase; this table records only elevated roles
//! (admin / staff / support). We deliberately do *not* add a cross-schema
//! foreign key to `auth.users` — that schema is Supabase-managed and may not
//! exist in local or CI databases. Referential integrity (the user id is real)
//! is established at the auth layer, which only writes the `sub` from a verified
//! Supabase token.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserRoles::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    // 'customer' | 'support' | 'staff' | 'admin'
                    .col(ColumnDef::new(UserRoles::Role).text().not_null())
                    .col(
                        ColumnDef::new(UserRoles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserRoles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserRoles {
    Table,
    UserId,
    Role,
    CreatedAt,
    UpdatedAt,
}
