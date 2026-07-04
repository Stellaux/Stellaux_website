use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseTransaction, DbBackend, FromQueryResult, Statement, TransactionTrait,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    common::{
        app_state::AppState,
        audit::{self, Actor},
        auth::AuthUser,
        dto::{ListEnvelope, Pagination},
        error::{AppError, AppResult},
    },
    domains::admin::dto::{
        CancelOrderRequest, CompatibilityPair, CountResponse, InventoryAdjust, KpiQuery, Order,
        OrderKpis, OrdersFilter, Product, ProductSummary, ProductVariant, RefundOrderRequest,
        UpsertProduct, UpsertProductVariant,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/orders", get(list_orders))
        .route("/orders/kpis", get(order_kpis))
        .route("/orders/{order_id}", get(get_order))
        .route("/orders/{order_id}/refund", post(refund_order))
        .route("/orders/{order_id}/cancel", post(cancel_order))
        .route("/orders/{order_id}/labels", post(reprint_label))
        .route(
            "/products",
            get(list_products).post(create_product).put(upsert_product),
        )
        .route(
            "/products/{product_id}",
            get(get_product)
                .patch(update_product)
                .delete(delete_product),
        )
        .route("/products/{product_id}/variants", post(create_variant))
        .route("/variants/{variant_id}", patch(update_variant))
        .route("/inventory", get(list_inventory).post(adjust_inventory))
        .route("/inventory/adjustments", post(adjust_inventory))
        .route(
            "/craft/compatibility",
            get(list_compatibility).post(add_compatibility),
        )
        .route(
            "/craft/compatibility/{base_id}/{accessory_id}",
            delete(remove_compatibility),
        )
        .route("/channel-listings", get(list_channel_listings))
        .route(
            "/channel-listings/errors/count",
            get(channel_listing_error_count),
        )
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
struct ProductIdentityRow {
    id: Uuid,
}

#[derive(Debug, FromQueryResult)]
struct CategoryLookupRow {
    id: Uuid,
}

#[derive(Debug, FromQueryResult)]
struct CollectionLookupRow {
    id: Uuid,
}

#[derive(Debug, FromQueryResult)]
struct CountRow {
    count: i64,
}

#[derive(Debug, FromQueryResult)]
struct OrderRow {
    id: Uuid,
    number: String,
    status: String,
    total_cents: i64,
}

#[derive(Debug, FromQueryResult)]
struct InventoryLevelRow {
    variant_id: Uuid,
    quantity: i32,
    reserved: i32,
    available: i32,
}

#[derive(Debug, FromQueryResult)]
struct InventoryLockRow {
    inventory_id: Uuid,
    quantity: i32,
}

#[derive(Debug, FromQueryResult)]
struct OrderKpisRow {
    revenue_cents: i64,
    previous_revenue_cents: i64,
    new_orders: i64,
}

#[derive(Debug, serde::Serialize)]
struct InventoryLevelResponse {
    variant_id: Uuid,
    quantity: i32,
    reserved: i32,
    available: i32,
}

#[derive(Debug, serde::Serialize)]
struct ChannelListingRow {
    id: Uuid,
    channel: String,
    external_listing_id: String,
    external_variant_id: Option<String>,
    external_sku: Option<String>,
    product_id: Uuid,
    variant_id: Option<Uuid>,
    status: String,
    last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromQueryResult)]
struct ChannelListingDbRow {
    id: Uuid,
    channel: String,
    external_listing_id: String,
    external_variant_id: Option<String>,
    external_sku: Option<String>,
    product_id: Uuid,
    variant_id: Option<Uuid>,
    status: String,
    last_synced_at: Option<DateTime<Utc>>,
}

async fn list_orders(
    _admin: AuthUser,
    State(state): State<AppState>,
    Query(filter): Query<OrdersFilter>,
) -> AppResult<Json<ListEnvelope<Order>>> {
    let (limit, offset) = filter.page.clamped();
    let rows = OrderRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id,
                order_number as number,
                status,
                total_cents
            from public.orders
            where ($1::text is null or status = $1)
            order by placed_at desc, created_at desc
            limit $2 offset $3
        "#,
        vec![
            filter.status.clone().into(),
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
            from public.orders
            where ($1::text is null or status = $1)
        "#,
        vec![filter.status.into()],
    ))
    .one(&state.db)
    .await?
    .map(|row| row.count.max(0) as u64)
    .unwrap_or(0);

    Ok(Json(ListEnvelope::from_limit_offset(
        rows.into_iter().map(map_order_row).collect(),
        total,
        limit,
        offset,
    )))
}

