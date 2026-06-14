# Plan: Review Supabase Authentication State, Complete Setup, Then Add RLS Policies

## Bounded Contexts Affected

- `auth`
- `catalog`
- `account`
- `orders`
- `inventory`
- `webhooks`
- cross-cutting Supabase platform configuration

## Goal

1. Review and stabilize the current Supabase authentication setup.
2. Continue Supabase auth setup where the repo is incomplete or inconsistent.
3. Then implement explicit policy layers for:
   - public catalog read
   - authenticated self-access
   - admin/service-role-only tables

## Current State Review

### Rust server auth state

The Rust API already has a real Supabase JWT verification path:

- `stellaux_server/src/common/auth.rs`
- `stellaux_server/src/common/jwt.rs`
- `stellaux_server/src/common/config.rs`

What is already working in design:

- `require_supabase_auth` verifies Supabase JWTs through JWKS
- `require_supabase_admin` resolves authorization roles from internal `user_roles`
- `SUPABASE_JWKS_URL`, `SUPABASE_ISSUER`, and `SUPABASE_AUDIENCE` are configurable
- route groups in `server.rs` already use the Supabase auth middleware for protected/admin areas

### Supabase project state

The repo already has:

- `supabase/`
- `supabase/migrations/`
- `client/supabase/config.toml`

Current project ref in the client-side Supabase config:

- `client/supabase/config.toml` → `project_id = "ubqozvbtwfnnzzcmlgfb"`

### Major inconsistency discovered

There are effectively two Supabase stories in the repo right now:

1. the newer Rust-server-owned auth and schema direction in:
   - `stellaux_server/`
   - `shared/models/`
   - `supabase/migrations/`
2. an older client-Supabase-direct schema/auth path in:
   - `client/supabase/migrations/...`
   - `client/src/integrations/supabase/*`

This matters because the policy strategy depends on whether the browser is expected to read/write data directly through Supabase or primarily through the Rust API.

### Review findings

The auth review narrows the architecture decision enough to plan implementation concretely:

- Supabase Auth is already the canonical human identity provider in the Rust API.
- The Rust API already expects the caller id to be the Supabase user id (`auth.users.id`).
- `user_roles` is already treated as internal authorization state, not identity state.
- The new `supabase/migrations/*.sql` currently model a separate `public.users` table as if it were the identity root.
- The older client-side migrations and policies assume the real identity root is `auth.users`, which matches Supabase semantics and the current Rust middleware.

## Architecture Decision To Lock Before Policy Work

### Identity source of truth

Use **Supabase Auth (`auth.users`) as the only human identity root**.

Implications:

- Supabase-issued JWT `sub` maps to the canonical user id.
- RLS self-access policies should be written against `auth.uid()`.
- `public.user_roles` remains an application-owned authorization table keyed by the Supabase user id.
- Any application-owned profile/account table should be a 1:1 extension of `auth.users`, not a replacement for it.

### Business-interface source of truth

Use the **Rust API as the primary business interface**.

Implications:

- direct browser access through Supabase should be intentionally narrow
- RLS is used for carefully chosen cases, not as a substitute for backend domain logic
- order creation, inventory mutation, webhook ingestion, and admin operations stay backend-owned

### Data-access split to design around

The Phase 0/1 baseline should be:

- public catalog data may be readable directly through Supabase
- authenticated users may directly read only a narrow self-owned surface if we explicitly keep that path
- internal operational tables stay backend/service-role only

## Main Areas / Files

### Supabase platform and SQL

- `supabase/migrations/20260610000100_auth_foundation.sql`
- `supabase/migrations/20260610000200_catalog.sql`
- `supabase/migrations/20260610000300_inventory.sql`
- `supabase/migrations/20260610000400_orders.sql`
- `supabase/migrations/20260610000500_channel_listings.sql`
- `client/supabase/config.toml`

### Rust server

- `stellaux_server/src/common/auth.rs`
- `stellaux_server/src/common/jwt.rs`
- `stellaux_server/src/common/config.rs`
- `stellaux_server/src/common/roles.rs`

### Client-side Supabase integration to reconcile

- `client/src/integrations/supabase/client.ts`
- `client/src/integrations/supabase/client.server.ts`
- `client/src/integrations/supabase/auth-middleware.ts`
- `client/supabase/migrations/*`

### Documentation

- `docs/architecture/auth-contract.md`
- `docs/architecture/platform-runtime-contract.md`
- `docs/architecture/schema-ownership-matrix.md`
- `docs/architecture/table-ownership-matrix.md`

## Implementation Scope

### 1. Finalize the Supabase auth contract

Purpose:

- make the project’s authentication path unambiguous before policy work lands

