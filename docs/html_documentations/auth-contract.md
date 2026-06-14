# Auth Contract

## Identity Model

Phase 0 locks the project onto a unified human identity model:

- human users authenticate through Supabase
- authorization roles are resolved from internal Postgres data
- route middleware verifies caller identity and stamps normalized claims into the request

Server-issued JWTs remain allowed only for service-owned flows and transitional cases. They are not the long-term primary path for human admin access.

## Responsibility Split

### `src/common/auth.rs`

Owns transport-level concerns:

- bearer token extraction
- Axum middleware
- request extensions / `AuthUser`
- Supabase JWT verification entrypoint
- server-JWT verification entrypoint

### `src/common/jwt.rs`

Owns token cryptography and JWKS behavior:

- token issue/verify helpers
- JWKS cache
- low-level claim parsing

### `src/common/roles.rs`

Owns authorization-role lookup from internal storage.

### `src/domains/auth/*`

Owns business auth flows:

- login/session contract
- signup
- forgot/reset password
- future user-facing auth policies and adapters

## Caller Contract

Downstream domains should depend on normalized caller identity, not on JWT implementation details.

The stable caller contract is:

- `user_id`
- `role`
- `identity provider`
- intended audience when relevant

## Role Contract

Current roles:

- `customer`
- `support`
- `staff`
- `admin`

Role lookup defaults to least privilege. Missing or malformed role rows must never escalate privileges.

## Ownership Rule

Handlers that load user-owned resources by id must either:

- scope the query by `user_id`, or
- call an ownership guard such as `ensure_owns(owner_id)`

This rule applies regardless of whether the caller originated from Supabase or a service-issued JWT.