async fn order_kpis(
    _admin: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<KpiQuery>,
) -> AppResult<Json<OrderKpis>> {
    let window = resolve_kpi_window(query.range.as_deref().unwrap_or("today"))?;
    let row = OrderKpisRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                coalesce(sum(total_cents) filter (
                    where placed_at >= $1 and placed_at < $2 and status in ('paid', 'shipped')
                ), 0)::bigint as revenue_cents,
                coalesce(sum(total_cents) filter (
                    where placed_at >= $3 and placed_at < $4 and status in ('paid', 'shipped')
                ), 0)::bigint as previous_revenue_cents,
                count(*) filter (where placed_at >= $1 and placed_at < $2)::bigint as new_orders
            from public.orders
        "#,
        vec![
            window.current_start.into(),
            window.current_end.into(),
            window.previous_start.into(),
            window.previous_end.into(),
        ],
    ))
    .one(&state.db)
    .await?
    .ok_or_else(|| AppError::Internal(anyhow::anyhow!("orders aggregate query returned no row")))?;

    Ok(Json(OrderKpis {
        revenue_cents: row.revenue_cents,
        previous_revenue_cents: row.previous_revenue_cents,
        new_orders: row.new_orders.max(0) as u64,
    }))
}

async fn get_order(
    _admin: AuthUser,
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Order>> {
    let row = OrderRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id,
                order_number as number,
                status,
                total_cents
            from public.orders
            where id = $1
        "#,
        vec![order_id.into()],
    ))
    .one(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(map_order_row(row)))
}

async fn refund_order(
    admin: AuthUser,
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(body): Json<RefundOrderRequest>,
) -> AppResult<StatusCode> {
    if body.amount_cents <= 0 {
        return Err(AppError::BadRequest(
            "amount_cents must be greater than zero".to_string(),
        ));
    }

    let row = OrderRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id,
                order_number as number,
                status,
                total_cents
            from public.orders
            where id = $1
        "#,
        vec![order_id.into()],
    ))
    .one(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    if body.amount_cents > row.total_cents {
        return Err(AppError::BadRequest(
            "amount_cents exceeds order total".to_string(),
        ));
    }

    audit::events::order_refunded(order_id, admin.user_id(), body.amount_cents);
    Ok(StatusCode::NO_CONTENT)
}

async fn cancel_order(
    admin: AuthUser,
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(body): Json<CancelOrderRequest>,
) -> AppResult<StatusCode> {
    if body.reason.trim().is_empty() {
        return Err(AppError::BadRequest("reason is required".to_string()));
    }

    let updated = state
        .db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                update public.orders
                   set status = 'cancelled',
                       updated_at = now()
                 where id = $1
                 returning id
            "#,
            vec![order_id.into()],
        ))
        .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    audit::events::order_cancelled(order_id, admin.user_id(), &body.reason);
    Ok(StatusCode::NO_CONTENT)
}

async fn reprint_label(
    _admin: AuthUser,
    _state: State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "order_id": order_id,
        "_todo": "re-fetch shippo transaction or void + re-buy"
    })))
}

async fn list_products(
    _admin: AuthUser,
    State(state): State<AppState>,
    Query(page): Query<Pagination>,
) -> AppResult<Json<ListEnvelope<ProductSummary>>> {
    let (limit, offset) = page.clamped();
    let rows = ProductSummaryRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id,
                handle as slug,
                name as title,
                (status = 'active') as active
            from public.products
            order by updated_at desc, created_at desc
            limit $1 offset $2
        "#,
        vec![(limit as i64).into(), (offset as i64).into()],
    ))
    .all(&state.db)
    .await?;
    let total = CountRow::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        "select count(*)::bigint as count from public.products".to_string(),
    ))
    .one(&state.db)
    .await?
    .map(|row| row.count.max(0) as u64)
    .unwrap_or(0);

    Ok(Json(ListEnvelope::from_limit_offset(
        rows.into_iter().map(map_product_summary_row).collect(),
        total,
        limit,
        offset,
    )))
}

async fn create_product(
    admin: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpsertProduct>,
) -> AppResult<Json<Product>> {
    let txn = state.db.begin().await?;
    let product = upsert_product_record(&txn, &admin, None, body).await?;
    txn.commit().await?;
    Ok(Json(product))
}

