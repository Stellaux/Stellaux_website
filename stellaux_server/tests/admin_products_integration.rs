//! Env-gated integration test for the internal-admin product create flow.
//!
//! Skips unless `TEST_DATABASE_URL` (or `DATABASE_URL`) is set, so normal local
//! `cargo test` remains green without a live Postgres instance.

use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    middleware,
};
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, FromQueryResult, Statement,
};
use stellaux_server::{
    common::{
        app_state::AppState,
        auth::require_internal_admin,
        config::{
            AuthConfig, Config, CorsConfig, DatabaseConfig, ResendConfig, ServerConfig,
            ShippoConfig, StorageBackend, StorageConfig, StripeConfig, WarehouseConfig,
        },
        storage,
    },
    domains::admin::routes,
};
use tower::ServiceExt;
use uuid::Uuid;

const INTERNAL_ADMIN_TOKEN: &str = "svc-token";
const ORIGINAL_USER: &str = "dddddddd-dddd-dddd-dddd-dddddddddddd";

#[derive(Debug, FromQueryResult)]
struct ProductRow {
    slug: String,
    title: String,
    category_slug: String,
    default_material: Option<String>,
    status: String,
    collection_count: i64,
}

fn test_database_url() -> Option<String> {
    std::env::var("TEST_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .filter(|s| !s.trim().is_empty())
}

macro_rules! db_url_or_skip {
    () => {
        match test_database_url() {
            Some(url) => url,
            None => {
                eprintln!("skipping: set TEST_DATABASE_URL to run admin product integration tests");
                return;
            }
        }
    };
}

async fn setup_db(url: &str) -> DatabaseConnection {
    let db = Database::connect(url).await.expect("connect postgres");
    db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .expect("enable pgcrypto");
    db.execute_unprepared(
        r#"
        DROP TABLE IF EXISTS public.product_media CASCADE;
        DROP TABLE IF EXISTS public.product_variants CASCADE;
        DROP TABLE IF EXISTS public.product_collections CASCADE;
        DROP TABLE IF EXISTS public.products CASCADE;
        DROP TABLE IF EXISTS public.collections CASCADE;
        DROP TABLE IF EXISTS public.category_size_options CASCADE;
        DROP TABLE IF EXISTS public.categories CASCADE;
        "#,
    )
    .await
    .expect("drop catalog tables");
    db.execute_unprepared(include_str!(
        "../../supabase/migrations/20260610000200_catalog.sql"
    ))
    .await
    .expect("apply catalog migration");
    db
}

fn build_state(db: DatabaseConnection) -> AppState {
    let local_path = std::env::temp_dir()
        .join(format!("stellaux-admin-products-it-{}", Uuid::new_v4()))
        .display()
        .to_string();
    let storage = storage::build(&StorageConfig {
        backend: StorageBackend::Local,
        public_base_url: String::new(),
        local_path: local_path.clone(),
        s3_bucket: None,
        s3_endpoint: None,
        s3_region: String::new(),
        s3_access_key_id: None,
        s3_secret_access_key: None,
    })
    .expect("build local storage");

    AppState {
        db,
        config: Arc::new(Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 8080,
                environment: "test".into(),
                request_timeout_secs: 30,
                body_limit_bytes: 1024 * 1024,
                webhook_body_limit_bytes: 1024 * 1024,
            },
            database: DatabaseConfig {
                url: "postgres://example".into(),
                pool_size: 1,
            },
            auth: AuthConfig {
                jwt_secret: "secret".into(),
                jwt_expiry_seconds: 3600,
                issuer: "issuer".into(),
                audience: "audience".into(),
                internal_admin_token: Some(INTERNAL_ADMIN_TOKEN.into()),
                supabase_jwks_url: None,
                supabase_audience: "authenticated".into(),
                supabase_issuer: None,
                jwks_ttl_seconds: 3600,
            },
            cors: CorsConfig { origins: vec![] },
            storage: StorageConfig {
                backend: StorageBackend::Local,
                public_base_url: String::new(),
                local_path,
                s3_bucket: None,
                s3_endpoint: None,
                s3_region: String::new(),
                s3_access_key_id: None,
                s3_secret_access_key: None,
            },
            stripe: StripeConfig::default(),
            shippo: ShippoConfig::default(),
            resend: ResendConfig::default(),
            warehouse: WarehouseConfig::default(),
        }),
        http: reqwest::Client::new(),
        storage,
        jwks: None,
    }
}

fn build_app(state: AppState) -> Router {
    Router::new()
        .nest("/admin", routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_internal_admin,
        ))
        .with_state(state)
}

#[tokio::test]
async fn internal_admin_put_products_creates_normalized_product() {
    let url = db_url_or_skip!();
    let db = setup_db(&url).await;

    let category_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            insert into public.categories (id, slug, name, size_unit, sort_order)
            values ($1, $2, $3, 'none', 0)
        "#,
        vec![category_id.into(), "drinkware".into(), "Drinkware".into()],
    ))
    .await
    .expect("insert category");
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            insert into public.collections (id, slug, name)
            values ($1, $2, $3)
        "#,
        vec![
            collection_id.into(),
            "summer-drop".into(),
            "Summer Drop".into(),
        ],
    ))
    .await
    .expect("insert collection");

    let app = build_app(build_state(db.clone()));
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/admin/products")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {INTERNAL_ADMIN_TOKEN}"),
                )
                .header("X-Original-User", ORIGINAL_USER)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "slug": "mug",
                        "title": "Stone Mug",
                        "description": "hand-thrown",
                        "category_slug": "drinkware",
                        "default_material": "stoneware",
                        "collection_slugs": ["summer-drop"],
                        "active": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["slug"], "mug");
    assert_eq!(created["title"], "Stone Mug");
    assert_eq!(created["description"], "hand-thrown");
    assert_eq!(created["variants"], serde_json::json!([]));

    let stored = ProductRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            select
                p.handle as slug,
                p.name as title,
                c.slug as category_slug,
                p.default_material,
                p.status,
                (
                    select count(*)::bigint
                    from public.product_collections pc
                    where pc.product_id = p.id
                ) as collection_count
            from public.products p
            join public.categories c on c.id = p.category_id
            where p.handle = $1
        "#,
        vec!["mug".into()],
    ))
    .one(&db)
    .await
    .expect("query stored product")
    .expect("created product row");

    assert_eq!(stored.slug, "mug");
    assert_eq!(stored.title, "Stone Mug");
    assert_eq!(stored.category_slug, "drinkware");
    assert_eq!(stored.default_material.as_deref(), Some("stoneware"));
    assert_eq!(stored.status, "active");
    assert_eq!(stored.collection_count, 1);
}
