# Plan: Convert `shared/models/` SQL Into Supabase CLI Migrations

## Bounded Contexts Affected

- `auth`
- `catalog`
- `inventory`
- `orders`
- `channel`

## Goal

Move the canonical SQL currently living in `shared/models/` into first-class Supabase CLI migrations under `supabase/migrations/`, with valid dependency ordering and migration-safe SQL.

The target outcome is:

- Supabase CLI becomes the migration entrypoint for schema evolution
- `shared/models/` no longer acts as an unsequenced pile of SQL files
- schema dependencies are resolved in migration order
- invalid or deferred schema fragments are either fixed or explicitly staged

## Current Baseline

### Existing Supabase project layout

- `supabase/` already exists
- `supabase/migrations/20260609042630_new-migration.sql` exists but is empty

### Existing canonical SQL files

- `shared/models/catalog.sql`
- `shared/models/channel.sql`
- `shared/models/email_token.sql`
- `shared/models/guest.sql`
- `shared/models/inventory.sql`
- `shared/models/order_items.sql`
- `shared/models/orders.sql`
- `shared/models/session.sql`
- `shared/models/user.sql`

### Key dependency and validity issues discovered

- `guest.sql` references `session(id)`, but `session.sql` is currently only a placeholder comment
- `orders.sql` depends on `users` and `guest_profiles`
- `order_items.sql` depends on `orders`, `products`, and `product_variants`
- `inventory.sql` depends on `catalog.sql`
- `channel.sql` depends on `catalog.sql`
- `catalog.sql` is the strongest foundation and should land before channel/inventory/order-item references

## Implementation Scope

### 1. Define the migration breakdown and ordering

Files to modify:

- `supabase/migrations/20260609042630_new-migration.sql`

Likely new files:

- `supabase/migrations/<timestamp>_platform_foundation.sql`
- `supabase/migrations/<timestamp>_catalog.sql`
- `supabase/migrations/<timestamp>_inventory.sql`
- `supabase/migrations/<timestamp>_orders.sql`
- `supabase/migrations/<timestamp>_channel_listings.sql`

Layers modified:

- database schema
- migration orchestration

Planned work:

- replace the empty placeholder migration with real timestamped migration files
- split the schema into dependency-safe units instead of one unordered dump
- make apply order explicit via Supabase migration filenames

### 2. Normalize and fix the SQL while converting it

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
- corresponding new files under `supabase/migrations/`

Layers modified:

- canonical SQL
- migration SQL

Planned work:

- ensure all SQL is valid PostgreSQL/Supabase migration SQL
- fix table/reference names where needed for consistency
- resolve incomplete dependencies before migration creation
- decide explicitly whether to:
  - implement `session` now because `guest_profiles` depends on it, or
  - defer both `session` and `guest_profiles` together into a later migration

Important note:

- `session.sql` is currently not implementable as-is because it is only a placeholder comment.
- That means this task is not just “copy files”; it includes a schema staging decision.

### 3. Establish the canonical relationship between `shared/models/` and `supabase/migrations/`

Files to modify:

- `plans/overview/overview_timeline.md`
- `docs/architecture/migration-discipline.md`
- potentially `docs/architecture/schema-ownership-matrix.md`

Layers modified:

- architecture documentation
- planning documentation

Planned work:

- define whether `shared/models/` remains:
  - a canonical design/reference layer from which Supabase migrations are authored, or
  - fully deprecated in favor of `supabase/migrations/`
- document the rule so future schema work does not drift into both places inconsistently

Recommended direction:

- after conversion, Supabase migrations should be the executable source of truth
- `shared/models/` should either be retired or kept only as curated design documentation, not as a competing schema definition source

### 4. Align Rust-side migration expectations with the Supabase-first workflow

Files to inspect and likely modify:

- `stellaux_server/src/migration/mod.rs`
- `stellaux_server/src/common/bootstrap.rs`
- architecture docs describing migration flow

Layers modified:

- runtime migration expectations
- architecture documentation

Planned work:

- document whether the Rust service should continue applying SeaORM migrations at boot
- if Supabase CLI becomes the primary migration path, make the runtime docs reflect that clearly
- avoid leaving two competing migration authorities active without a contract

Important note:

- This step may remain documentation-only in the first pass if changing the runtime migration mechanism would be too broad, but the authority must be clarified.

## Proposed Migration Order

1. Platform/auth foundation
   - `users`
   - `email_tokens`
   - `session` if implemented now
   - `guest_profiles` only if `session` lands in the same sequence
2. Catalog foundation
   - categories
   - category_size_options
   - collections
   - products
   - product_collections
   - product_variants
   - size trigger/function
   - product_media
3. Inventory
   - inventory tables in `private`
4. Orders
   - `orders`
   - `order_items`
5. Channel mapping
   - `channel_listings`

## Key Decisions Requiring Care During Implementation

- Whether `session` and `guest_profiles` should be implemented now or deferred together
- Whether to preserve `shared/models/` after conversion or retire it
- Whether the current SeaORM-at-boot migration flow should remain active alongside Supabase migrations temporarily

## Validation After Implementation

- `supabase/migrations/` contains ordered, non-empty SQL migration files
- schema dependencies resolve in migration order
- no migration references a table that has not yet been created
- architecture docs clearly state the migration authority

## Out of Scope

- adding new product/order business logic
- implementing Stripe/Shippo/webhook schemas unless they are already present in `shared/models/`
- broad refactors to SeaORM entities beyond what is needed to keep the schema story coherent