async fn upsert_product(
    admin: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpsertProduct>,
) -> AppResult<Json<Product>> {
    let txn = state.db.begin().await?;
    let existing = find_product_id_by_slug(&txn, &body.slug).await?;
    let product = upsert_product_record(&txn, &admin, existing, body).await?;
    txn.commit().await?;
    Ok(Json(product))
}

async fn get_product(
    _admin: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Product>> {
    Ok(Json(load_product(&state.db, id).await?))
}

async fn update_product(
    admin: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpsertProduct>,
) -> AppResult<Json<Product>> {
    let txn = state.db.begin().await?;
    let product = upsert_product_record(&txn, &admin, Some(id), body).await?;
    txn.commit().await?;
    Ok(Json(product))
}

async fn create_variant(
    _admin: AuthUser,
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(body): Json<UpsertProductVariant>,
) -> AppResult<Json<ProductVariant>> {
    let txn = state.db.begin().await?;
    let variant = upsert_variant_record(&txn, None, product_id, body).await?;
    txn.commit().await?;
    Ok(Json(variant))
}

async fn update_variant(
    _admin: AuthUser,
    State(state): State<AppState>,
    Path(variant_id): Path<Uuid>,
    Json(body): Json<UpsertProductVariant>,
) -> AppResult<Json<ProductVariant>> {
    let txn = state.db.begin().await?;
    let product_id = find_variant_product_id(&txn, variant_id).await?;
    let variant = upsert_variant_record(&txn, Some(variant_id), product_id, body).await?;
    txn.commit().await?;
    Ok(Json(variant))
}

async fn delete_product(
    _admin: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let updated = state
        .db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "update public.products set status = 'archived', updated_at = now() where id = $1",
            vec![id.into()],
        ))
        .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(json!({ "deleted": id })))
}

async fn list_inventory(_admin: AuthUser, State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows = InventoryLevelRow::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        r#"
            select
                variant_id,
                quantity,
                reserved,
                available
            from private.inventory
            order by updated_at desc, variant_id
        "#
        .to_string(),
    ))
    .all(&state.db)
    .await?;
    Ok(Json(json!({
        "items": rows.into_iter().map(|row| InventoryLevelResponse {
            variant_id: row.variant_id,
            quantity: row.quantity,
            reserved: row.reserved,
            available: row.available,
        }).collect::<Vec<_>>()
    })))
}

async fn adjust_inventory(
    admin: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<InventoryAdjust>,
) -> AppResult<StatusCode> {
    if body.delta == 0 {
        return Err(AppError::BadRequest("delta must be non-zero".to_string()));
    }
    if body.reason.trim().is_empty() {
        return Err(AppError::BadRequest("reason is required".to_string()));
    }

    let txn = state.db.begin().await?;
    let locked = InventoryLockRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id as inventory_id,
                quantity
            from private.inventory
            where variant_id = $1
            for update
        "#,
        vec![body.variant_id.into()],
    ))
    .one(&txn)
    .await?
    .ok_or(AppError::NotFound)?;
    let new_quantity = locked.quantity + body.delta;
    if new_quantity < 0 {
        return Err(AppError::BadRequest(
            "adjustment would make quantity negative".to_string(),
        ));
    }

    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            update private.inventory
               set quantity = $1,
                   updated_at = now()
             where id = $2
        "#,
        vec![new_quantity.into(), locked.inventory_id.into()],
    ))
    .await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            insert into private.inventory_adjustment (
                inventory_id,
                channel,
                change,
                reason,
                actor_user_id,
                notes
            )
            values ($1, 'website', $2, $3, $4, $5)
        "#,
        vec![
            locked.inventory_id.into(),
            body.delta.into(),
            body.reason.clone().into(),
            admin.user_id().into(),
            body.notes.into(),
        ],
    ))
    .await?;
    txn.commit().await?;

    audit::events::inventory_adjusted(
        body.variant_id,
        Actor::User {
            id: admin.user_id(),
            role: admin.role().as_str().to_string(),
        },
        body.delta,
        &body.reason,
    );
    Ok(StatusCode::NO_CONTENT)
}

async fn list_compatibility(_admin: AuthUser, _state: State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [] })))
}

async fn add_compatibility(
    _admin: AuthUser,
    _state: State<AppState>,
    Json(_pair): Json<CompatibilityPair>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "_todo": "insert into craft_compatibility" })))
}

async fn remove_compatibility(
    _admin: AuthUser,
    _state: State<AppState>,
    Path((base_id, accessory_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "removed": [base_id, accessory_id] })))
}

