# Order Contract

> **Audience:** Stellaux backend (`stellaux_server`), the internal ops dashboard (separate
> repo), and the storefront. This is the **shared interface** for orders, order line items,
> and order media — schema, API surface, lifecycle, and expected flows. All three surfaces
> build to this contract.
>
> **Source of truth for scope:** [REQUIREMENTS.MD](REQUIREMENTS.MD). **Source of truth for the
> shape below:** the migrated SQL under `supabase/migrations/`. Conventions (IDs, money,
> errors, pagination, auth) are shared with [Catalog_Contract.md §1](Catalog_Contract.md#1-conventions).
>
> **Last updated:** 2026-06-14

---

## 1. Schema

All tables in `public`; RLS enabled. The Rust API (service role) is the business interface.

### 1.1 `orders`

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `user_id` | uuid → users | nullable |
| `guest_profile_id` | uuid → guest_profiles | nullable |
| — | check | **exactly one** of `user_id` / `guest_profile_id` is set (`order_owner_check`) |
| `order_number` | text unique | human-facing reference |
| `status` | text | `pending` \| `paid` \| `shipped` \| `cancelled` (default `pending`) |
| `currency` | text | default `USD` |
| `subtotal_cents` | bigint | sum of line items |
| `tax_cents` | bigint | |
| `shipping_cents` | bigint | |
| `total_cents` | bigint | `subtotal + tax + shipping` |
| `total` | int | ⚠️ legacy/redundant scalar — see [§5 Gaps](#5-schema-gaps--drift) |
| `shipping_address` | jsonb | snapshot |
| `billing_address` | jsonb | snapshot |
| `placed_at` | timestamptz | default now() |
| `paid_at` | timestamptz | set on `pending → paid` |
| `shipped_at` | timestamptz | set on `paid → shipped` |
| `created_at` / `updated_at` | timestamptz | |

Indexes: `user_id`, `guest_profile_id`, `order_number`.

> **Ownership:** an order belongs to a registered user **or** a guest profile, never both.
> Customer-facing reads scope by `user_id`; guest reads resolve via the guest session.

### 1.2 `order_items`

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `order_id` | uuid → orders | cascade |
| `product_id` | uuid → products | |
| `variant_id` | uuid → product_variants | **nullable** — marketplace rows may not resolve to a variant immediately; fall back to `sku` |
| `sku` | text | purchase-time snapshot (canonical SKU) |
| `name` | text | product name at purchase time |
| `quantity` | int > 0 | |
| `unit_price_cents` | bigint | snapshot at order time |
| `total_cents` | bigint | `unit_price_cents × quantity` |

Indexes: `order_id`, `variant_id`. **Size is not on this row** — it lives on the variant
(`product_variants.size_value`); resolve via `variant_id`.

### 1.3 `order_media`

Photographs of the **actual purchased / custom-crafted piece**, uploaded from the internal
dashboard. Distinct from `product_media` (catalog stock photos). Anchored on `order_item_id`,
which already encodes order + variant + size.

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `order_id` | uuid → orders | cascade |
| `order_item_id` | uuid → order_items | cascade — **the anchor** |
| `variant_id` | uuid → product_variants | denormalized for filter/label |
| `size_value` | numeric(6,2) | snapshot of variant size at capture |
| `storage_key` | text unique | path in the private `order-media` bucket |
| `kind` | text | `image` \| `video` \| `model_3d` |
| `alt_text` | text | |
| `position` | int | slot within the line |
| `is_current` | boolean | active-version pointer (default true) |
| `captured_by` | uuid → users | dashboard operator |
| `created_at` | timestamptz | |

Indexes / invariants:
- `order_media_item_current_idx` — partial `(order_item_id) where is_current` (hot read).
- `order_media_current_slot_uniq` — **unique `(order_item_id, position) where is_current`**:
  exactly one live image per (line, slot).

**Storage bucket `order-media`:** private (no public URL), 10 MB/object, mime allow-list
`image/jpeg|png|webp|avif`. Key convention:
`order-media/{order_id}/{order_item_id}/{position}-{version_uuid}.{ext}`. Reads are served as
short-lived **signed URLs** minted by the Rust API after the ownership guard. Writes come from
the dashboard via the Rust admin API using the service role (bypasses storage RLS). No anon
storage policies exist → closed by default.

---

## 2. Order lifecycle

```
            POST /checkout/session                 Stripe webhook              Shippo tracking
 cart ───────────────────────────►  pending  ──────────────────────►  paid  ──────────────►  shipped
        (reserve inventory,                  (checkout.session.completed:        (track_updated:
         create order pending,                locate by stripe session id,        TRANSIT → shipped)
         create Stripe session)               pending → paid, set paid_at,
                                              decrement on-hand)

        session expired / payment_failed → cancelled (release reservations, cart abandoned)
```

| From | To | Trigger | Side effects |
|---|---|---|---|
| — | `pending` | `POST /api/v1/checkout/session` | reserve inventory; snapshot line items + totals + addresses; create Stripe session |
| `pending` | `paid` | Stripe `checkout.session.completed` | locate order by Stripe session id; set `paid_at`; convert reserved → sale (decrement on-hand); mark cart converted |
| `pending` | `cancelled` | session expired / `payment_intent.payment_failed` | release reservations; mark cart abandoned |
| `paid` | `shipped` | Shippo `track_updated` = TRANSIT, or admin marks fulfilled | set `shipped_at`; queue shipping email |
| `paid`/`shipped` | (refund) | admin refund | restock if pre-fulfillment; queue refund email |

> The webhook **never creates an order from scratch** — it locates the pre-created `pending`
> order. This requires a stored Stripe-session reference on the order; that column is **not
> yet in the live schema** — see [§5 Gaps](#5-schema-gaps--drift).

---

## 3. API

### 3.1 Customer — `/api/v1/account` (auth required, scoped to caller)

| Method · Path | Purpose |
|---|---|
| `GET /orders` | List the caller's orders (paginated) |
| `GET /orders/{order_id}` | Order detail incl. items and **current media** (with signed URLs) |

```jsonc
// GET /api/v1/account/orders/{order_id}
{
  "id": "uuid", "order_number": "STX-10241", "status": "paid",
  "currency": "USD",
  "subtotal_cents": 18500, "tax_cents": 1200, "shipping_cents": 800, "total_cents": 20500,
  "placed_at": "2026-06-14T...", "paid_at": "2026-06-14T...", "shipped_at": null,
  "shipping_address": { ... }, "billing_address": { ... },
  "items": [
    {
      "id": "uuid", "product_id": "uuid", "variant_id": "uuid",
      "sku": "RC-18K-18", "name": "Rope Chain", "size_value": 18,
      "quantity": 1, "unit_price_cents": 18500, "total_cents": 18500,
      "media": [
        { "id": "uuid", "kind": "image", "position": 0, "alt_text": "...",
          "url": "https://.../signed?token=...", "url_expires_at": "2026-06-14T..." }
      ]
    }
  ]
}
```

### 3.2 Checkout — `/api/v1/checkout`

| Method · Path | Body | Result |
|---|---|---|
| `POST /shipping-rates` | `{ cart_id, address }` | 2–3 Shippo service levels w/ cents pricing |
| `POST /session` | `{ cart_id, shippo_rate_id, address }` | reserves inventory, creates `pending` order + Stripe session; returns hosted checkout URL |

### 3.3 Admin — `/api/v1/admin` (admin; some reads `staff`/`support`)

| Method · Path | Purpose |
|---|---|
| `GET /orders` | List/filter (`status`, `source`, paginated) |
| `GET /orders/{order_id}` | Full detail (customer, items, payment, tracking, media) |
| `POST /orders/{order_id}/refund` | Refund (confirmation-gated, audit-logged, restock if pre-fulfillment) |
| `POST /orders/{order_id}/cancel` | Cancel |
| `POST /orders/{order_id}/labels` | Reprint shipping label |
| **`POST /orders/{order_id}/items/{order_item_id}/media`** | **Upload order media** (see §4) |
| `DELETE /orders/{order_id}/media/{media_id}` | Retire a media version |

### 3.4 Webhooks — `/api/v1/webhooks`

`POST /stripe`, `POST /shippo`. Signature verified over the **raw** body; idempotent on
`(source, external_id)`. Drive the lifecycle transitions in §2. Not called by the dashboard.

---

## 4. Order-media flow (internal dashboard)

The dashboard owns capture; the **Rust admin API owns the write** so the storage object and the
DB row commit together and get audit-logged.

**Upload / replace** — `POST /api/v1/admin/orders/{order_id}/items/{order_item_id}/media`
(`multipart` or pre-resolved upload), one transaction:
1. Resolve `order_item_id` → `variant_id`, `size_value`.
2. Write the object to `order-media/{order_id}/{order_item_id}/{position}-{version_uuid}.{ext}`
   (service role).
3. If replacing a slot: `update order_media set is_current = false where order_item_id = $1 and
   position = $2 and is_current`, then insert the new row (`is_current = true`). The partial
   unique index guarantees one live image per slot; old rows remain as history.

> Alternative: the server hands the dashboard a **scoped signed upload URL**
> (`createSignedUploadUrl`) for the computed key; the dashboard PUTs bytes directly, then calls
> back to register the row. Bucket stays private either way.

**Retrieval (efficient).** One batched query per order, then batch-mint signed URLs:

```sql
select oi.id  as order_item_id, oi.variant_id, oi.sku, oi.name,
       om.id  as media_id, om.storage_key, om.kind, om.alt_text, om.position
from public.order_items oi
left join public.order_media om
       on om.order_item_id = oi.id and om.is_current
where oi.order_id = $1
order by oi.id, om.position;
```

`order_media_item_current_idx` makes this an index fan-out; the API caches signed URLs
in-process for `ttl − margin`.

**Cleanup.** Non-current rows are swept by a scheduled job after a grace window (~30 days):
collect their `storage_key`s, `remove([...])` from the bucket, then delete the rows.

---

## 5. Schema Gaps & Drift

Resolve before the dependent flows ship.

1. **No Stripe/Shippo/fulfillment columns on `orders`.** The pending-at-checkout model
   ([REQUIREMENTS UC-4/UC-5](REQUIREMENTS.MD#6-core-use-cases--flows)) requires the webhook to
   locate the order by a stored Stripe session id, and tracking to be mirrored back. The live
   `orders` table has **none** of: `stripe_checkout_session_id` (unique),
   `stripe_payment_intent_id`, `tracking_number`/`tracking_carrier`/`tracking_url`,
   `shipping_method`, `source` + `external_order_id`, `fulfillment_status`. Add them before
   checkout/fulfillment is wired.
2. **`status` enum is narrow.** Live values are `pending|paid|shipped|cancelled`. The flows
   reference `delivered` (Shippo DELIVERED) and `refunded`. Decide whether these are
   `status` values or a separate `fulfillment_status` (the latter matches REQUIREMENTS).
3. **`orders.total` (int) is redundant** with `total_cents` (bigint) and ambiguous in unit.
   Drop it or document it as a deprecated mirror; the dashboard must not rely on it — use
   `total_cents`.
4. **Order media endpoints are contract-only.** `order_media` table + bucket are migrated
   (`20260613000100_order_media.sql`), but the admin upload/read endpoints in §3.3/§4 are not
   yet implemented.

---

## 6. Related

- [Catalog_Contract.md](Catalog_Contract.md) — products, variants, media, channels, inventory
- [REQUIREMENTS.MD](REQUIREMENTS.MD) — scope, actors, UC-4/UC-5/UC-6 flows
- [backend_integration.md](backend_integration.md) — Stripe/Shippo build plan
