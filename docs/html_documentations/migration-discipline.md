# Migration Discipline

## Source Of Truth

Canonical schema definitions live under `shared/models/`.

SeaORM migrations under `stellaux_server/src/migration/` are execution artifacts that apply the approved schema to the database. They are not the primary place to invent schema shape.

## Phase 0 Rule

Before Phase 1 feature work expands catalog and order schemas, the team must normalize `shared/models/` into valid, reviewable Postgres DDL with stable ordering and clear table ownership.

## Expected Workflow

1. update canonical SQL in `shared/models/`
2. review schema ownership and boundary impact
3. add or update the SeaORM migration that applies the approved SQL
4. register the migration in `src/migration/mod.rs`
5. validate locally with `cargo check` and migration boot

## Ordering Rule

Migration ordering must be deterministic and append-only:

- each migration gets a timestamped filename
- `Migrator::migrations()` order is the apply order
- previously applied migrations are never edited in incompatible ways

## Boundary Rule

Foundational tables must be established before feature-specific tables rely on them:

- users / roles / sessions
- webhook event log
- audit log
- idempotency records
- core catalog and inventory tables
- order and shipment tables

## Anti-Pattern To Avoid

Do not let ad hoc SeaORM Rust table definitions drift away from `shared/models/`. If a schema decision matters to the product model, it must be visible in the canonical SQL source first.