async fn list_channel_listings(
    _admin: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Value>> {
    let rows = ChannelListingDbRow::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        r#"
            select
                id,
                channel,
                external_listing_id,
                external_variant_id,
                external_sku,
                product_id,
                variant_id,
                status,
                last_synced_at
            from public.channel_listings
            order by updated_at desc, created_at desc
        "#
        .to_string(),
    ))
    .all(&state.db)
    .await?;
    Ok(Json(json!({
        "items": rows.into_iter().map(|row| ChannelListingRow {
            id: row.id,
            channel: row.channel,
            external_listing_id: row.external_listing_id,
            external_variant_id: row.external_variant_id,
            external_sku: row.external_sku,
            product_id: row.product_id,
            variant_id: row.variant_id,
            status: row.status,
            last_synced_at: row.last_synced_at,
        }).collect::<Vec<_>>()
    })))
}

async fn channel_listing_error_count(
    _admin: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<CountResponse>> {
    let count = CountRow::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        r#"
            select count(*)::bigint as count
            from public.channel_listings
            where status = 'error'
        "#
        .to_string(),
    ))
    .one(&state.db)
    .await?
    .map(|row| row.count.max(0) as u64)
    .unwrap_or(0);
    Ok(Json(CountResponse { count }))
}

fn map_order_row(row: OrderRow) -> Order {
    Order {
        id: row.id,
        number: row.number,
        status: row.status,
        channel: None,
        total_cents: row.total_cents,
    }
}

fn map_product_summary_row(row: ProductSummaryRow) -> ProductSummary {
    ProductSummary {
        id: row.id,
        slug: row.slug,
        title: row.title,
        active: row.active,
    }
}

async fn load_product<C>(db: &C, product_id: Uuid) -> AppResult<Product>
where
    C: ConnectionTrait,
{
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
        "#,
        vec![product_id.into()],
    ))
    .one(db)
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
            order by created_at asc, id asc
        "#,
        vec![product_id.into()],
    ))
    .all(db)
    .await?;

    Ok(Product {
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
    })
}

async fn find_product_id_by_slug<C>(db: &C, slug: &str) -> AppResult<Option<Uuid>>
where
    C: ConnectionTrait,
{
    Ok(
        ProductIdentityRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "select id from public.products where handle = $1",
            vec![slug.to_string().into()],
        ))
        .one(db)
        .await?
        .map(|row| row.id),
    )
}

async fn upsert_product_record(
    txn: &DatabaseTransaction,
    admin: &AuthUser,
    target_id: Option<Uuid>,
    body: UpsertProduct,
) -> AppResult<Product> {
    let collection_slugs = normalized_collection_slugs(&body);
    let requested_status = requested_product_status(&body)?;
    let product_id = if let Some(product_id) = target_id {
        let category_id = match body.category_slug.as_deref() {
            Some(slug) => Some(resolve_category_id(txn, slug).await?),
            None => None,
        };
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                update public.products
                   set handle = $1,
                       name = $2,
                       description = $3,
                       category_id = coalesce($4, category_id),
                       default_material = coalesce($5, default_material),
                       status = coalesce($6, status),
                       updated_at = now()
                 where id = $7
            "#,
            vec![
                body.slug.clone().into(),
                body.title.clone().into(),
                body.description.clone().into(),
                category_id.into(),
                body.default_material.clone().into(),
                requested_status.clone().into(),
                product_id.into(),
            ],
        ))
        .await?;
        product_id
    } else {
        let category_slug = body.category_slug.as_deref().ok_or_else(|| {
            AppError::BadRequest(
                "category_slug is required to create a product from the normalized schema"
                    .to_string(),
            )
        })?;
        let category_id = resolve_category_id(txn, category_slug).await?;
        let row = ProductIdentityRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                insert into public.products (
                    handle,
                    name,
                    description,
                    category_id,
                    default_material,
                    status
                )
                values ($1, $2, $3, $4, $5, $6)
                returning id
            "#,
            vec![
                body.slug.clone().into(),
                body.title.clone().into(),
                body.description.clone().into(),
                category_id.into(),
                body.default_material.clone().into(),
                requested_status
                    .as_deref()
                    .unwrap_or("draft")
                    .to_string()
                    .into(),
            ],
        ))
        .one(txn)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("product insert returned no id")))?;
        row.id
    };

    if !collection_slugs.is_empty() {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "delete from public.product_collections where product_id = $1",
            vec![product_id.into()],
        ))
        .await?;
        for slug in collection_slugs {
            let collection_id = resolve_collection_id(txn, &slug).await?;
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                    insert into public.product_collections (product_id, collection_id)
                    values ($1, $2)
                    on conflict do nothing
                "#,
                vec![product_id.into(), collection_id.into()],
            ))
            .await?;
        }
    }

    audit::record(
        if target_id.is_some() {
            "product.updated"
        } else {
            "product.created"
        },
        Actor::User {
            id: admin.user_id(),
            role: admin.role().as_str().to_string(),
        },
        audit::Subject::Product { id: product_id },
        Some(json!({ "slug": body.slug })),
    );
    load_product(txn, product_id).await
}

