//! Initial catalog schema: products, variants, images, inventory, and craft
//! compatibility. Postgres 13+ is assumed (`gen_random_uuid()` is built in).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── products ─────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Products::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Products::Handle).text().not_null().unique_key())
                    .col(ColumnDef::new(Products::Name).text().not_null())
                    .col(ColumnDef::new(Products::Description).text())
                    .col(ColumnDef::new(Products::Collection).text())
                    .col(ColumnDef::new(Products::Category).text().not_null())
                    .col(ColumnDef::new(Products::Material).text().not_null())
                    .col(
                        ColumnDef::new(Products::Status)
                            .text()
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(Products::Popularity)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Products::Spec).text())
                    // craft_role: 'base' | 'accessory' | NULL
                    .col(ColumnDef::new(Products::CraftRole).text())
                    // craft_base_type: 'pendant' | 'chain' | 'trunk' | NULL
                    .col(ColumnDef::new(Products::CraftBaseType).text())
                    .col(
                        ColumnDef::new(Products::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Products::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_products_status")
                    .table(Products::Table)
                    .col(Products::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_products_craft_role")
                    .table(Products::Table)
                    .col(Products::CraftRole)
                    .to_owned(),
            )
            .await?;

        // ── product_variants ─────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(ProductVariants::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductVariants::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(ProductVariants::ProductId).uuid().not_null())
                    .col(
                        ColumnDef::new(ProductVariants::Sku)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ProductVariants::Size).text())
                    .col(ColumnDef::new(ProductVariants::PriceCents).integer().not_null())
                    .col(
                        ColumnDef::new(ProductVariants::WeightGrams)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(ProductVariants::DimensionsMm).json_binary())
                    .col(
                        ColumnDef::new(ProductVariants::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_variants_product")
                            .from(ProductVariants::Table, ProductVariants::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_product_variants_product")
                    .table(ProductVariants::Table)
                    .col(ProductVariants::ProductId)
                    .to_owned(),
            )
            .await?;

        // ── product_images ───────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(ProductImages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductImages::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(ProductImages::ProductId).uuid().not_null())
                    .col(ColumnDef::new(ProductImages::Url).text().not_null())
                    .col(ColumnDef::new(ProductImages::Alt).text())
                    .col(
                        ColumnDef::new(ProductImages::Position)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_images_product")
                            .from(ProductImages::Table, ProductImages::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ── inventory_levels ─────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(InventoryLevels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InventoryLevels::VariantId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InventoryLevels::OnHand)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryLevels::Reserved)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryLevels::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_inventory_levels_variant")
                            .from(InventoryLevels::Table, InventoryLevels::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ── inventory_adjustments ────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(InventoryAdjustments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InventoryAdjustments::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(InventoryAdjustments::VariantId).uuid().not_null())
                    .col(ColumnDef::new(InventoryAdjustments::Delta).integer().not_null())
                    .col(ColumnDef::new(InventoryAdjustments::Reason).text().not_null())
                    .col(ColumnDef::new(InventoryAdjustments::Channel).text())
                    .col(ColumnDef::new(InventoryAdjustments::ActorUserId).uuid())
                    .col(ColumnDef::new(InventoryAdjustments::Notes).text())
                    .col(
                        ColumnDef::new(InventoryAdjustments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_inventory_adjustments_variant")
                            .from(InventoryAdjustments::Table, InventoryAdjustments::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // ── craft_compatibility ──────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(CraftCompatibility::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CraftCompatibility::BaseProductId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CraftCompatibility::AccessoryProductId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(CraftCompatibility::BaseProductId)
                            .col(CraftCompatibility::AccessoryProductId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_craft_compat_base")
                            .from(CraftCompatibility::Table, CraftCompatibility::BaseProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_craft_compat_accessory")
                            .from(
                                CraftCompatibility::Table,
                                CraftCompatibility::AccessoryProductId,
                            )
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CraftCompatibility::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(InventoryAdjustments::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(InventoryLevels::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProductImages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProductVariants::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await?;
        Ok(())
    }
}

// ── Table identifiers ───────────────────────────────────────────────────────

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    Handle,
    Name,
    Description,
    Collection,
    Category,
    Material,
    Status,
    Popularity,
    Spec,
    CraftRole,
    CraftBaseType,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProductVariants {
    Table,
    Id,
    ProductId,
    Sku,
    Size,
    PriceCents,
    WeightGrams,
    DimensionsMm,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ProductImages {
    Table,
    Id,
    ProductId,
    Url,
    Alt,
    Position,
}

#[derive(DeriveIden)]
enum InventoryLevels {
    Table,
    VariantId,
    OnHand,
    Reserved,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum InventoryAdjustments {
    Table,
    Id,
    VariantId,
    Delta,
    Reason,
    Channel,
    ActorUserId,
    Notes,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CraftCompatibility {
    Table,
    BaseProductId,
    AccessoryProductId,
}
