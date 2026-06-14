# Handoff — Unified Supabase identity + DB role authorization

**Date:** 2026-06-08
**Branch:** main
**Status:** ⚠️ Implementation complete but **does not compile yet** — one unresolved
trait/`Send` error on mounting the new async middleware (details below).

## Goal (confirmed with user)
Move authorization to a **single identity provider (Supabase)** + **DB-backed
roles**. Customers and staff all authenticate via Supabase; *authorization* role
comes from a general `user_roles` table (`role` column: customer/support/staff/admin).
This replaces the old split where Supabase tokens were hardcoded to
`Role::Customer` and only HS256 tokens could ever be `Admin`.

## What was changed

| File | Change |
|------|--------|
| `src/common/jwt.rs` | `Role` enum extended: `Customer, Support, Staff, Admin`; added `Role::from_db_str` (unknown/absent → `Customer`, least privilege). |
| `src/entity/user_roles.rs` (new) | sea-orm entity: `user_id` (PK, = Supabase `auth.users.id`), `role: String`, timestamps. One row per user. |
| `src/entity/mod.rs` | Registered `user_roles` module + `UserRoles` in prelude. |
| `src/migration/m20260608_000001_user_roles.rs` (new) | Creates `user_roles`. **No** cross-schema FK to `auth.users` (Supabase-managed, absent in local/CI). |
| `src/migration/mod.rs` | Registered the new migration in the apply vec. |
| `src/common/roles.rs` (new) | `roles::lookup(db, user_id) -> AppResult<Role>`; absent row → `Customer`. |
| `src/common.rs` | Registered `pub mod roles;`. |
| `src/common/auth.rs` | `require_supabase_auth` now resolves role via `roles::lookup` (was hardcoded `Customer`). Added shared `supabase_claims()` helper + new `require_supabase_admin` (Supabase + `role == Admin` gate). Added `AuthUser::user_id()`, `role()`, and `ensure_owns(owner_id)` ownership guard (admins bypass). HS256 `require_auth`/`require_admin` kept for service accounts. |
| `src/server.rs` | `protected` group → `require_supabase_auth`; `admin` group → `require_supabase_admin`. Updated routing-group doc comments. |

## ⛔ Open blocker — compile error
Mounting `require_supabase_auth` / `require_supabase_admin` fails with:

```
error[E0277]: the trait bound `FromFn<...>: Service<...>` is not satisfied
  --> src/server.rs:124 (and :133)
  = note: there are multiple different versions of crate `axum` in the dependency graph
```

### Diagnosis so far (what we ruled out)
- The old HS256 `require_auth`/`require_admin` (sync body) **compile fine** against
  the current tree → the issue is specific to the new async middleware.
- Bypassing `roles::lookup` → still fails.
- Bypassing `verify_supabase` too (trivial `Send` body) → **still fails**.
- Signatures are byte-identical to the working `require_auth`
  `(State<AppState>, Request, Next) -> AppResult<Response>`.
- `cargo tree` confirms **two axum versions**: `0.8.9` (direct) and `0.7.9` (via
  `axum-prometheus 0.7.0`). Our code uses 0.8.9. This split is pre-existing and the
  HS256 middleware tolerates it, so it's likely the generic hint, not the root cause
  — but it has NOT been definitively cleared.

### Key insight
In the baseline, `require_supabase_auth` was **never mounted**, so its future's
`Send`/trait obligations were checked for the *first time* by this change. The
working theory is a `!Send` future somewhere in the Supabase verify path
(`jwt::verify_supabase` / `JwksCache`) — BUT the trivial-body test (which removed
that path) still failed, which is unexpected and unresolved.

### Next debugging steps (in order)
1. Read the full untruncated error: `cargo check 2>&1 | less` and open the
   `long-type-*.txt` file it references to see the concrete `_` type in `FromFn<…>`.
2. Prime suspect: the async helper `supabase_claims(state: &AppState, req: &Request)`
   that borrows `&req` across the `.await`. Try **inlining** its logic directly into
   each middleware (no borrowing async helper) and re-check.
3. If still failing, add an explicit assertion to locate the non-`Send` value:
   wrap the body future and `fn assert_send<T: Send>(_: &T){}`.
4. Investigate the axum 0.7/0.8 split for real: confirm `middleware::Next` and
   `extract::Request` in `auth.rs` resolve to 0.8.9 (they should). Consider bumping
   `axum-prometheus` to a 0.8-compatible version to eliminate the dual-version noise.

## Remaining work after compile is fixed
- **Per-resource ownership**: handlers are all stubs today. When the DB layer lands,
  every handler that loads a resource by path/query id must call
  `user.ensure_owns(owner_id)` (or filter `where user_id = user.user_id()`).
  See `account.rs::get_order` — currently echoes the id with a `_todo`.
- **Granting admins**: insert a `user_roles` row (`role = 'admin'`) keyed by the
  Supabase user id. No UI/endpoint for this yet.
- **Config**: `protected`/`admin` now require `SUPABASE_JWKS_URL` set (else 500).
- HS256 `login`/`signup` in `src/domain/auth.rs` are still stubs returning `null`.

## Related SQL review (separate thread, already applied by user)
- `shared/models/inventory.sql` — fixed `{}`→`()`, `change` composite-type →
  `adjustment_id` FK, added `channel` CHECK.
- `shared/models/user.sql` / `guest.sql` — `user`→`users` (reserved word),
  `password`→`password_hash`, dropped contradictory `UNIQUE` on guest email,
  removed circular `order_id` NOT NULL. Still missing: `session` and `orders`
  tables that `guest_profiles` FKs reference.
