//! Storage abstraction over `object_store`. Same API for local filesystem and
//! S3-compatible backends (R2, S3, MinIO, B2).
//!
//! Choose backend via `STORAGE_BACKEND` env var:
//!   - `local` → files under `STORAGE_LOCAL_PATH`, served by this server at
//!     `STORAGE_PUBLIC_URL` via a `ServeDir` mount.
//!   - `s3` / `r2` / `minio` → bucket at `STORAGE_S3_ENDPOINT`, served directly
//!     from `STORAGE_PUBLIC_URL` (CDN/edge in front of the bucket).
//!
//! Stored object keys are environment-agnostic — only the prefix differs
//! across deploys. Always store the *key* in the DB (e.g.,
//! `images/products/meridien-signet/main.jpg`) and call `Storage::public_url`
//! when serializing it for clients.

use std::sync::Arc;

use anyhow::Context;
use bytes::Bytes;
use object_store::{
    ObjectStore, aws::AmazonS3Builder, local::LocalFileSystem, path::Path as ObjectPath,
};

use crate::common::{
    config::{StorageBackend, StorageConfig},
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct Storage {
    inner: Arc<dyn ObjectStore>,
    public_base_url: String,
    backend: StorageBackend,
}

impl Storage {
    /// Upload an object. Overwrites if key already exists.
    pub async fn put(&self, key: &str, bytes: Bytes) -> AppResult<()> {
        let path = ObjectPath::from(key);
        self.inner
            .put(&path, bytes.into())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("storage put({key}): {e}")))?;
        Ok(())
    }

    /// Fetch an object's bytes. Returns `AppError::NotFound` if missing.
    pub async fn get(&self, key: &str) -> AppResult<Bytes> {
        let path = ObjectPath::from(key);
        let result = self.inner.get(&path).await.map_err(|e| match e {
            object_store::Error::NotFound { .. } => AppError::NotFound,
            other => AppError::Internal(anyhow::anyhow!("storage get({key}): {other}")),
        })?;
        result
            .bytes()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("storage read({key}): {e}")))
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let path = ObjectPath::from(key);
        self.inner
            .delete(&path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("storage delete({key}): {e}")))?;
        Ok(())
    }

    /// Construct the externally-visible URL for an object key.
    pub fn public_url(&self, key: &str) -> String {
        format!(
            "{}/{}",
            self.public_base_url.trim_end_matches('/'),
            key.trim_start_matches('/')
        )
    }

    pub fn backend(&self) -> StorageBackend {
        self.backend
    }
}

/// Build the configured backend. Called once at boot.
pub fn build(config: &StorageConfig) -> anyhow::Result<Storage> {
    match config.backend {
        StorageBackend::Local => {
            std::fs::create_dir_all(&config.local_path)
                .with_context(|| format!("creating local storage dir {}", config.local_path))?;
            let store = LocalFileSystem::new_with_prefix(&config.local_path)
                .context("opening local filesystem backend")?;
            Ok(Storage {
                inner: Arc::new(store),
                public_base_url: config.public_base_url.clone(),
                backend: StorageBackend::Local,
            })
        }
        StorageBackend::S3 => {
            let bucket = config
                .s3_bucket
                .as_ref()
                .context("STORAGE_S3_BUCKET required when STORAGE_BACKEND=s3")?;
            let access_key = config
                .s3_access_key_id
                .as_ref()
                .context("STORAGE_S3_ACCESS_KEY_ID required when STORAGE_BACKEND=s3")?;
            let secret_key = config
                .s3_secret_access_key
                .as_ref()
                .context("STORAGE_S3_SECRET_ACCESS_KEY required when STORAGE_BACKEND=s3")?;

            let mut builder = AmazonS3Builder::new()
                .with_bucket_name(bucket)
                .with_region(&config.s3_region)
                .with_access_key_id(access_key)
                .with_secret_access_key(secret_key);

            // Custom endpoint = R2 / MinIO / Backblaze. AWS S3 leaves this unset.
            if let Some(endpoint) = &config.s3_endpoint {
                builder = builder
                    .with_endpoint(endpoint)
                    .with_allow_http(endpoint.starts_with("http://"));
            }

            let store = builder.build().context("building S3 client")?;
            Ok(Storage {
                inner: Arc::new(store),
                public_base_url: config.public_base_url.clone(),
                backend: StorageBackend::S3,
            })
        }
    }
}
