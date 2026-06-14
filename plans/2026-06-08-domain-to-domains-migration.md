# Plan: Migrate Flat `src/domain/` Handlers Into `src/domains/*`

## Bounded Contexts Affected

- `account`
- `admin`
- `auth`
- `cart`
- `catalog`
- `checkout`
- `craft`
- `webhooks`

## Goal

Move the existing flat Axum handler modules from `stellaux_server/src/domain/` into bounded-context folders under `stellaux_server/src/domains/` so the project complies with `AGENTS.md` directory boundaries and stops treating `src/domain/` as the long-term home for feature code.

This migration is structural. The current handlers are still mostly stubs, so the immediate target is to place them in the correct domain folders with clean module wiring and DTO separation, without introducing `application/`, `domain/`, or `infra/` logic that does not exist yet.

## Implementation Scope

### 1. Create a real `src/domains/mod.rs`

New file:

- `stellaux_server/src/domains/mod.rs`

Purpose:

- Export each bounded context module from the crate root.
- Make `crate::domains::<context>` the canonical import path used by `server.rs`.

### 2. Create per-context module trees under `src/domains/`

For each bounded context below, create a directory-based module with `api/` and `dto/` layers. Existing empty scaffolding under `auth/` should be reused rather than replaced.

#### `account`

New files:

- `stellaux_server/src/domains/account/mod.rs`
- `stellaux_server/src/domains/account/api/mod.rs`
- `stellaux_server/src/domains/account/api/routes.rs`
- `stellaux_server/src/domains/account/dto/mod.rs`

Layers modified:

- `api`
- `dto`

Notes:

- Move route registration and handlers from `src/domain/account.rs` into `api/routes.rs`.
- Move `UpdateProfileRequest`, `ChangePasswordRequest`, and `UpsertAddress` into `dto/mod.rs`.

#### `admin`

New files:

- `stellaux_server/src/domains/admin/mod.rs`
- `stellaux_server/src/domains/admin/api/mod.rs`
- `stellaux_server/src/domains/admin/api/routes.rs`
- `stellaux_server/src/domains/admin/dto/mod.rs`

Layers modified:

- `api`
- `dto`

Notes:

- Reuse existing `stellaux_server/src/domains/admin/` directory.
- Move handlers and `routes()` from `src/domain/admin.rs` into `api/routes.rs`.
- Move `OrdersFilter`, `InventoryAdjust`, `CompatibilityPair`, and `UpsertProduct` into `dto/mod.rs`.

#### `auth`

Files to create or populate:

- `stellaux_server/src/domains/auth/mod.rs`
- `stellaux_server/src/domains/auth/api/mod.rs`
- `stellaux_server/src/domains/auth/api/routes.rs`
- `stellaux_server/src/domains/auth/dto/mod.rs`

Layers modified:

- `api`
- `dto`

Notes:

- Reuse existing `auth/api`, `auth/dto`, `auth/domain`, and `auth/infra` folders.
- Only `api` and `dto` are populated in this migration.
- Move `LoginRequest`, `SignupRequest`, `ForgotPasswordRequest`, and `ResetPasswordRequest` into `dto/mod.rs`.

#### `cart`

New files:

- `stellaux_server/src/domains/cart/mod.rs`
- `stellaux_server/src/domains/cart/api/mod.rs`
- `stellaux_server/src/domains/cart/api/routes.rs`
- `stellaux_server/src/domains/cart/dto/mod.rs`

Layers modified:

- `api`
- `dto`

Notes:

- Move `AddItem` and `UpdateItem` into `dto/mod.rs`.

#### `catalog`

New files:

- `stellaux_server/src/domains/catalog/mod.rs`
- `stellaux_server/src/domains/catalog/api/mod.rs`
- `stellaux_server/src/domains/catalog/api/routes.rs`
- `stellaux_server/src/domains/catalog/dto/mod.rs`

Layers modified:

- `api`
- `dto`

Notes:

- Move `ProductFilter` into `dto/mod.rs`.

#### `checkout`

New files:

- `stellaux_server/src/domains/checkout/mod.rs`
- `stellaux_server/src/domains/checkout/api/mod.rs`
- `stellaux_server/src/domains/checkout/api/routes.rs`
- `stellaux_server/src/domains/checkout/dto/mod.rs`

Layers modified:

- `api`
- `dto`

Notes:

- Move `Address`, `ShippingRatesRequest`, and `CreateSessionRequest` into `dto/mod.rs`.

#### `craft`

New files:

- `stellaux_server/src/domains/craft/mod.rs`
- `stellaux_server/src/domains/craft/api/mod.rs`
- `stellaux_server/src/domains/craft/api/routes.rs`
- `stellaux_server/src/domains/craft/dto/mod.rs`

Layers modified:

- `api`
- `dto`

Notes:

- Move `BasesQuery` and `AccessoriesQuery` into `dto/mod.rs`.

#### `webhooks`

New files:

- `stellaux_server/src/domains/webhooks/mod.rs`
- `stellaux_server/src/domains/webhooks/api/mod.rs`
- `stellaux_server/src/domains/webhooks/api/routes.rs`

Layers modified:

- `api`

Notes:

- `webhooks` currently has no serde DTOs to extract, so only the `api` layer is needed in this pass.

### 3. Rewire crate exports and router composition

Files to modify:

- `stellaux_server/src/lib.rs`
- `stellaux_server/src/server.rs`

Layers modified:

- crate module wiring
- api composition

Notes:

- Add `pub mod domains;` in `lib.rs`.
- Stop using `crate::domain::*` in `server.rs`.
- Mount routes from `crate::domains::<context>::api::routes()` or `crate::domains::<context>::routes()` depending on the final module export style chosen during implementation.

### 4. Remove or reduce the deprecated flat module

Files to modify or delete:

- `stellaux_server/src/domain.rs`
- `stellaux_server/src/domain/account.rs`
- `stellaux_server/src/domain/admin.rs`
- `stellaux_server/src/domain/auth.rs`
- `stellaux_server/src/domain/cart.rs`
- `stellaux_server/src/domain/catalog.rs`
- `stellaux_server/src/domain/checkout.rs`
- `stellaux_server/src/domain/craft.rs`
- `stellaux_server/src/domain/webhooks.rs`

Layers modified:

- deprecated routing module

Notes:

- Preferred end state: delete the old flat files entirely once `server.rs` no longer references them.
- If any temporary compatibility shim is needed, keep it minimal and treat it as transitional only; do not add new business logic to `src/domain/`.

### 5. Add module-level tests where domain logic exists

Files potentially modified:

- The new `domain/` layer files only if pure domain models are introduced during migration.

Notes:

- The current flat files contain API handlers and DTOs, not pure domain models, so this migration is not expected to add `domain/` invariants yet.
- If implementation introduces any pure domain model while reorganizing, add `#[cfg(test)]` unit tests in that module per `AGENTS.md`.

## Changes To `src/common/`

- None expected.

## Risks / Review Points

- `server.rs` is the only current runtime wiring point for these handlers, so router imports must be updated consistently.
- `auth/` already has empty DDD folders; implementation should reuse them cleanly instead of introducing conflicting structure.
- The migration should keep handler behavior unchanged while only changing module paths and DTO locations.

## Validation After Implementation

- Run `cargo check`
- Run `cargo test --lib`

