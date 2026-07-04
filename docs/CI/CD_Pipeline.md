# CI/CD Pipeline

## Overview

The backend CI/CD pipeline is split into two GitHub Actions workflows:

- `.github/workflows/backend-ci.yml` runs on pull requests to `main` and on direct pushes to `main`.
- `.github/workflows/backend-release.yml` runs on pushes to `main`, on `v*` and `release/**` tags, and on manual dispatches.

This keeps fast quality gates on every change while reserving release builds and server deployment for the mainline branch and explicit release promotion.

## What Is Enforced Today

### 1. Pull Request Checks

The `Pull Request Checks` job is the first quality gate for the Rust/Axum backend in `stellaux_server/`.

It runs:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo test --lib`

These checks ensure formatting, linting, compilation, and unit-test coverage are validated before backend code is merged.

### 2. Integration-Test Harness

The `Integration Tests` job provisions an isolated PostgreSQL 16 service container and passes CI-safe environment variables to the backend.

It currently runs:

- `cargo test --tests -- --nocapture`

This gives the repository a ready-made path for database-backed integration tests without touching developer machines or any future staging database. The harness is in place even though the backend does not yet contain a dedicated `stellaux_server/tests/` integration suite.

### 3. Database and Security Validation

The `Database and Security` job runs after the baseline compilation checks pass.

It includes:

- `cargo audit` for known Rust dependency vulnerabilities
- `cargo outdated --root-deps-only` as an informational upgrade report
- `typos` across `.github/`, `docs/`, and `stellaux_server/`
- conditional `cargo sqlx prepare --check` when `stellaux_server/.sqlx/` exists

`SQLX_OFFLINE=true` is set for this job so the build path is compatible with future SQLx offline metadata.

### 4. Release Build and Image Validation

The release workflow adds the production-oriented build steps that should only happen after code has reached `main`:

- `cargo build --locked --release --bin stellaux_server`
- artifact upload of the release binary
- Docker multi-stage image build using `docker/build-push-action`
- Docker layer caching with:
  - `cache-from: type=gha`
  - `cache-to: type=gha,mode=max`

The server Dockerfile also sets `SQLX_OFFLINE=true` in the builder stage and copies the full crate after dependency cooking so future `.sqlx/` metadata or embedded migrations can be included without giving up dependency-layer caching.

### 5. Fly.io Deployment

The release workflow now deploys `stellaux_server` to Fly.io:

- `main` branch pushes deploy the staging app with `stellaux_server/fly.staging.toml`
- `v*` tags and manual production dispatches deploy the production app with `stellaux_server/fly.production.toml`
- both deploy jobs wait on Fly health checks that call `GET /readyz`

This means a deployment is only considered healthy after the backend can connect to Postgres and finish boot-time migrations.

## Current Design Notes

- The workflows target `stellaux_server/` directly because the repository does not yet have a root Rust workspace.
- `Swatinem/rust-cache` is enabled to cache Cargo dependencies and the backend `target/` directory.
- The integration-test job is intentionally separate from the fast PR gate so database-backed tests can grow independently.
- Fly deploys run from the `stellaux_server/` crate directory so the Docker build context matches the backend Dockerfile.
- The Fly deploy jobs use GitHub Environments (`staging`, `production`) so approval rules and environment-scoped secrets can be applied in GitHub settings.
- Swagger UI is intentionally a dev-only Cargo feature and is not compiled into the production release path.

## Fly.io Setup

### Required GitHub Environment Secret

Add `FLY_API_TOKEN` to both GitHub Environments:

- `staging`
- `production`

Using environment-scoped secrets allows different tokens or approval policies per environment.

### Required Fly App Setup

Create the two apps before the workflow can deploy:

- `stellaux-server-staging`
- `stellaux-server-production`

Then set the backend secrets on each app with `fly secrets set`.

### Minimum Runtime Secrets / Vars

At minimum, each Fly app needs values for:

- `DATABASE_URL`
- `JWT_SECRET`
- `CORS_ORIGINS`
- `SUPABASE_JWKS_URL`
- `SUPABASE_ISSUER`

Depending on the features enabled in that environment, also set:

- `INTERNAL_ADMIN_TOKEN`
- `STRIPE_SECRET_KEY`
- `STRIPE_WEBHOOK_SECRET`
- `STRIPE_SUCCESS_URL`
- `STRIPE_CANCEL_URL`
- `SHIPPO_API_TOKEN`
- `SHIPPO_WEBHOOK_SECRET`
- `RESEND_API_KEY`
- `RESEND_FROM_EMAIL`
- warehouse address fields
- storage settings (`STORAGE_BACKEND`, `STORAGE_PUBLIC_URL`, and S3/R2-compatible credentials)

## Future Improvements

The current workflows are a strong CI foundation, but several recommendations should be completed before calling the pipeline production-ready.

### 1. Real SQLx Offline Enforcement

Add a committed `stellaux_server/.sqlx/` directory and update the backend flow so:

- `cargo sqlx prepare --check` always runs
- release and CI builds rely on offline metadata consistently

This is currently deferred because the repository does not yet ship SQLx offline metadata.

### 2. Database Migrations in CI

The backend already applies embedded SeaORM migrations on boot via `Migrator::up(&db, None)`. The next CI improvement is to exercise that same boot path inside integration tests.

Recommended next step:

- centralize test setup around the embedded migrator
- apply migrations before integration tests using the same runtime path as production
- keep schema changes backward-compatible for rolling deploys

See `docs/INTEGRATION_COVERAGE_PLAN.md` for the next expansion steps.

### 3. Staging and Production Environments

Maintain two isolated deployment targets:

- `staging`: auto-deploy on every push to `main`
- `production`: deploy only from a manual approval step or a pushed release tag

Each environment should have:

- its own database
- separate secrets
- independent app URLs
- GitHub Environments with approval rules

### 4. Zero-Downtime Hosting

Fly.io is now wired as the default server host. Keep the deploy jobs waiting on readiness checks and scale production to at least two machines once the storage and database topology are ready for it.

### 5. Branch Protection

Enable GitHub branch protection on `main` and require these jobs to pass before merge:

- `Pull Request Checks`
- `Integration Tests`
- `Database and Security`

This is a repository setting rather than a versioned file, so it must be configured in GitHub after the workflows are merged.

### 6. Dependency Maintenance

Dependabot has been added for:

- Rust dependencies in `stellaux_server/`
- GitHub Actions
- npm dependencies in `client/`

As the project matures, add automerge rules or a Renovate policy for low-risk patch updates after CI passes.

### 7. Compiler Cache for Larger Builds

If CI time grows significantly, add `sccache` on top of `Swatinem/rust-cache` to improve rebuild performance across workflows and runners.
