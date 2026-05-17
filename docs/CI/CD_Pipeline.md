# CI/CD Pipeline

## Overview

The backend CI/CD pipeline is split into two GitHub Actions workflows:

- `.github/workflows/backend-ci.yml` runs on pull requests to `main` and on direct pushes to `main`.
- `.github/workflows/backend-release.yml` runs on pushes to `main`, on `release/**` tags, and on manual dispatches.

This keeps fast quality gates on every change while reserving the heavier release build and container validation for the mainline branch.

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

- `cargo build --release --bin stellaux_server`
- artifact upload of the release binary
- Docker multi-stage image build using `docker/build-push-action`
- Docker layer caching with:
  - `cache-from: type=gha`
  - `cache-to: type=gha,mode=max`

The server Dockerfile also sets `SQLX_OFFLINE=true` in the builder stage and copies the full crate after dependency cooking so future `.sqlx/` metadata or embedded migrations can be included without giving up dependency-layer caching.

## Current Design Notes

- The workflows target `stellaux_server/` directly because the repository does not yet have a root Rust workspace.
- `Swatinem/rust-cache` is enabled to cache Cargo dependencies and the backend `target/` directory.
- The integration-test job is intentionally separate from the fast PR gate so database-backed tests can grow independently.
- The release workflow currently validates the production Docker image but does not yet publish or deploy it to a hosting platform.

## Future Improvements

The current workflows are a strong CI foundation, but several recommendations should be completed before calling the pipeline production-ready.

### 1. Real SQLx Offline Enforcement

Add a committed `stellaux_server/.sqlx/` directory and update the backend flow so:

- `cargo sqlx prepare --check` always runs
- release and CI builds rely on offline metadata consistently

This is currently deferred because the repository does not yet ship SQLx offline metadata.

### 2. Database Migrations in CI

Before production rollout, add a real migration strategy that CI can execute against the temporary PostgreSQL container.

Recommended next step:

- wire up a proper migration crate or embedded migration runner
- apply migrations before integration tests
- keep schema changes backward-compatible for rolling deploys

The existing `migration/` crate is still only a placeholder and is not yet part of the backend startup or CI path.

### 3. Staging and Production Environments

Introduce two isolated deployment targets:

- `staging`: auto-deploy on every push to `main`
- `production`: deploy only from a manual approval step or a pushed `release/**` tag

Each environment should have:

- its own database
- separate secrets
- independent app URLs
- GitHub Environments with approval rules

### 4. Zero-Downtime Hosting

Once a hosting platform is selected, extend the release workflow to deploy only after the new revision is healthy.

Recommended platforms for this backend:

- Fly.io
- Railway

The deployment job should wait for health checks to pass and only then mark the rollout successful.

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
