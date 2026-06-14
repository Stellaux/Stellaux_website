# Auth Contract

> **Audience:** Stellaux backend (`stellaux_server`), the internal ops dashboard (separate
> repo), and the storefront. This is the **shared interface** for authentication and
> authorization — identity model, token formats, route protection, roles, and the caller
> contract every domain depends on. All surfaces build to this contract.
>
> **Source of truth for scope:** [REQUIREMENTS.MD §2](REQUIREMENTS.MD#2-actors--roles).
> **Source of truth for the shapes below:** `src/common/{auth,jwt,roles}.rs`, the route wiring
> in `src/server.rs`, and `m20260608_000001_user_roles.rs`. Conventions (base path, error
> envelope) are shared with [Catalog_Contract.md §1](Catalog_Contract.md#1-conventions).
>
> **Last updated:** 2026-06-14

---

## 1. Identity vs authorization

Two separate concerns, two separate owners — **do not conflate them**:

| Concern | Owner | Where |
|---|---|---|
| **Identity** — who the caller is | **Supabase Auth** (`auth.users`) | external; verified via RS256 + JWKS |
| **Authorization** — what they may do | **our `user_roles` table** | internal Postgres, keyed by the Supabase user id |

Supabase is the **single identity provider** for every human user (customers and staff alike).
A verified Supabase token gives us the user id (`sub`); we then look up that id in `user_roles`
to get the authorization role. **A user with no `user_roles` row is a plain `customer`** — the
default is least privilege, so elevation is always an explicit DB write and a missing/malformed
row can never escalate.

`user_roles` deliberately has **no FK** to `auth.users` (that schema is Supabase-managed and may
not exist in local/CI DBs); integrity is enforced at the auth layer, which only ever writes a
`sub` taken from a verified Supabase token.

---

## 2. Token formats

| | HS256 (internal) | RS256 (Supabase) |
|---|---|---|
| Issuer | this service (`jwt::issue`) | Supabase Auth |
| Key | symmetric `JWT_SECRET` | Supabase JWKS (cached, TTL) |
| `iss` / `aud` | `stellaux-api` / `stellaux-clients` (configurable) | Supabase project / `authenticated` |
| Verified by | `require_auth`, `require_admin` | `require_supabase_auth`, `require_supabase_admin` |
| **Mounted on any route today** | **No** — see [§7 Gaps](#7-gaps--drift) | **Yes** — all protected + admin routes |

**Practical consequence for all clients (incl. the dashboard):** to call any protected or admin
endpoint you must present a **Supabase access token**:

```
Authorization: Bearer <supabase_access_token>
```

The HS256 path is implemented (`jwt::issue`/`verify`, `require_auth`/`require_admin`) but is
**reserved for service-owned / transitional flows** and is not wired into the router. Don't
build the dashboard against it.

> If `SUPABASE_JWKS_URL` is unset, the protected and admin groups return **500** (not 401) —
> the server cannot verify identity at all. JWKS must be configured in every real environment.

---

## 3. Route protection matrix

From `src/server.rs`. This is the authoritative map the dashboard and storefront must respect.

| Group | Middleware | Token required | Routes |
|---|---|---|---|
| **Public** | none | none | `/healthz`, `/readyz`, `/metrics`, `/api/v1/catalog/*`, `/api/v1/craft/*`, `/api/v1/auth/*`, `/storage/*` (local backend) |
| **Webhooks** | none (handler verifies signature) | none | `/api/v1/webhooks/*` |
| **Protected** | `require_supabase_auth` | any valid Supabase JWT | `/api/v1/cart/*`, `/api/v1/checkout/*`, `/api/v1/account/*` |
| **Admin** | `require_supabase_admin` | Supabase JWT **+ role == `admin`** | `/api/v1/admin/*` |

Notes:
- **Admin gate is `admin` only.** `staff` and `support` do **not** pass `require_supabase_admin`
  today, even though they are real roles (see [§7 Gaps](#7-gaps--drift)). Any finer-grained
  staff/support access must be enforced per-handler via the `AuthUser` extractor.
- Webhooks are unauthenticated at the transport layer; each handler verifies the provider
  signature over the raw body ([Order_Contract.md §3.4](Order_Contract.md#34-webhooks--apiv1webhooks)).

---

## 4. Roles

`Role` is a closed enum (`src/common/jwt.rs`), serialized lowercase. Stored as text in
`user_roles.role`; unknown/absent values degrade to `customer`.

| Role | Meaning | Gains access via |
|---|---|---|
| `customer` | Authenticated shopper (default) | Protected group; owns only their own resources |
| `support` | Support staff | per-handler checks only (no group gate yet) |
| `staff` | Operations staff | per-handler checks only (no group gate yet) |
| `admin` | Privileged operator | Admin group + bypasses ownership guard |

**Role elevation** is an `admin`-only action that writes a `user_roles` row for a target user id
(audit-logged). There is no self-service elevation. The unauthenticated `guest` from
REQUIREMENTS is not a stored role — it is simply the absence of a token (Public group).

### 4.1 `user_roles` schema

| Column | Type | Notes |
|---|---|---|
| `user_id` | uuid pk | the Supabase `auth.users.id` (`sub`) |
| `role` | text | `customer` \| `support` \| `staff` \| `admin` |
| `created_at` / `updated_at` | timestamptz | |

Only elevated users need a row; absence ⇒ `customer`.

---

## 5. Caller contract (normalized `Claims`)

After any auth middleware runs, downstream handlers see a single normalized claim set via the
`AuthUser` extractor — **never raw provider JWT internals**. This is the stable contract domains
depend on:

| Field | Type | Source |
|---|---|---|
| `sub` (`user_id()`) | uuid | Supabase user id |
| `role` (`role()`) | Role | resolved from `user_roles` |
| `iss` | string | issuer |
| `aud` | string | audience |
| `iat` / `exp` | unix seconds | token times |

### 5.1 Ownership guard (mandatory)

Any handler that loads a user-owned resource **by a path/query id** (rather than by the caller's
own id) must call `AuthUser::ensure_owns(owner_id)`:

- caller owns the resource (`sub == owner_id`) → allowed
- caller is `admin` → allowed (vertical privilege)
- `staff` / `support` / other `customer` → **403 Forbidden** (they do *not* bypass)

Alternatively, scope the query by `user_id` directly (e.g. `where user_id = user.user_id()`).
This rule holds regardless of which token type authenticated the caller. (Covers
[REQUIREMENTS §5 ownership rule](REQUIREMENTS.MD#5-cross-cutting-rules).)

---

## 6. Auth domain endpoints — `/api/v1/auth` (public)

Email/password flows. **These are stubs today** (`_todo` placeholders) — shapes are the contract
to build to.

| Method · Path | Body | Result |
|---|---|---|
| `POST /login` | `{ email, password }` | `{ token, user }` on success; 401 on bad credentials |
| `POST /signup` | `{ email, password, display_name? }` | `{ user_id }`; 409 if email exists |
| `POST /forgot-password` | `{ email }` | `{ sent: true }` (always, to avoid account enumeration) |
| `POST /reset-password` | `{ token, new_password }` | 200 on valid token; 400/401 on invalid/expired |

> **Unresolved architectural question — see [§7](#7-gaps--drift).** Supabase is declared the
> single identity provider, yet these endpoints do local password verification against
> `public.users.password_hash` and (per the stub) would mint an **HS256** token — which the
> protected/admin groups don't accept. Until resolved, the dashboard must authenticate through
> **Supabase Auth directly** (GoTrue `signInWithPassword`) and use the returned Supabase access
> token, **not** `/api/v1/auth/login`.

---

## 7. Gaps & drift

Resolve before auth is wired end-to-end; these directly affect the dashboard.

1. **HS256 path is unmounted.** `require_auth`/`require_admin` and `jwt::issue` exist but no
   route group uses them; everything protected uses `require_supabase_*`. So a token from
   `/api/v1/auth/login` (HS256) cannot reach `/cart`, `/checkout`, `/account`, or `/admin`.
   **Decide:** either (a) `/auth` proxies to Supabase Auth and returns a Supabase session, or
   (b) protected groups also accept internal HS256 for the transitional path. (a) matches the
   "Supabase is the single identity provider" stance.
2. **`staff` / `support` have no route gate.** Only `admin` passes the admin group; the other
   elevated roles exist but are unreachable except via per-handler `AuthUser` checks. Add
   role-aware middleware (e.g. `require_role(min)`) if staff/support need scoped admin routes.
3. **Auth stub references the wrong tables.** Comments mention `profiles` / `auth.users`; the
   migrated table is `public.users` (with `password_hash`). Reconcile naming — and clarify
   whether `public.users.password_hash` is even used if Supabase owns identity.
4. **Password reset ownership.** `forgot/reset-password` stubs imply a local token +
   `email_tokens` + Resend flow, but if Supabase owns identity, reset is Supabase's flow
   (REQUIREMENTS UC-7 notes Supabase handles it). Pick one; don't run two reset systems.
5. **No logout / refresh / session-introspection endpoints**, and no `jti` / revocation list
   (noted as future in `jwt.rs`). Token refresh is currently the client's responsibility via
   Supabase. Document the expected refresh cadence for the dashboard.

---

## 8. Configuration (env)

| Var | Default | Purpose |
|---|---|---|
| `JWT_SECRET` | (required) | HS256 signing secret (internal tokens) |
| `JWT_EXPIRY_SECONDS` | `3600` | internal token TTL |
| `JWT_ISSUER` | `stellaux-api` | `iss` for internal tokens |
| `JWT_AUDIENCE` | `stellaux-clients` | `aud` for internal tokens |
| `SUPABASE_JWKS_URL` | (none → Supabase verification disabled, protected/admin 500) | JWKS endpoint |
| `SUPABASE_AUDIENCE` | `authenticated` | expected `aud` on Supabase tokens |
| `SUPABASE_ISSUER` | (optional) | expected `iss`, e.g. `https://<proj>.supabase.co/auth/v1` |
| `JWKS_TTL_SECONDS` | `3600` | how long cached JWKS keys are trusted |

---

## 9. Related

- [REQUIREMENTS.MD](REQUIREMENTS.MD) — actors/roles, UC-7 (authentication)
- [Catalog_Contract.md](Catalog_Contract.md) · [Order_Contract.md](Order_Contract.md) — the contracts these tokens authorize
- [html_documentations/auth-contract.md](html_documentations/auth-contract.md) — earlier narrative auth notes (superseded by this doc)
