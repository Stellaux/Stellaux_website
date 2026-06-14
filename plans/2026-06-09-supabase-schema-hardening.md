# Plan: Harden Supabase Schema Boundaries and Provisioning Rules

## Bounded Contexts Affected

- `catalog`
- `inventory`
- `orders`
- `auth`
- cross-cutting Supabase platform/runtime contracts

## Goal

Harden the Supabase-facing architecture before more schema lands by resolving four high-impact risks:

1. uncontrolled exposure of `public` schema tables through Supabase PostgREST
2. missing creation of the `private` schema used by internal tables
3. ambiguous or unsafe Supabase connection-mode guidance for the Rust server
4. unclear migration authority between Rust/SeaORM and Supabase tooling

## Why This Plan Exists

The current architecture says the Rust API is the only supported business interface, but:

- core catalog tables currently live in `public`
- internal tables already assume a `private` schema that is not explicitly created
- older docs still describe storage and Supabase behavior from an earlier architecture
- migration authority is still split across architecture docs, Rust boot behavior, and now an emerging Supabase CLI workflow

These are painful to retrofit after the real Supabase project is populated, so they should be locked down now.

## Implementation Scope

### 1. Define `public` vs `private` schema placement explicitly

Purpose:

- make schema exposure intentional rather than accidental
- keep PostgREST visibility and internal operational data boundaries aligned with the Rust API architecture

Files to modify:

- `shared/models/catalog.sql`
- `shared/models/channel.sql`
- `shared/models/inventory.sql`
- `shared/models/orders.sql`
- `shared/models/order_items.sql`
- `shared/models/user.sql`
- `shared/models/guest.sql`
- related Supabase migration files once conversion happens

Likely new docs to modify:

- `docs/architecture/platform-runtime-contract.md`
- `docs/architecture/schema-ownership-matrix.md`
- `docs/architecture/table-ownership-matrix.md`

Layers modified:

- SQL schema design
- architecture documentation

Planned work:

- define a stable rule for which tables belong in `public`
- define a stable rule for which tables belong in `private`
- document whether any `public` tables are intentionally client-readable via Supabase APIs, versus only queryable through the Rust API

Recommended default:

- `public`: only tables intentionally exposed to Supabase clients, always with explicit RLS posture
- `private`: inventory, audit, webhook, idempotency, reconciliation, and other internal operational tables

### 2. Add `private` schema bootstrap as a foundational migration requirement

Purpose:

- make fresh Supabase provisioning succeed
- align the existing `private.inventory` design with actual database initialization

Files to modify:

- earliest foundational SQL artifact under `shared/models/`
- corresponding Supabase migration file(s)
- potentially `stellaux_server/src/migration/` docs if Rust-side migration authority remains documented

Likely new files:

- foundational Supabase migration containing `create schema if not exists private;`

Layers modified:

- schema bootstrap
- migration ordering

Planned work:

- create `private` before any internal tables reference it
- ensure all internal-only tables that assume `private` are sequenced after schema creation
- document `private` as a first-class convention, not an ad hoc table prefixing strategy

### 3. Lock down PostgREST exposure and RLS/grant posture for `public`

Purpose:

- prevent accidental data exposure via Supabase auto-generated REST APIs
- preserve the architecture rule that the Rust API is the primary business interface

Files to modify:

- Supabase migration SQL for `public` tables
- potentially `shared/models/` if policy declarations or comments are kept alongside canonical SQL
- `docs/architecture/platform-runtime-contract.md`
- `docs/architecture/external-services.html`

Layers modified:

- database security posture
- architecture documentation

Planned work:

- choose one explicit hardening posture for `public` tables:
  - enable RLS deny-by-default with no permissive policies unless intentionally required
  - and/or revoke default grants from `anon` / `authenticated`
- document which tables, if any, are intentionally exposed to Supabase clients
- make the default expectation “no accidental public data access through Supabase”

Recommended direction:

- RLS enabled on every `public` business table
- deny-by-default unless a specific read path is intentionally supported

Open design nuance:

- if the future website or internal tooling should read some catalog rows directly via Supabase, that should be a deliberate exception with explicit policies, not an emergent side effect of using the `public` schema

### 4. Standardize Supabase connection-mode guidance for the Rust server

Purpose:

- prevent prepared-statement and pooling issues in production
- make initial environment setup unambiguous

Files to modify:

- `docs/architecture/platform-runtime-contract.md`
- `docs/architecture/external-services.html`
- `docs/architecture/deployment.html`
- any environment/setup doc that mentions `DATABASE_URL`

Layers modified:

- architecture documentation
- deployment guidance

Planned work:

- document that the Rust server should use:
  - direct Postgres connection on `:5432`, or
  - a session-mode pooler if intentionally introduced later
- explicitly warn against transaction-mode poolers for SeaORM/sqlx prepared-statement usage
- align boot-fail setup guidance with this connection rule

### 5. Declare a single migration authority

Purpose:

- avoid drift between Rust boot migrations, Supabase CLI migrations, and ad hoc Studio changes

Files to modify:

- `docs/architecture/migration-discipline.md`
- `docs/architecture/platform-runtime-contract.md`
- `plans/2026-06-09-shared-models-to-supabase-migrations.md`
- any Supabase setup docs

Potential runtime docs to inspect:

- `stellaux_server/src/common/bootstrap.rs`
- `stellaux_server/src/migration/mod.rs`

Layers modified:

- architecture documentation
- migration workflow documentation

Planned work:

- declare one primary DDL authority for the project
- explicitly forbid schema changes made only through Supabase Studio UI
- define how Rust boot behavior relates to the chosen authority

Decision to resolve in implementation:

- either:
  - Supabase CLI migrations become the primary executable authority and Rust stops being described as the migration owner
- or:
  - Rust/SeaORM remains the only migration authority and Supabase CLI is not used for production DDL

The project should not leave both paths active without a documented source-of-truth rule.

### 6. Sync older architecture docs to the hardened Supabase posture

Purpose:

- eliminate contradictory guidance that will confuse future agents and implementers

Files to modify:

- `docs/architecture/external-services.html`
- `docs/architecture/index.html`
- `docs/architecture/endpoints-rest.html`
- `docs/architecture/internal-tools.html`
- any other legacy architecture HTML that still assumes:
  - public catalog exposure by default
  - generic S3/R2 storage as the selected production image path
  - transaction pooler guidance that conflicts with SeaORM/sqlx usage

Layers modified:

- architecture documentation

Planned work:

- align all docs with:
  - Supabase Postgres as operational truth
  - Supabase Storage for product imagery
  - hardened `public`/`private` schema rules
  - explicit migration authority
  - safe connection-mode guidance

## Proposed Execution Order

1. Define and document `public` vs `private` placement rules.
2. Add `create schema if not exists private;` to foundational migrations.
3. Add RLS/grant hardening for `public` tables.
4. Standardize direct/session connection guidance and remove transaction-pool ambiguity.
5. Declare the single migration authority.
6. Sync legacy architecture docs to match the hardened rules.

## Validation After Implementation

- a fresh Supabase project can create `private` successfully before internal tables are applied
- `public` tables have explicit exposure controls, not implicit ones
- docs consistently state whether the Rust API or Supabase clients may read/write each class of table
- connection guidance consistently points the Rust server at a prepared-statement-safe mode
- migration authority is stated once and echoed consistently across setup docs

## Out of Scope

- implementing full product/order/shipment business logic
- designing final marketplace RLS policies for every future dashboard/client use case
- broader refactors unrelated to Supabase boundary hardening

