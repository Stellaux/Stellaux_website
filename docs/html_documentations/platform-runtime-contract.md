# Platform Runtime Contract

## Purpose

This document locks the runtime boundaries for Phase 0 so product, order, shipment, and customer-service work can proceed without frontend-led redesign.

## Runtime Components

| Component | Implementation | Responsibility | Authority level |
|-----------|----------------|----------------|-----------------|
| API server | Rust 2024 + Axum 0.8 + Tokio | business logic, authorization, idempotency, orchestration | internal authority |
| Database | Supabase Postgres 16 | operational data store | internal source of truth |
| Object storage | Supabase Storage for product images, behind the Rust storage contract | product images and related catalog display assets | internal source of truth for stored files |
| Auth verification | Supabase JWKS cache + server JWT support | caller identity verification | transport authority |
| Payments | Stripe | payment initiation and payment-event authority | external authority, mirrored internally |
| Shipping | Shippo | shipping rates, labels, tracking events | external authority, mirrored internally |
| Email | Resend | transactional email delivery | external send authority |

## Provider Dependency Rules

### Supabase

Supabase is a platform dependency in three distinct roles:

- Postgres host and operational database
- human authentication provider
- product-image storage provider
- optional Realtime transport for internal tooling

### Schema exposure

Supabase auto-exposes `public` schema tables through PostgREST. For this project:

- `public` does not imply anonymous readability
- every `public` business table must have explicit RLS posture
- internal operational tables belong in `private`
- the Rust API remains the primary business interface even when a table lives in `public`

### Stripe

Stripe is a bounded external dependency for payment workflows only.

- authoritative for payment-event truth
- never the owner of internal order state transitions on its own
- webhook events must be normalized into internal order/payment records

### Shippo

Shippo is a bounded external dependency for shipment quoting, label purchase, and tracking.

- authoritative for shipment-provider events
- never the owner of internal fulfillment workflow semantics on its own
- tracking and label outcomes must be mirrored into internal shipment records

### Resend

Resend is a delivery dependency, not a source of truth.

- send outcomes may be logged
- message history needed by support or operations should be mirrored internally if it matters to the business

## Boot Contract

`bootstrap::init()` is responsible for:

1. tracing initialization
2. config load and validation
3. database connect and ping
4. migration application
5. shared outbound HTTP client creation
6. storage backend initialization against the selected storage provider contract
7. optional Supabase JWKS cache initialization

The server must not begin serving requests before those steps complete.

## AppState Contract

`AppState` may hold only long-lived shared runtime dependencies:

- database connection pool
- immutable config
- shared outbound HTTP client
- storage backend
- optional JWKS cache

Request-derived state, per-provider auth claims, and temporary webhook payload state must not be stored on `AppState`.

## Configuration Contract

Configuration is grouped by integration boundary:

- `server`
- `database`
- `auth`
- `cors`
- `storage`
- `stripe`
- `shippo`
- `resend`
- `warehouse`

Missing configuration must follow one of three explicit behaviors:

- boot fail: required for the whole service to start
- route fail: service boots, but requests that need the integration fail safely
- feature disabled: service boots and the integration-backed feature is intentionally unavailable

Current expected posture:

- `DATABASE_URL`: boot fail
- product-image storage configuration: boot fail
- `SUPABASE_JWKS_URL`: route fail for routes requiring Supabase verification
- Stripe, Shippo, Resend secrets: route fail for their own feature paths until Phase 1-4 implement richer health signaling

## Connection Contract

The Rust server should connect to Supabase Postgres using:

- the direct Postgres connection on port `5432`, or
- a session-compatible pooler introduced deliberately later

Do not point the long-lived Rust API at transaction-mode pooling for SeaORM/sqlx workloads that rely on prepared statements.

## Operational Principle

Supabase Postgres is the internal operational truth even when external providers remain authoritative for their own event streams. The API server is responsible for normalizing external data into internal models rather than letting each consumer integrate providers independently.

For product imagery, Supabase Storage is the selected file store, while Postgres remains the owner of product/image metadata, references, and business-facing relationships.
