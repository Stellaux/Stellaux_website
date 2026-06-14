# Plan: Phase 0 Platform Foundation and Contracts

## Bounded Contexts Affected

- `auth`
- `webhooks`
- cross-cutting platform/runtime in `src/common/`

## Goal

Lock the backend system boundaries before product, order, shipment, and customer-service implementation accelerates.

Phase 0 is not primarily about feature delivery. It is about making the operational backbone explicit so later phases do not have to redesign:

- runtime wiring
- external service boundaries
- storage and data-flow boundaries
- schema ownership
- migration discipline
- webhook persistence
- audit conventions
- idempotency conventions
- API contracts shared with the website and internal dashboard

## Current Baseline

The repo already has partial foundation pieces:

- `src/common/bootstrap.rs` boots config, DB, HTTP client, storage, and migrations
- `src/common/config.rs` already models Stripe, Shippo, Resend, storage, and Supabase JWKS config
- the architecture decision is now to use Supabase Postgres for operational data and Supabase Storage for product images
- `src/common/storage.rs`, `src/common/jwt.rs`, and `src/common/auth.rs` provide infrastructure scaffolding
- `src/domains/webhooks/api/routes.rs` exists but is still a stub
- `src/domains/auth/` exists but only `api/` and `dto/` are populated
- `shared/models/` exists as the intended SQL source of truth, but the current files are inconsistent and not yet a reliable canonical schema
- `src/migration/` runs SeaORM migrations at boot, but it is not yet clearly tied to the SQL-first discipline described in `AGENTS.md`

Phase 0 should convert that partial scaffolding into explicit contracts the rest of the roadmap can safely build on.

## Implementation Scope

### 1. Runtime Architecture Contract

Purpose:

- make the service boundaries explicit for the API server, Postgres, storage, auth, payments, shipping, and email
- finalize the product-image storage decision so internal tooling and storefront work share the same assumptions
- define which components are authoritative versus mirrored
- ensure the runtime config surface matches those boundaries

Files to modify:

- `stellaux_server/src/common/config.rs`
- `stellaux_server/src/common/bootstrap.rs`
- `stellaux_server/src/common/app_state.rs`
- `stellaux_server/src/common/storage.rs`
- `stellaux_server/src/server.rs`

Likely new files:

- `docs/architecture/platform-runtime-contract.md`
- `docs/architecture/service-boundaries.md`

Layers modified:

- `common`
- API composition
- architecture documentation

Planned work:

- document the runtime contract for Rust API, Supabase Postgres, Supabase Storage, Stripe, Shippo, Resend, and Supabase JWKS auth
- standardize required/optional env vars by integration and environment
- confirm which clients/resources belong on `AppState` and which should stay request-local
- lock the production-vs-dev behavior for storage, webhook body limits, and external HTTP clients
- define the data flow between Postgres metadata and Supabase Storage product-image binaries
- define failure posture for missing integration config:
  - boot fail
  - route fail
  - feature disabled

### 2. Auth Boundary and Identity Contract

Purpose:

- separate cross-cutting token verification concerns from domain-level auth flows
- lock the unified identity decision before customer and admin features build on it

Files to modify:

- `stellaux_server/src/common/auth.rs`
- `stellaux_server/src/common/jwt.rs`
- `stellaux_server/src/common/roles.rs`
- `stellaux_server/src/domains/auth/mod.rs`
- `stellaux_server/src/domains/auth/api/routes.rs`

Likely new files:

- `stellaux_server/src/domains/auth/domain/mod.rs`
- `stellaux_server/src/domains/auth/domain/ports.rs`
- `stellaux_server/src/domains/auth/application/mod.rs`
- `stellaux_server/src/domains/auth/application/session_contracts.rs`
- `stellaux_server/src/domains/auth/infra/mod.rs`
- `docs/architecture/auth-contract.md`

Layers modified:

- `common`
- `auth/api`
- `auth/application`
- `auth/domain`
- `auth/infra`

Planned work:

- define the permanent split between:
  - `src/common/auth.rs` for transport/middleware/JWKS verification
  - `src/domains/auth/*` for business auth flows like signup, password reset, and user-facing session contracts
- confirm token policy:
  - when server-issued JWTs are allowed
  - when Supabase JWTs are required
  - how admin/support/staff/customer roles are resolved
- finalize ownership of user records, session concepts, password-reset tokens, and role data
- ensure downstream domains can depend on a stable caller identity contract without importing implementation details

### 3. Webhook, Audit, and Idempotency Conventions

Purpose:

- establish the operational event-ingestion contract before Stripe, Shippo, eBay, and Etsy flows are implemented deeply

Files to modify:

- `stellaux_server/src/domains/webhooks/mod.rs`
- `stellaux_server/src/domains/webhooks/api/routes.rs`
- `stellaux_server/src/common/audit.rs`
- `stellaux_server/src/common/error.rs`

Likely new files:

