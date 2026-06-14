# Table Ownership Matrix

## Purpose

This matrix defines which bounded context should own schema decisions for key table families during Phase 0 and the next implementation phases.

| Table family | Owning context | Notes |
|--------------|----------------|-------|
| `users` | `auth` | human identity records and login-facing profile identity |
| `user_roles` | `auth` | authorization roles resolved after identity verification |
| `session` | `auth` | session/browser identity support when used |
| `email_tokens` | `auth` | password reset and email verification style flows |
| `guest_profiles` | `account` with `auth` coordination | guest-to-user conversion touches both contexts |
| webhook event log | `webhooks` | technical receipt, replay safety, dedupe |
| audit log | cross-cutting with `common` runtime support | business event trail consumed across contexts |
| idempotency records | cross-cutting with `common` primitives | persisted per owning flow, not globally improvised |
| `products`, `product_variants`, `product_media` | `catalog` | canonical sellable catalog model |
| inventory tables | `inventory` | on-hand, reserved, adjustments, alerts |
| marketplace channel mapping tables | `catalog` + `orders` coordination | listing-to-product and order-source normalization |
| `orders`, `order_items` | `orders` | internal order model regardless of source |
| shipment tables | `shipment` / `fulfillment` | labels, tracking, delivery |

## Ownership Rule

If a table is primarily a transport or operational concern, it belongs to the context that normalizes and governs that concern. Cross-context consumers may read it through contracts, but should not redefine it independently.

## Exposure Rule

- Ownership does not imply direct Supabase client exposure.
- Tables in `public` still require explicit RLS posture.
- Tables in `private` are internal by default and should be accessed through backend contracts.