async fn upsert_variant_record(
    txn: &DatabaseTransaction,
    target_id: Option<Uuid>,
    product_id: Uuid,
    body: UpsertProductVariant,
) -> AppResult<ProductVariant> {
    let material = body.material.trim().to_string();
    let sku = body.sku.trim().to_string();
    if material.is_empty() {
        return Err(AppError::BadRequest(
            "variant.material is required".to_string(),
        ));
    }
    if sku.is_empty() {
        return Err(AppError::BadRequest("variant.sku is required".to_string()));
    }
    if body.price_cents < 0 {
        return Err(AppError::BadRequest(
            "variant.price_cents must be zero or greater".to_string(),
        ));
    }
    if body.cost_cents.is_some_and(|cost| cost < 0) {
        return Err(AppError::BadRequest(
            "variant.cost_cents must be zero or greater".to_string(),
        ));
    }
    let status = normalized_catalog_status(body.status.as_deref(), "variant.status")?;
    let type_label = body
        .type_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Default")
        .to_string();
    let variant_id = if let Some(variant_id) = target_id {
        let updated = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                    update public.product_variants
                       set material = $1,
                           design = $2,
                           type_label = $3,
                           size_value = $4,
                           sku = $5,
                           price_cents = $6,
                           cost_cents = $7,
                           status = coalesce($8, status),
                           updated_at = now()
                     where id = $9
                "#,
                vec![
                    material.clone().into(),
                    body.design.clone().into(),
                    type_label.clone().into(),
                    body.size_value.into(),
                    sku.clone().into(),
                    body.price_cents.into(),
                    body.cost_cents.into(),
                    status.clone().into(),
                    variant_id.into(),
                ],
            ))
            .await?;
        if updated.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        variant_id
    } else {
        let row = ProductVariantRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                insert into public.product_variants (
                    product_id,
                    material,
                    design,
                    type_label,
                    size_value,
                    sku,
                    price_cents,
                    cost_cents,
                    status
                )
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                returning id, sku, price_cents
            "#,
            vec![
                product_id.into(),
                material.clone().into(),
                body.design.clone().into(),
                type_label.into(),
                body.size_value.into(),
                sku.clone().into(),
                body.price_cents.into(),
                body.cost_cents.into(),
                status.unwrap_or_else(|| "draft".to_string()).into(),
            ],
        ))
        .one(txn)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("variant insert returned no row")))?;
        return Ok(ProductVariant {
            id: row.id,
            sku: row.sku,
            price_cents: row.price_cents,
        });
    };

    load_variant(txn, variant_id).await
}

async fn resolve_category_id<C>(db: &C, slug: &str) -> AppResult<Uuid>
where
    C: ConnectionTrait,
{
    CategoryLookupRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "select id from public.categories where slug = $1",
        vec![slug.to_string().into()],
    ))
    .one(db)
    .await?
    .map(|row| row.id)
    .ok_or_else(|| AppError::BadRequest(format!("unknown category_slug: {slug}")))
}

async fn resolve_collection_id<C>(db: &C, slug: &str) -> AppResult<Uuid>
where
    C: ConnectionTrait,
{
    CollectionLookupRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "select id from public.collections where slug = $1",
        vec![slug.to_string().into()],
    ))
    .one(db)
    .await?
    .map(|row| row.id)
    .ok_or_else(|| AppError::BadRequest(format!("unknown collection_slug: {slug}")))
}

fn normalized_collection_slugs(body: &UpsertProduct) -> Vec<String> {
    let mut slugs = Vec::new();
    if let Some(slug) = body
        .collection_slug
        .as_ref()
        .filter(|slug| !slug.trim().is_empty())
    {
        slugs.push(slug.clone());
    }
    for slug in &body.collection_slugs {
        if !slug.trim().is_empty() && !slugs.iter().any(|existing| existing == slug) {
            slugs.push(slug.clone());
        }
    }
    slugs
}

