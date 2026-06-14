# Stellaux — Architecture

> Scope, actors, modules, and use-case flows live in [REQUIREMENTS.MD](REQUIREMENTS.MD)
> (the source of truth). This document covers **system architecture only** — runtime
> topology, sources of truth, and layering. It deliberately does not restate the product
> goal or the module list.

---

## System topology

```
                            ┌──────────────────────────────┐
                            │  Cloudflare Workers (Edge)   │
                            │  TanStack Start storefront   │
                            └──────────────┬───────────────┘
                                           │  REST /api/v1/*
                            ┌──────────────┴───────────────┐
                            │   stellaux_server            │
                            │   Rust · Axum · SeaORM        │
                            └──────────────┬───────────────┘
        ┌──────────────────────────────────┼──────────────────────────────────┐
        ▼                                  ▼                                  ▼
┌──────────────┐                  ┌──────────────┐                  ┌──────────────┐
│   Supabase   │                  │    Stripe    │                  │    Shippo    │
│   Postgres   │  ◀──webhooks──▶  │  Checkout +  │  ◀──webhooks──▶  │   Labels +   │
│   + Auth     │                  │  PaymentInt. │                  │   Tracking   │
│   + Storage  │                  └──────────────┘                  └──────────────┘
└──────┬───────┘                                                   ┌──────────────┐
       ├── (future) Etsy / eBay sync workers                       │    Resend    │
       └──                                                         │ (txn emails) │
                                                                   └──────────────┘
```

## Sources of truth

| Concern | Authority | Mirrored where |
|---|---|---|
| Catalog · inventory · customers | **Supabase Postgres** | — (origin) |
| Payment state | **Stripe** | Postgres, via webhook |
| Fulfillment (label + tracking) | **Shippo** | Postgres, via webhook |

The first-party website **and** the internal dashboard consume the *same* backend business
contracts that marketplace operations use; surfaces must not fork parallel models.

## Backend layering (target)

Per [bounded-context-map.md](html_documentations/bounded-context-map.md):

```
api/  ──►  application/  ──►  domain/   ◄── infra/ (implements domain ports)
   may depend on dto/ + common
```

- `api/` may depend on `application/`, `dto/`, and `common`.
- `application/` may depend on `domain/`.
- `infra/` implements `domain` ports; `domain/` defines business-facing contracts/models.
- `src/common/` owns cross-cutting platform concerns (config, bootstrap, auth middleware,
  JWT/JWKS, storage, error translation, audit + idempotency) and **no** business rules.

**Current state:** only the `auth` and `webhooks` contexts realize the full four-layer
structure; the remaining domains expose `api/` + `dto/` today and grow inner layers as
business logic lands. See [REQUIREMENTS.MD §4](REQUIREMENTS.MD#4-modules-bounded-contexts).

## Related references

- [API_Guidelines.md](API_Guidelines.md) — REST/OpenAPI conventions, versioning, utoipa
- [backend_integration.md](backend_integration.md) — full backend build plan and schema
- [html_documentations/](html_documentations/) — context map, auth contract, ownership matrices, observability
