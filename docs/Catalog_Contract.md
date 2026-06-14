# Catalog Contract

> **Audience:** Stellaux backend (`stellaux_server`), the internal ops dashboard (separate
> repo), and the storefront. This is the **shared interface** for catalog data — schema,
> API surface, and expected flows. All three surfaces must build to this contract; none may
> fork a parallel model.
>
> **Source of truth for scope:** [REQUIREMENTS.MD](REQUIREMENTS.MD). **Source of truth for
> the shape below:** the migrated SQL under `supabase/migrations/` (mirrored from
> `shared/models/`). Where the live schema and a consumer disagree, the live schema wins and
> the consumer is the bug.
>
> **Last updated:** 2026-06-14

---

## 1. Conventions

| Concern | Rule |
|---|---|
| Base path | `/api/v1` |
| Catalog routes mount | `/api/v1/catalog/*`, `/api/v1/craft/*`, admin at `/api/v1/admin/*` |
| IDs | `uuid` (server-generated, `gen_random_uuid()`) |
| Money | integer **cents**, `bigint` (`price_cents`, `cost_cents`); never floats |
| Timestamps | `timestamptz`, RFC 3339 UTC |
| Auth | `Authorization: Bearer <token>`; role resolved server-side ([REQUIREMENTS §2](REQUIREMENTS.MD#2-actors--roles)) |
| Pagination (query) | `limit` (default 20, clamped 1–100), `offset` (default 0) |
| List envelope | `{ "items": [...], "total": n, "limit": n, "offset": n }` |
| Error envelope | `{ "error": { "message": string, "code": number, "details"?: object } }` |
| Error codes | 400 bad request / validation, 401 unauthorized, 403 forbidden, 404 not found, 409 conflict, 500 internal (scrubbed) |

Reads of catalog data are public (anon allowed). All writes require `admin` (catalog/inventory)
or `staff` where noted. Gating is server-side; never trust client role state.

---

## 2. Schema

All tables in `public` unless marked `private`. RLS is enabled on every table; the Rust API
(service role) is the primary business interface — "public read" below means the API exposes
it to anon callers, not that PostgREST is open.

### 2.1 `categories`

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `slug` | text unique | URL key, e.g. `rings` |
| `name` | text | |
| `size_unit` | text | `none` \| `inch` \| `ring_us` \| `mm` — **drives variant sizing** |
| `sort_order` | int | |
| `created_at` / `updated_at` | timestamptz | |

### 2.2 `category_size_options`

Allowed discrete sizes per category. If a category has rows here, variant `size_value` **must**
match one of them (enforced by trigger, §2.6).

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `category_id` | uuid → categories | cascade |
| `size_value` | numeric(6,2) | |
| `label` | text | optional display label |
| `sort_order` | int | |
| | | unique `(category_id, size_value)` |

### 2.3 `collections`

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `slug` | text unique | |
| `name` | text | e.g. `Vol. I`, `Atelier` |
| `description` | text | |

### 2.4 `products`

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `handle` | text unique | slug used in storefront URLs |
| `name` | text | |
| `description` | text | |
| `category_id` | uuid → categories | **FK, not a string** |
| `default_material` | text | display hint; authoritative material is per-variant |
| `tags` | text[] | default `{}` |
| `status` | text | `draft` \| `active` \| `archived` (public read = `active`) |
| `created_at` / `updated_at` | timestamptz | |

### 2.5 `product_collections`

Join table. PK `(product_id, collection_id)`, both cascade.

### 2.6 `product_variants`

The sellable unit. **Size lives here**, not on the order line.

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `product_id` | uuid → products | cascade |
| `material` | text | authoritative material (`18k Gold`, `Silver`, …) |
| `design` | text | optional |
| `type_label` | text | |
| `size_value` | numeric(6,2) | null iff category `size_unit = none`; else required + validated |
| `sku` | text unique | **canonical internal SKU** (one per sellable unit) |
| `barcode` | text | |
| `price_cents` | bigint ≥ 0 | |
| `cost_cents` | bigint ≥ 0 | optional |
| `weight_grams` | int | required for Shippo rate quotes |
| `dimensions` | text | |
| `status` | text | `active` \| `archived` \| `draft` |
| | | unique `(product_id, material, design, size_value)` |

**Trigger `product_variants_size_check`** (authoritative — dashboard must respect it):
- category `size_unit = none` → `size_value` must be **null**.
- otherwise `size_value` is **required**, and if the category has `category_size_options`, it
  must match one of them. Violations raise `23514`.

### 2.7 `product_media`

Catalog/stock photography. **Distinct from `order_media`** (actual purchased piece — see
[Order_Contract.md](Order_Contract.md)).

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `product_id` | uuid → products | cascade |
| `variant_id` | uuid → product_variants | nullable; cascade |
| `storage_key` | text | path within the media storage backend |
| `kind` | text | `image` \| `video` \| `model_3d` |
| `alt_text` | text | |
| `position` | int | order within the gallery |

### 2.8 `channel_listings`

Maps an internal product/variant to an external marketplace listing. Owned/maintained by the
dashboard + sync workers.

| Column | Type | Notes |
|---|---|---|
| `id` | uuid pk | |
| `channel` | text | `etsy` \| `ebay` \| `website` |
| `external_listing_id` | text | |
| `external_variant_id` | text | nullable |
| `external_sku` | text | nullable |
| `product_id` | uuid → products | |
| `variant_id` | uuid → product_variants | nullable |
| `raw` | jsonb | last raw payload from the channel |
| `status` | text | `active` \| `inactive` \| `error` |
| `last_synced_at` | timestamptz | |
| | | unique `(channel, external_listing_id, external_variant_id)` |

### 2.9 `private.inventory` (availability source)

Availability is read alongside catalog but **owned by the inventory context**. Full detail in
[Order_Contract.md](Order_Contract.md) / inventory docs; the catalog-relevant read is:

| Column | Type | Notes |
|---|---|---|
| `variant_id` | uuid unique → product_variants | |
| `quantity` | int ≥ 0 | on-hand |
| `reserved` | int ≥ 0 | held by open checkouts |
| `available` | int generated | **`quantity - reserved`** (stored generated column) |

> Contract note: the live generated column is `available = quantity - reserved`. The
> `safety_buffer` term from [REQUIREMENTS §5](REQUIREMENTS.MD#5-cross-cutting-rules) is **not
> yet** in this table — see [§6 Gaps](#6-schema-gaps--drift).

---

## 3. API — Public catalog

### `GET /api/v1/catalog/products`

List active products. **Query:** `category`, `material`, `collection`, `sort`, `limit`,
`offset`. **200:** list envelope of [Product summary](#33-response-shapes). Filtering by
`material` resolves through `product_variants.material`.

### `GET /api/v1/catalog/products/{handle}`

**200:** full [Product detail](#33-response-shapes) (product + variants + media + per-variant
availability). **404:** unknown/inactive handle.

### `GET /api/v1/catalog/collections`
**200:** `{ "items": [Collection] }`.

### `GET /api/v1/catalog/categories`
**200:** `{ "items": [Category] }` including `size_unit` and size options (so the dashboard and
storefront render the correct size selector).

### 3.3 Response shapes

```jsonc
// Product summary (list)
{
  "id": "uuid", "handle": "atelier-rope-chain", "name": "Rope Chain",
  "category": { "slug": "necklaces", "name": "Necklaces", "size_unit": "inch" },
  "default_material": "18k Gold",
  "status": "active",
  "from_price_cents": 18500,           // min active variant price
  "primary_image": { "storage_key": "...", "alt_text": "..." },
  "in_stock": true                     // any variant available > 0
}

// Product detail
{
  "id": "uuid", "handle": "...", "name": "...", "description": "...",
  "category": { "slug": "...", "name": "...", "size_unit": "inch",
                "size_options": [{ "size_value": 18, "label": "18\"" }] },
  "collections": [{ "slug": "atelier", "name": "Atelier" }],
  "tags": ["new"],
  "media": [{ "storage_key": "...", "kind": "image", "alt_text": "...", "position": 0 }],
  "variants": [
    {
      "id": "uuid", "sku": "RC-18K-18", "material": "18k Gold", "design": null,
      "type_label": "Rope", "size_value": 18,
      "price_cents": 18500, "weight_grams": 7, "dimensions": "...",
      "status": "active", "available": 4
    }
  ]
}
```

---

## 4. API — Craft

Craft products are ordinary `products`/`product_variants` distinguished by craft role, plus a
compatibility graph between bases and accessories.

### `GET /api/v1/craft/bases` · `GET /api/v1/craft/accessories`
**200:** list of craft products (each with its variants), filtered by craft role.

### `GET /api/v1/craft/compatibility/{base_handle}`
**200:** `{ "base": Product, "compatible_accessories": [Product] }`.

> **Backing schema is not yet migrated** — craft role and the compatibility table do not exist
> in the live DB. See [§6 Gaps](#6-schema-gaps--drift). Dashboard work that writes compatibility
> must wait on, or drive, that migration.

---

## 5. API — Admin (writes)

All require `admin` (catalog) / `admin`|`staff` (inventory). Audit-logged.

| Method · Path | Purpose | Body / notes |
|---|---|---|
| `GET /api/v1/admin/products` | List incl. drafts/archived | paginated |
| `POST /api/v1/admin/products` | Create product | see [UpsertProduct](#52-upsertproduct) |
| `GET /api/v1/admin/products/{id}` | Detail incl. variants | |
| `PATCH /api/v1/admin/products/{id}` | Update product | partial |
| `DELETE /api/v1/admin/products/{id}` | Delete/archive | `{ "deleted": id }` |
| `GET /api/v1/admin/inventory` | List inventory levels | |
| `POST /api/v1/admin/inventory` | Adjust on-hand | `{ variant_id, delta, reason, notes? }` → writes `inventory_adjustment` |
| `GET /api/v1/admin/compatibility` | List craft pairs | |
| `POST /api/v1/admin/compatibility` | Add pair | `{ base_product_id, accessory_product_id }` |
| `DELETE /api/v1/admin/compatibility/{...}` | Remove pair | |
| `GET /api/v1/admin/channel-listings` | List channel mappings | |

### 5.2 `UpsertProduct`

> **Contract caveat — the implemented DTO drifts from the migrated schema.** The current
> `UpsertProduct` accepts `{ handle, name, category, material, collection, craft_role,
> craft_base_type }` (strings), but the schema uses `category_id` (FK), `default_material`,
> `product_collections` (join), per-variant `material`, and has **no** `craft_role`/
> `craft_base_type` columns. **Canonical target** for this contract: the dashboard sends
> `category_slug` + `collection_slugs[]` + `default_material`; the server resolves slugs to
> FKs. Variants (with their own `material`/`size_value`/`sku`/`price_cents`) are created via a
> nested array or a sibling `…/variants` call. This DTO must be reconciled — tracked in
> [§6](#6-schema-gaps--drift).

---

## 6. Schema Gaps & Drift

Actionable backlog for keeping backend + dashboard consistent. Resolve before the dependent
dashboard features ship.

1. **Craft has no backing schema.** No `craft_role`/`craft_base_type` columns on `products`
   and no `craft_compatibility` table are migrated, yet the craft endpoints and the admin
   compatibility endpoints + `UpsertProduct` reference them. Add a `craft_compatibility`
   migration and decide where craft role lives (column on `products` vs a `product_roles`
   table) before craft is built.
2. **`UpsertProduct` DTO vs normalized schema** (see [§5.2](#52-upsertproduct)): strings
   (`category`, `material`, `collection`) vs FKs (`category_id`, `product_collections`,
   per-variant `material`). Reconcile to slug-resolved FKs.
3. **`safety_buffer` not in `private.inventory`.** REQUIREMENTS §5 defines
   `available = on_hand − reserved − safety_buffer`; the live generated column omits the
   buffer. Add the column (and fold into the generated expression) or amend the formula.
4. **Stub endpoints return hardcoded data.** `collections`/`categories` currently return
   literal arrays; `products`/`product` return `_todo` placeholders. They must read from the
   tables above to satisfy this contract.

---

## 7. Related

- [Order_Contract.md](Order_Contract.md) — orders, order items, order media, checkout/fulfillment
- [REQUIREMENTS.MD](REQUIREMENTS.MD) — scope, actors, use-case flows
- [API_Guidelines.md](API_Guidelines.md) — REST/OpenAPI conventions
