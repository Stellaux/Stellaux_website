# Plan: Finish Supabase RBAC Implementation by Resolving Middleware Compile Failure

## Bounded Context Affected

- `auth`

## Cross-Cutting Areas Affected

- `stellaux_server/src/common/`
- `stellaux_server/src/server.rs`
- `stellaux_server/Cargo.toml`

## Goal

Complete the unified Supabase identity + DB-backed RBAC work by fixing the compile blocker on the protected/admin middleware path, while preserving the intended authorization model:

- all human users authenticate with Supabase
- authorization roles resolve from `user_roles`
- protected routes accept any valid Supabase user
- admin routes require `role == admin`

The current tree already reflects that intent, but `require_supabase_auth` and `require_supabase_admin` do not mount successfully through `axum::middleware::from_fn_with_state`.

## Current Blocker

`cargo check` fails when mounting:

- `require_supabase_auth`
- `require_supabase_admin`

at:

- `stellaux_server/src/server.rs`

with `E0277` (`FromFn<...>: Service<...> not satisfied`) and a note about multiple `axum` versions in the dependency graph.

Based on the current code and handoff, there are two plausible causes that should be investigated in this order:

1. The async helper path in `stellaux_server/src/common/auth.rs`, especially `supabase_claims(state: &AppState, req: &Request)`, may be creating a future shape that `from_fn_with_state` cannot use cleanly.
2. The project currently pulls both `axum 0.8.x` and `axum 0.7.x` (`axum-prometheus = 0.7`), which may be surfacing as a middleware trait mismatch even if the business logic is correct.

## Implementation Scope

### 1. Refactor the Supabase middleware flow to remove borrowed request/state across await points

Files to modify:

- `stellaux_server/src/common/auth.rs`

Layers modified:

- `common auth middleware`

Plan:

- Refactor `require_supabase_auth` and `require_supabase_admin` so the async verification path does not depend on an async helper that borrows `&Request` across `.await`.
- Prefer one of these patterns:
  - extract the bearer token up front into an owned `String`, then call a helper that accepts owned data or plain references that do not outlive the await boundary
  - inline the Supabase verification/role lookup flow directly into each middleware if that is the clearest way to satisfy Axum’s middleware bounds
- Keep the external behavior unchanged:
  - missing bearer token → `Unauthorized`
  - missing JWKS config → `Internal`
  - absent `user_roles` row → `Customer`
  - admin middleware still rejects non-admin callers with `Forbidden`

Notes:

- Preserve `AuthUser::user_id()`, `role()`, and `ensure_owns()`; they are part of the approved RBAC design.
- Do not reintroduce hardcoded `Role::Customer` behavior for verified Supabase callers.

### 2. If the middleware still fails, replace the `from_fn_with_state` path with a cleaner Axum-compatible auth gate

Files to modify:

- `stellaux_server/src/common/auth.rs`
- `stellaux_server/src/server.rs`

Layers modified:

- `common auth middleware`
- `api composition`

Plan:

- If the refactor in step 1 is not sufficient, implement the same authorization behavior using an alternative Axum-compatible pattern that avoids the failing `FromFn` instantiation.
- Acceptable options include:
  - a dedicated middleware layer/service wrapper for auth
  - a route-layer compatible adapter that keeps the authorization behavior in `common/auth.rs`

Requirements:

- `server.rs` must continue to express the route-group split clearly:
  - public
  - webhooks
  - protected
  - admin
- `api/` handlers must continue to rely on `AuthUser` instead of calling infra/auth code directly.

### 3. Align the Axum dependency graph if middleware refactoring alone does not clear the error

Files to modify:

- `stellaux_server/Cargo.toml`
- potentially `stellaux_server/src/server.rs`

Layers modified:

- `dependency/runtime wiring`

Plan:

- Reconcile the `axum` version split introduced by `axum-prometheus`.
- Preferred direction:
  - move all HTTP/router-facing dependencies onto the same Axum major/minor line used by the application
- If `axum-prometheus` cannot be aligned cleanly:
  - replace it with a metrics integration compatible with the active Axum version while keeping the `/metrics` endpoint behavior intact

Notes:

- This should be treated as a targeted compatibility fix, not a broad dependency upgrade sweep.
- Keep the existing metrics endpoint contract in `server.rs`.

### 4. Add focused tests for the RBAC helpers that now carry policy

Files to modify:

- `stellaux_server/src/common/jwt.rs`
- `stellaux_server/src/common/auth.rs`
- `stellaux_server/src/common/roles.rs` if helpful

Layers modified:

- `common auth policy`

Plan:

- Add unit tests for logic that encodes authorization policy and can be tested without external services:
  - `Role::from_db_str`
  - `AuthUser::ensure_owns()`
- If a small pure helper is introduced during the middleware refactor, add tests for that helper as well.

Notes:

- These are not domain-layer invariants under `src/domains/*`, but they are important guardrails for the new RBAC behavior.

## New Files Expected

- None required by default.

If a small helper module or test support file is needed in `src/common/`, keep it narrowly scoped and document why it is preferable to expanding `auth.rs`.

## Changes To `src/common/`

- `src/common/auth.rs` is expected to be the primary implementation file.
- `src/common/jwt.rs` may gain tests or small helper adjustments if needed for the middleware refactor.
- `src/common/roles.rs` should remain the single DB role lookup entry point.

## Things That Must Not Change

- Do not add new code to `stellaux_server/src/domain/`.
- Do not revert the unified Supabase identity design.
- Do not fall back to HS256 admin tokens for human admin access.
- Do not bypass `user_roles` for authorization decisions.

## Validation After Implementation

1. Run `cargo check`
2. Run `cargo test --lib`
3. If dependency alignment changes, ensure the `/metrics` route is still mounted from `server.rs`

## Open Review Points

- If step 1 fixes the issue without dependency changes, prefer that smaller fix.
- If the dual-`axum` graph is the real blocker, dependency alignment is justified because this is infrastructure correctness, not scope creep.
- If local disk pressure blocks `cargo test --lib` again, capture that explicitly in the developer handoff, but only after `cargo check` is green so the middleware fix is proven.

