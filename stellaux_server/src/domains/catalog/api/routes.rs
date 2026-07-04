use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sea_orm::{DbBackend, FromQueryResult, Statement};
use uuid::Uuid;

use crate::{
    common::{
        app_state::AppState,
        dto::ListEnvelope,
        error::{AppError, AppResult},
    },
    domains::{
        admin::dto::{Product, ProductSummary, ProductVariant},
        catalog::dto::ProductFilter,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(list_products))
        .route("/products/{product_id}", get(get_product))
        .route("/collections", get(list_collections))
        .route("/categories", get(list_categories))
}

#[derive(Debug, FromQueryResult)]
struct ProductSummaryRow {
    id: Uuid,
    slug: String,
    title: String,
    active: bool,
}

#[derive(Debug, FromQueryResult)]
struct ProductHeaderRow {
    id: Uuid,
    slug: String,
    title: String,
    description: Option<String>,
}

#[derive(Debug, FromQueryResult)]
struct ProductVariantRow {
    id: Uuid,
    sku: String,
    price_cents: i64,
}

#[derive(Debug, FromQueryResult)]
struct CountRow {
    count: i64,
}

#[derive(Debug, serde::Serialize)]
struct CollectionSummary {
    slug: String,
    name: String,
}

#[derive(Debug, serde::Serialize)]
struct CategorySummary {
    slug: String,
    name: String,
    size_unit: String,
}

async fn list_products(
    State(state): State<AppState>,
    Query(filter): Query<ProductFilter>,
) -> AppResult<Json<ListEnvelope<ProductSummary>>> {
    let (limit, offset) = filter.page.clamped();
    let rows = ProductSummaryRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                p.id,
                p.handle as slug,
                p.name as title,
                true as active
            from public.products p
            join public.categories c on c.id = p.category_id
            where p.status = 'active'
              and ($1::text is null or c.slug = $1)
              and (
                    $2::text is null
                    or p.default_material = $2
                    or exists (
                        select 1
                        from public.product_variants v
                        where v.product_id = p.id
                          and v.status = 'active'
                          and v.material = $2
                    )
              )
              and (
                    $3::text is null
                    or exists (
                        select 1
                        from public.product_collections pc
                        join public.collections col on col.id = pc.collection_id
                        where pc.product_id = p.id
                          and col.slug = $3
                    )
              )
            order by p.updated_at desc, p.created_at desc
            limit $4 offset $5
        "#,
        vec![
            normalized_filter(filter.category.as_deref()).into(),
            normalized_filter(filter.material.as_deref()).into(),
            normalized_filter(filter.collection.as_deref()).into(),
            (limit as i64).into(),
            (offset as i64).into(),
        ],
    ))
    .all(&state.db)
    .await?;
    let total = CountRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select count(*)::bigint as count
            from public.products p
            join public.categories c on c.id = p.category_id
            where p.status = 'active'
              and ($1::text is null or c.slug = $1)
              and (
                    $2::text is null
                    or p.default_material = $2
                    or exists (
                        select 1
                        from public.product_variants v
                        where v.product_id = p.id
                          and v.status = 'active'
                          and v.material = $2
                    )
              )
              and (
                    $3::text is null
                    or exists (
                        select 1
                        from public.product_collections pc
                        join public.collections col on col.id = pc.collection_id
                        where pc.product_id = p.id
                          and col.slug = $3
                    )
              )
        "#,
        vec![
            normalized_filter(filter.category.as_deref()).into(),
            normalized_filter(filter.material.as_deref()).into(),
            normalized_filter(filter.collection.as_deref()).into(),
        ],
    ))
    .one(&state.db)
    .await?
    .map(|row| row.count.max(0) as u64)
    .unwrap_or(0);

    Ok(Json(ListEnvelope::from_limit_offset(
        rows.into_iter()
            .map(|row| ProductSummary {
                id: row.id,
                slug: row.slug,
                title: row.title,
                active: row.active,
            })
            .collect(),
        total,
        limit,
        offset,
    )))
}

async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> AppResult<Json<Product>> {
    let header = ProductHeaderRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id,
                handle as slug,
                name as title,
                description
            from public.products
            where id = $1
              and status = 'active'
        "#,
        vec![product_id.into()],
    ))
    .one(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    let variants = ProductVariantRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id,
                sku,
                price_cents
            from public.product_variants
            where product_id = $1
              and status = 'active'
            order by created_at asc, id asc
        "#,
        vec![product_id.into()],
    ))
    .all(&state.db)
    .await?;

    Ok(Json(Product {
        id: header.id,
        slug: header.slug,
        title: header.title,
        description: header.description,
        variants: variants
            .into_iter()
            .map(|variant| ProductVariant {
                id: variant.id,
                sku: variant.sku,
                price_cents: variant.price_cents,
            })
            .collect(),
    }))
}

async fn list_collections(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<CollectionSummary>>> {
    let rows = CollectionSummaryRow::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        r#"
            select slug, name
            from public.collections
            order by name asc
        "#
        .to_string(),
    ))
    .all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| CollectionSummary {
                slug: row.slug,
                name: row.name,
            })
            .collect(),
    ))
}

async fn list_categories(State(state): State<AppState>) -> AppResult<Json<Vec<CategorySummary>>> {
    let rows = CategorySummaryRow::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        r#"
            select slug, name, size_unit
            from public.categories
            order by sort_order asc, name asc
        "#
        .to_string(),
    ))
    .all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| CategorySummary {
                slug: row.slug,
                name: row.name,
                size_unit: row.size_unit,
            })
            .collect(),
    ))
}

#[derive(Debug, FromQueryResult)]
struct CollectionSummaryRow {
    slug: String,
    name: String,
}

#[derive(Debug, FromQueryResult)]
struct CategorySummaryRow {
    slug: String,
    name: String,
    size_unit: String,
}

fn normalized_filter(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