Files to modify:

- `stellaux_server/src/common/auth.rs`
- `stellaux_server/src/common/jwt.rs`
- `stellaux_server/src/common/config.rs`
- `docs/architecture/auth-contract.md`
- `docs/architecture/platform-runtime-contract.md`
- potentially `docs/architecture/external-services.html`

Likely files to inspect and align:

- `client/src/integrations/supabase/client.ts`
- `client/src/integrations/supabase/client.server.ts`
- `client/src/integrations/supabase/auth-middleware.ts`

Planned work:

- confirm the canonical human auth flow is Supabase Auth
- record the exact required values for:
  - `SUPABASE_JWKS_URL`
  - `SUPABASE_ISSUER`
  - `SUPABASE_AUDIENCE`
- define HS256 server-issued JWTs as non-primary and backend/transitional only
- document whether client auth tokens are expected to be passed to the Rust API, or used directly against Supabase data APIs

Recommended direction:

- Supabase Auth for human identity
- Rust API as primary business interface
- direct Supabase data access only where intentionally allowed by policy

Concrete acceptance for this step:

- auth docs no longer present `public.users` as the primary identity owner
- Supabase env contract is explicit enough to configure local/dev/prod consistently
- the team has one stated answer for "who owns identity?"

### 2. Reconcile old client-side Supabase assumptions with the new backend-owned architecture

Purpose:

- avoid building policies for two contradictory data-access models

Files to inspect and likely modify:

- `client/supabase/migrations/*`
- `client/src/integrations/supabase/client.ts`
- `client/src/integrations/supabase/client.server.ts`
- `client/src/integrations/supabase/auth-middleware.ts`
- `docs/architecture/endpoints-rest.html`

Planned work:

- determine whether the old `client/supabase/migrations` history is still authoritative for any tables or should be retired
- identify which client routes still assume direct Supabase table access
- decide whether those routes should:
  - remain direct-to-Supabase with RLS
  - move behind the Rust API
  - or be split by use case

Important note:

- this decision directly affects the policy design for “public catalog read” and “authenticated self-access”

Recommended decision:

- keep client-side Supabase for auth/session wiring
- keep direct table access only for explicitly public-read data
- move account/order/admin mutations behind the Rust API unless there is a strong reason to keep a direct path
- treat `client/supabase/migrations/*` as legacy reference once the new root migration set is authoritative

### 3. Resolve the `public.users` versus `auth.users` mismatch

Purpose:

- prevent self-access policies and ownership joins from being built on contradictory identity models

Files to modify:

- `supabase/migrations/20260610000100_auth_foundation.sql`
- `supabase/migrations/20260610000400_orders.sql`
- canonical SQL under `shared/models/`
- `docs/architecture/auth-contract.md`
- `docs/architecture/schema-ownership-matrix.md`

Planned work:

- decide whether `public.users` becomes:
  - a profile/account-extension table keyed to `auth.users(id)`, or
  - a deprecated artifact removed from the new migration path
- make all self-owned foreign keys and policy checks align with Supabase identity
- ensure `orders.user_id` semantics match the Rust middleware’s `claims.sub`

Recommended direction:

- keep Supabase identity in `auth.users`
- if an application-facing account table is needed, model it as a profile row keyed by the same uuid
- do not maintain an independent login identity in `public.users`

### 4. Add explicit public catalog read policies

Purpose:

- define deliberate read access for catalog data instead of relying on schema placement or accidental openness

Files to modify:

- `supabase/migrations/20260610000200_catalog.sql`
- potentially `supabase/migrations/20260610000500_channel_listings.sql`
- canonical SQL comments in `shared/models/catalog.sql`

Planned work:

- add explicit `create policy` statements for public-facing catalog reads if browser/Supabase direct reads are desired
- keep write access closed unless explicitly needed
- define exactly which tables are readable:
  - likely `categories`
  - `category_size_options`
  - `collections`
  - `products` limited to storefront-visible rows
  - `product_collections`
  - `product_variants`
  - `product_media`
- keep operational or marketplace reconciliation tables out of broad public read unless justified

Design choice to resolve:

- whether `channel_listings` should remain backend-only, even if product catalog rows are readable

Recommended catalog policy shape:

- `anon` and `authenticated` may `select`
- only active/storefront-visible products and variants should be readable
- all inserts/updates/deletes remain blocked for client roles
- `channel_listings` stays backend-only

### 5. Add authenticated self-access policies

Purpose:

- allow authenticated users to access only their own rows where direct Supabase access is still desired

Files to modify:

- `supabase/migrations/20260610000100_auth_foundation.sql`
- `supabase/migrations/20260610000400_orders.sql`
- future account/address/profile migrations if those tables are still used from the client side