- `stellaux_server/src/common/idempotency.rs`
- `stellaux_server/src/domains/webhooks/dto/mod.rs`
- `stellaux_server/src/domains/webhooks/domain/mod.rs`
- `stellaux_server/src/domains/webhooks/domain/webhook_event.rs`
- `stellaux_server/src/domains/webhooks/domain/webhook_repository.rs`
- `stellaux_server/src/domains/webhooks/application/mod.rs`
- `stellaux_server/src/domains/webhooks/application/process_webhook_use_case.rs`
- `stellaux_server/src/domains/webhooks/infra/mod.rs`
- `stellaux_server/src/domains/webhooks/infra/seaorm_webhook_repository.rs`
- `docs/architecture/webhook-and-idempotency-contract.md`

Layers modified:

- `common`
- `webhooks/api`
- `webhooks/application`
- `webhooks/domain`
- `webhooks/infra`

Planned work:

- define the canonical lifecycle for incoming external events:
  - receive raw payload
  - verify signature
  - persist receipt
  - deduplicate/idempotency check
  - dispatch business processing
  - record success/failure
- distinguish:
  - technical webhook/event log
  - business audit log
  - idempotency key storage
- establish event naming and source naming conventions that later marketplace integrations can reuse
- keep Phase 0 focused on contracts and persistence seams, not full Stripe/Shippo business logic

### 4. SQL Source-of-Truth and Migration Discipline

Purpose:

- make `shared/models/` the real canonical schema source before Phase 1 and Phase 2 start adding product/order tables aggressively

Files to modify:

- `shared/models/catalog.sql`
- `shared/models/channel.sql`
- `shared/models/email_token.sql`
- `shared/models/guest.sql`
- `shared/models/inventory.sql`
- `shared/models/order_items.sql`
- `shared/models/orders.sql`
- `shared/models/session.sql`
- `shared/models/user.sql`
- `stellaux_server/src/migration/mod.rs`
- existing migration files under `stellaux_server/src/migration/`

Likely new files:

- `shared/models/001_platform_foundation.sql`
- `shared/models/002_catalog_inventory.sql`
- `shared/models/003_orders_customers.sql`
- `shared/models/004_webhooks_audit_idempotency.sql`
- `stellaux_server/src/migration/m2026xxxx_00000x_platform_foundation.rs`
- `docs/architecture/schema-ownership-matrix.md`
- `docs/architecture/migration-discipline.md`

Layers modified:

- SQL schema source of truth
- migration runner
- architecture documentation

Planned work:

- normalize the SQL files into valid Postgres DDL
- decide whether Phase 0 should:
  - consolidate the current loose SQL files into numbered canonical SQL artifacts
  - or retain the current split but add strict naming/ordering conventions
- define how SeaORM migration files embed or execute the canonical SQL from `shared/models/`
- document the ownership of each table by bounded context
- explicitly separate:
  - foundational tables needed before feature work
  - product/inventory tables
  - order/customer tables
  - webhook/audit/idempotency tables

### 5. Bounded Context and Table Ownership Matrix

Purpose:

- prevent Phase 1 through Phase 4 from duplicating models or crossing domain boundaries

Files to modify:

- `plans/overview/overview_timeline.md`

Likely new files:

- `docs/architecture/bounded-context-map.md`
- `docs/architecture/table-ownership-matrix.md`
- `docs/architecture/api-contract-surface.md`

Layers modified:

- planning documentation
- architecture documentation

Planned work:

- finalize which bounded contexts own which responsibilities and tables
- clarify which cross-cutting concerns remain in `src/common/` versus domain-local code
- define the minimum API contracts the website and dashboard can safely target during later parallel work
- ensure frontend teams can work against stable contracts without dictating backend sequencing

## Proposed Execution Order

1. Confirm runtime architecture and integration boundaries in `src/common/` and architecture docs.
2. Lock the auth boundary and caller identity contract.
3. Define webhook, audit, and idempotency conventions plus persistence seams.
4. Normalize SQL source-of-truth structure and migration discipline.
5. Publish bounded-context and table ownership matrices for later phases.

## Key Decisions To Resolve During Implementation

- Whether server-issued JWTs remain a long-term supported path or only a transitional/service-account path beside Supabase JWTs
- Product-image storage is no longer an open decision: use Supabase Storage, with Postgres holding image metadata and references
- Whether foundational operational tables like `webhook_events`, `audit_log`, and `idempotency_keys` land in Phase 0 or are only specified here and created at the start of Phase 1
- Whether `shared/models/` should be restructured into numbered migration-style SQL files now, before more schema files are added
- Whether `auth` and `webhooks` should get full `application/domain/infra` scaffolding in Phase 0 or only the contracts needed to support the next phases

## Changes To `src/common/`

Expected and justified. Phase 0 is primarily a cross-cutting foundation pass.

Most likely `src/common/` changes:

- hardening configuration/runtime boundaries
- clarifying auth/JWKS responsibilities
- formalizing audit conventions
- introducing idempotency support
- documenting service integration posture

## Validation After Implementation

- `cargo check`
- `cargo test --lib`
- architecture docs reviewed for consistency with `AGENTS.md`
- migration strategy confirmed against the SQL-first rule in `AGENTS.md`
- no new business logic added to deprecated `src/domain/`

## Out of Scope

- Full product/catalog CRUD
- Marketplace ingestion implementation
- Full order lifecycle logic
- Shippo label purchase workflow
- Customer-service UI or full website UX

Those belong to later phases once Phase 0 contracts are locked.
