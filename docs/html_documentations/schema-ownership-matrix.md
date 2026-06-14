# Schema Ownership Matrix

## Core Principle

Schema ownership follows business ownership, not UI ownership.

## Foundational Schemas

| Area | Canonical home | Primary owner |
|------|----------------|---------------|
| identity and roles | `shared/models/user.sql` and related auth SQL | `auth` |
| webhook event persistence | future webhook SQL artifact | `webhooks` |
| audit persistence | future audit SQL artifact | cross-cutting operational support |
| idempotency persistence | future operational SQL artifact | cross-cutting operational support |
| catalog and media | `shared/models/catalog.sql` | `catalog` |
| inventory | `shared/models/inventory.sql` | `inventory` |
| orders and line items | `shared/models/orders.sql`, `shared/models/order_items.sql` | `orders` |

## Schema Boundary Rule

- `public` is reserved for tables that may need controlled Supabase-client visibility later.
- Every `public` business table must have explicit RLS posture; deny-by-default is the baseline.
- `private` is for internal operational tables that should not be exposed through Supabase PostgREST.

## Review Rule

Any schema change proposed in a later phase should answer two questions before implementation:

1. Which bounded context owns this table or column?
2. Which external system, if any, remains authoritative for the corresponding real-world event?