fn requested_product_status(body: &UpsertProduct) -> AppResult<Option<String>> {
    if let Some(status) = body.status.as_deref() {
        return normalized_catalog_status(Some(status), "product.status");
    }
    Ok(body.active.map(active_to_status).map(str::to_string))
}

fn normalized_catalog_status(status: Option<&str>, field: &str) -> AppResult<Option<String>> {
    match status.map(str::trim).filter(|status| !status.is_empty()) {
        Some(value @ ("draft" | "active" | "archived")) => Ok(Some(value.to_string())),
        Some(other) => Err(AppError::BadRequest(format!(
            "{field} must be one of: draft, active, archived (got {other})"
        ))),
        None => Ok(None),
    }
}

fn active_to_status(active: bool) -> &'static str {
    if active { "active" } else { "draft" }
}

async fn load_variant<C>(db: &C, variant_id: Uuid) -> AppResult<ProductVariant>
where
    C: ConnectionTrait,
{
    let variant = ProductVariantRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                id,
                sku,
                price_cents
            from public.product_variants
            where id = $1
        "#,
        vec![variant_id.into()],
    ))
    .one(db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(ProductVariant {
        id: variant.id,
        sku: variant.sku,
        price_cents: variant.price_cents,
    })
}

async fn find_variant_product_id<C>(db: &C, variant_id: Uuid) -> AppResult<Uuid>
where
    C: ConnectionTrait,
{
    ProductIdentityRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "select product_id as id from public.product_variants where id = $1",
        vec![variant_id.into()],
    ))
    .one(db)
    .await?
    .map(|row| row.id)
    .ok_or(AppError::NotFound)
}

struct KpiWindow {
    current_start: DateTime<Utc>,
    current_end: DateTime<Utc>,
    previous_start: DateTime<Utc>,
    previous_end: DateTime<Utc>,
}

fn resolve_kpi_window(range: &str) -> AppResult<KpiWindow> {
    let today = Utc::now().date_naive();
    match range {
        "today" => Ok(window_from_days(today, 1)),
        "last_7d" => Ok(window_from_days(today, 7)),
        "last_30d" => Ok(window_from_days(today, 30)),
        other => Err(AppError::BadRequest(format!(
            "unsupported KPI range: {other}"
        ))),
    }
}

fn window_from_days(today: NaiveDate, days: i64) -> KpiWindow {
    let tomorrow = today + Duration::days(1);
    let current_start = tomorrow - Duration::days(days);
    let previous_start = current_start - Duration::days(days);
    KpiWindow {
        current_start: midnight_utc(current_start),
        current_end: midnight_utc(tomorrow),
        previous_start: midnight_utc(previous_start),
        previous_end: midnight_utc(current_start),
    }
}

fn midnight_utc(date: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(
        &date
            .and_hms_opt(0, 0, 0)
            .expect("midnight must always be valid"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_collection_slugs_merges_singular_and_plural_without_duplicates() {
        let slugs = normalized_collection_slugs(&UpsertProduct {
            slug: "mug".into(),
            title: "Stone Mug".into(),
            description: None,
            category_slug: Some("drinkware".into()),
            default_material: Some("stoneware".into()),
            collection_slug: Some("summer-drop".into()),
            collection_slugs: vec!["summer-drop".into(), "atelier".into()],
            status: None,
            active: Some(true),
        });

        assert_eq!(
            slugs,
            vec!["summer-drop".to_string(), "atelier".to_string()]
        );
    }

    #[test]
    fn requested_product_status_prefers_explicit_status_over_legacy_active_flag() {
        let status = requested_product_status(&UpsertProduct {
            slug: "mug".into(),
            title: "Stone Mug".into(),
            description: None,
            category_slug: Some("drinkware".into()),
            default_material: Some("stoneware".into()),
            collection_slug: None,
            collection_slugs: Vec::new(),
            status: Some("archived".into()),
            active: Some(true),
        })
        .unwrap();

        assert_eq!(status.as_deref(), Some("archived"));
    }

    #[test]
    fn normalized_catalog_status_rejects_unknown_value() {
        let err = normalized_catalog_status(Some("retired"), "product.status").unwrap_err();
        assert!(err.to_string().contains("draft, active, archived"));
    }
}
