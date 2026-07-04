## Context & Goal

Wire `stellaux_server` to deploy on Fly.io in a way that matches the existing deployment strategy:

- auto-deploy staging from `main`
- deploy production from a release tag or manual promotion
- gate deploy success on app readiness, not just process liveness
- keep the release build free of dev-only Swagger UI asset downloads
- document the next integration-test expansion steps

This change is cross-cutting deployment/release work rather than a single bounded context feature.

## Files To Create / Modify

### Deployment assets

- `stellaux_server/fly.staging.toml`
- `stellaux_server/fly.production.toml`
- `.github/workflows/backend-release.yml`

### Build / dependency hardening

- `stellaux_server/Cargo.toml`
- `stellaux_server/Dockerfile`

### Documentation

- `docs/CI/CD_Pipeline.md`
- `docs/INTEGRATION_COVERAGE_PLAN.md`

## Proposed Schema Changes

None.

## Observability Plan

- Reuse the existing `/healthz`, `/readyz`, and `/metrics` endpoints.
- Configure Fly health checks to use `/readyz` so deploy success means:
  - Postgres is reachable
  - boot-time migrations have completed
  - the app is actually able to serve traffic

## Risks / Notes

- Fly app names and primary region are reasonable defaults and may need adjustment before first launch.
- Fly secrets still need to be configured outside git, especially:
  - `DATABASE_URL`
  - `JWT_SECRET`
  - `CORS_ORIGINS`
  - `SUPABASE_JWKS_URL`
  - `SUPABASE_ISSUER`
  - payment/shipping/email secrets as those features move to staging/prod
- `STORAGE_BACKEND` should be explicitly configured for Fly deployments to avoid accidental reliance on ephemeral local disk.