Planned work:

- add self-scoped `select` / `insert` / `update` / `delete` policies where appropriate
- likely candidate tables:
  - account/profile extension table keyed by Supabase user id
  - `orders` read access, if direct customer order history is intentionally supported
  - `order_items` via parent-order ownership
- keep self-service writes narrow; prefer Rust API for complex business mutations
- use `auth.uid()` as the only user-ownership primitive

Recommended self-access baseline:

- authenticated users may read their own account/profile row
- authenticated users may read their own orders and order items if direct Supabase read is kept
- order creation and mutation should remain Rust-API-owned for now
- `user_roles` should not be browser-writable

### 6. Add admin/service-role-only posture for internal tables

Purpose:

- make internal operational data intentionally inaccessible to normal client roles

Files to modify:

- `supabase/migrations/20260610000100_auth_foundation.sql`
- `supabase/migrations/20260610000300_inventory.sql`
- `supabase/migrations/20260610000500_channel_listings.sql`
- future webhook/audit/idempotency migrations
- architecture docs describing `private`

Planned work:

- keep `private` tables without public client policies
- ensure operational tables stay inaccessible to `anon` / `authenticated`
- if admin dashboard direct Supabase reads are desired later, add narrow authenticated/admin policies rather than blanket exposure

Recommended default:

- service-role / backend only for:
  - `public.user_roles`
  - `public.email_tokens`
  - `public.session`
  - `public.guest_profiles`
  - inventory internals
  - reconciliation tables
  - webhook/audit/idempotency tables
  - marketplace mapping data unless explicitly needed in the browser
  - direct admin access should flow through the Rust API first, not through wide-open Supabase policies

### 7. Document the final RLS model

Purpose:

- prevent future agents from enabling RLS without defining policies again

Files to modify:

- `docs/architecture/platform-runtime-contract.md`
- `docs/architecture/auth-contract.md`
- `docs/architecture/schema-ownership-matrix.md`
- `docs/architecture/external-services.html`
- possibly `docs/architecture/endpoints-rest.html`

Planned work:

- document that:
  - enabling RLS is only the mechanism
  - explicit policies define behavior
  - deny-by-default is the baseline
- list which access pattern applies to each table family:
  - public read
  - self-only
  - backend/service-role only

## Proposed Execution Order

1. Review and finalize the Supabase auth contract.
2. Resolve the identity-root mismatch by standardizing on `auth.users`.
3. Reconcile old client-side Supabase assumptions with the new Rust-server-owned architecture.
4. Implement public catalog read policies.
5. Implement authenticated self-access policies.
6. Lock internal tables to admin/service-role/backend-only posture.
7. Update docs so the policy model is explicit and durable.

## Key Decisions To Resolve During Implementation

- Whether to keep any direct authenticated customer reads beyond catalog plus order-history
- Whether `public.users` becomes a profile-extension table or is removed from the new migration path
- Whether older `client/supabase/migrations` are still active architecture or should be retired from the main path

## Table Access Matrix To Implement

### Public catalog read

- `public.categories`
- `public.category_size_options`
- `public.collections`
- `public.products`
- `public.product_collections`
- `public.product_variants`
- `public.product_media`

### Authenticated self-access

- user-owned account/profile extension table once identity alignment is finalized
- `public.orders` via `auth.uid() = user_id`
- `public.order_items` via parent-order ownership

### Backend or service-role only

- `public.user_roles`
- `public.email_tokens`
- `public.session`
- `public.guest_profiles`
- `public.channel_listings`
- `private.inventory`
- `private.inventory_adjustment`
- `private.inventory_log`
- `private.inventory_alert`
- webhook/audit/idempotency tables as they land

## New Files To Create During Implementation

- one or more new Supabase migration files dedicated to policy creation
- optional follow-up docs note if the old `client/supabase/migrations` history is formally retired

## Layers Expected To Change

- `common`
  - auth/JWKS contract and config documentation only unless a setup gap is discovered
- `domains/auth`
  - contract documentation and any identity-aligned auth adapters
- `domains/catalog`
  - policy-facing data exposure decisions
- `domains/account`
  - self-access ownership model
- `domains/orders`
  - self-read ownership model
- `domains/webhooks`
  - backend-only table posture documentation

## Validation After Implementation

- Supabase auth config values are explicitly documented
- every RLS-enabled table that needs client access has explicit policies
- backend-only tables remain inaccessible to normal Supabase client roles
- auth and data-access docs no longer imply contradictory models
- self-access policy checks align with Supabase `auth.uid()` and the Rust middleware’s `claims.sub`
