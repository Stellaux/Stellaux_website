# Integration Coverage Plan

## Goal

Grow `stellaux_server` integration coverage from a single env-gated admin flow into a reliable release gate that exercises:

- boot-time migrations
- auth boundaries
- database writes and reads
- webhook idempotency
- external-provider integrations through mocks
- readiness behavior that mirrors deployment

The existing GitHub Actions Postgres harness is already the right execution environment. The next step is to make the tests consistently use it and expand what they prove.

## Current Baseline

- CI provisions PostgreSQL 16 and runs `cargo test --tests -- --nocapture`.
- `tests/admin_products_integration.rs` exists, but it skips unless `TEST_DATABASE_URL` or `DATABASE_URL` is set.
- The test applies a SQL file directly instead of booting the same migration path the server uses at runtime.

## Phase 1: Shared Test Harness

Create `stellaux_server/tests/support/mod.rs` with:

- `test_db_url()` that requires `TEST_DATABASE_URL` in CI
- `reset_public_schema()` helpers for isolated test setup
- `apply_embedded_migrations()` that calls `Migrator::up(&db, None)`
- `test_state()` / `test_router()` builders so integration tests exercise the real router and middleware stack

Outcome:

- every integration test uses the same DB/bootstrap path
- migrations tested in CI match the production boot path

## Phase 2: Platform Smoke Tests

Add integration coverage for the deployment-critical surface:

- `tests/platform_health_integration.rs`
  - `GET /healthz` returns 200 without DB access side effects
  - `GET /readyz` returns 200 after migrations and DB connect succeed
- `tests/bootstrap_migrations_integration.rs`
  - app boot creates `seaql_migrations`
  - repeated boot is idempotent

Outcome:

- the same readiness gate Fly uses is covered in CI
- migration-on-boot behavior is no longer assumed

## Phase 3: Auth And Admin Boundaries

Expand the existing admin test coverage into a small auth matrix:

- `tests/admin_products_integration.rs`
  - valid internal admin token can create/update product records
  - missing `X-Original-User` is rejected
  - invalid internal token is rejected
- `tests/supabase_auth_integration.rs`
  - protected route returns 500 when `SUPABASE_JWKS_URL` is intentionally unset
  - protected route accepts a valid mocked Supabase JWT when JWKS is available
  - admin route rejects non-admin role

Use `wiremock` for JWKS responses so no real network access is required.

Outcome:

- deploy-time misconfiguration around Supabase auth becomes visible before merge

## Phase 4: Checkout And Webhook Flows

Add database-backed tests for the highest-risk customer paths:

- `tests/cart_checkout_integration.rs`
  - add to cart
  - create checkout session
  - reserve inventory
- `tests/webhooks_integration.rs`
  - Stripe webhook accepted with valid signature
  - duplicate webhook is idempotent
  - checkout completion produces the expected order-side effects

Use `wiremock` for Stripe / Shippo / Resend provider calls where the code depends on outbound HTTP.

Outcome:

- the release gate begins to cover the order lifecycle instead of only compile/build health

## Phase 5: Storage And Media

Add tests around media/storage behavior:

- local storage upload + fetch path in a temp directory
- S3-config validation failure when required env vars are missing
- admin media endpoints return stable URLs

Outcome:

- Fly/storage misconfiguration gets caught during CI rather than after deploy

## CI Changes After Phase 1

Once the shared harness is in place:

- make `TEST_DATABASE_URL` the canonical env var for integration runs
- fail fast if it is missing in CI
- keep `cargo test --tests -- --nocapture` as the execution command
- consider splitting integration jobs by concern once runtime grows:
  - `platform-and-auth`
  - `catalog-and-admin`
  - `checkout-and-webhooks`

## Recommended Execution Order

1. Shared harness + migration-on-boot coverage
2. Health/readiness tests
3. Auth/admin matrix
4. Checkout + webhook idempotency
5. Storage/media behavior

This order hardens deployment confidence first, then expands into revenue-critical workflows.
