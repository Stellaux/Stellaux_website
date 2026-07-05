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
//! `images/products/meridien-signet/main.jpg`).
//!
//! Serializing a key for a client depends on whether the object is public:
//!   - Public objects (catalog stock photography behind a CDN) → `public_url`.
//!   - Private objects (`order-media`: photos of a customer's own piece) →
//!     `signed_url`, which mints a short-lived presigned GET so the bucket can
//!     stay closed. On the `local` backend there is nothing to sign, so
//!     `signed_url` falls back to the `public_url` served by the `ServeDir`
//!     mount (dev/test only).

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use bytes::Bytes;
use object_store::{
    ObjectStore,
    aws::{AmazonS3, AmazonS3Builder},
    local::LocalFileSystem,
    path::Path as ObjectPath,
    signer::Signer,
};
use reqwest::Method;

use crate::common::{
    config::{StorageBackend, StorageConfig},
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct Storage {
    inner: Arc<dyn ObjectStore>,
    /// Concrete S3 handle kept alongside `inner` because presigning lives on the
    /// `Signer` trait, which is not object-safe on `dyn ObjectStore`. `None` for
    /// the local backend (nothing to presign).
    signer: Option<Arc<AmazonS3>>,
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

    /// Construct the externally-visible URL for a *public* object key.
    pub fn public_url(&self, key: &str) -> String {
        format!(
            "{}/{}",
            self.public_base_url.trim_end_matches('/'),
            key.trim_start_matches('/')
        )
    }

    /// Mint a short-lived presigned GET URL for a *private* object key.
    ///
    /// Use this for anything in a closed bucket (e.g. `order-media`) after the
    /// caller has passed its ownership guard: the returned URL grants read access
    /// to this one object for `expires_in`, so it can be handed to a browser
    /// without exposing bucket credentials.
    ///
    /// On the local backend there are no credentials to sign with; the object is
    /// already reachable via the `ServeDir` mount, so this returns `public_url`.
    /// That path is for dev/test only — do not rely on it to keep anything private.
    pub async fn signed_url(&self, key: &str, expires_in: Duration) -> AppResult<String> {
        match &self.signer {
            Some(s3) => {
                let path = ObjectPath::from(key);
                let url = s3
                    .signed_url(Method::GET, &path, expires_in)
                    .await
                    .map_err(|e| {
                        AppError::Internal(anyhow::anyhow!("signing url for {key}: {e}"))
                    })?;
                Ok(url.to_string())
            }
            None => Ok(self.public_url(key)),
        }
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
                signer: None,
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

            // Custom endpoint = R2 / MinIO / Backblaze / Supabase Storage. AWS S3
            // leaves this unset. These all require path-style addressing
            // (`{endpoint}/{bucket}/{key}`) rather than virtual-hosted
            // (`{bucket}.{endpoint}/...`) — Supabase Storage only supports path
            // style. object_store already defaults to path style, but pin it so a
            // future default flip can't silently break these backends.
            if let Some(endpoint) = &config.s3_endpoint {
                builder = builder
                    .with_endpoint(endpoint)
                    .with_virtual_hosted_style_request(false)
                    .with_allow_http(endpoint.starts_with("http://"));
            }

            // Keep the concrete `AmazonS3` in an Arc so it backs both the generic
            // `ObjectStore` API (`inner`) and presigning (`signer`).
            let store = Arc::new(builder.build().context("building S3 client")?);
            Ok(Storage {
                inner: store.clone(),
                signer: Some(store),
                public_base_url: config.public_base_url.clone(),
                backend: StorageBackend::S3,
            })
        }
    }
}
