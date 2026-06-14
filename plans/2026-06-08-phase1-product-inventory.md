# Plan: Phase 1 — Product & Inventory Backbone (schema-first)

> Source of truth: `shared/models/*.sql`. This plan **extends and corrects** the
> existing SQL there. SeaORM entities and the `src/migration/*.rs` runner are
> **explicitly deferred** (see "Deferred"), per the directive to disregard the
> current SeaORM scripts for now. SQL is canonical; entities get regenerated
> from it later.

## Bounded Context Affected

- `catalog` (primary)
- `admin` (product maintenance — surface only; deep work is later)
- `craft` only if modular compatibility affects sellable SKUs (out of scope here)

## Cross-Cutting Areas Affected

- `shared/models/` — canonical SQL schema (the bulk of this phase)
- `stellaux_server/src/domains/catalog/` — DDD scaffolding to read the schema
- `stellaux_server/src/common/` — only if a shared `Money`/value type is justified

## Goal

Land the canonical catalog + inventory schema that marketplace operations need:
products, **variants with a category-governed size axis and a material+design
type axis**, categories, collections, media, per-variant inventory, and the
external-channel listing mapping. Everything expressed first as SQL in
`shared/models/`, then surfaced through a clean `catalog` domain.

---

## The variant model (the core design decision)

A **sellable SKU = `product` × `type` × `size`**, stored as one flat
`product_variants` row (one row per orderable combination — matches
`order_items.sku` and per-variant `inventory`).

Two axes, exactly as described:

### Axis 1 — "type" = material + design
Decomposed into two queryable columns plus a denormalized display label:
- `material` — e.g. `18k light gold`, `vermeil`, `sterling silver`
- `design` — e.g. `chain`, `rope`, `curb`, `solid`
- `type_label` — e.g. `"18k light gold chain"` (for UI + marketplace titles)

Product-level `material` (in the current draft) becomes an optional
`default_material` on `products` for filtering/display; the **authoritative
per-SKU material lives on the variant**.

### Axis 2 — size, with the unit governed by the product's category
This is the "which option depends on the jewelry type" requirement. The unit is
a property of the **category**, not the variant:

| Category | `size_unit` | Example `size_value` |
|----------|-------------|----------------------|
| ring | `ring_us` | `7`, `7.5` (numbered scale) |
| necklace / pendant | `inch` | `16`, `18`, `20`, `24` |
| bracelet | `inch` | `6.5`, `7`, `7.5` |
| earring / charm | `none` | `null` (size-less) |

**Single source of truth:** `size_unit` is stored **only on `categories`**. A
variant stores just `size_value numeric`. The unit is always derived by joining
the product's category — so there is no duplicated unit to drift. A DB trigger
enforces the cross-table invariant (value present iff the category is sized; and
optionally that the value is one of the category's allowed options).

This is the principled choice for "single point of truth." See **Open
Decisions** for the lighter app-level alternative if the team prefers to avoid a
trigger.

---

## SQL schema — authoritative DDL for the Developer

These rewrite `shared/models/catalog.sql`, fill `shared/models/channel.sql`, and
align `shared/models/inventory.sql`. Style note: standardized on lowercase +
explicit schema, matching the files being edited (see Open Decisions re:
repo-wide style/`public` vs `private`).

### `categories` — declares the size unit per jewelry type
```sql
create table public.categories (
    id          uuid primary key default gen_random_uuid(),
    slug        text not null unique,        -- 'ring','necklace','pendant','bracelet','earring','charm'
    name        text not null,
    size_unit   text not null default 'none'
                check (size_unit in ('none','inch','ring_us','mm')),
    sort_order  int  not null default 0,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);
```

### `category_size_options` — allowed size values per category (validation + UI)
```sql
create table public.category_size_options (
    id          uuid primary key default gen_random_uuid(),
    category_id uuid not null references public.categories(id) on delete cascade,
    size_value  numeric(6,2) not null,       -- 18.00 | 7.00
    label       text,                        -- '18"' | 'US 7'
    sort_order  int  not null default 0,
    unique (category_id, size_value)
);
```

### `collections` + `product_collections` (a product may sit in many collections)
```sql
create table public.collections (
    id          uuid primary key default gen_random_uuid(),
    slug        text not null unique,
    name        text not null,
    description text,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);

create table public.product_collections (
    product_id    uuid not null references public.products(id)    on delete cascade,
    collection_id uuid not null references public.collections(id) on delete cascade,
    primary key (product_id, collection_id)
);
```

### `products` — the design/concept (corrected from the draft)
```sql
create table public.products (
    id               uuid primary key default gen_random_uuid(),
    handle           text not null unique,
    name             text not null,
    description      text,
    category_id      uuid not null references public.categories(id),
    default_material text,                    -- display/filter default; per-SKU material is on the variant
    tags             text[] not null default '{}',
    status           text not null default 'draft'
                     check (status in ('draft','active','archived')),
    created_at       timestamptz not null default now(),
    updated_at       timestamptz not null default now()
);
create index products_category_id_idx on public.products (category_id);
```

### `product_variants` — the sellable SKU (consolidates draft's display/details split)
```sql
create table public.product_variants (
    id           uuid primary key default gen_random_uuid(),
    product_id   uuid not null references public.products(id) on delete cascade,

    -- Axis 1: type = material + design
    material     text not null,              -- '18k light gold'
    design       text,                       -- 'chain'
    type_label   text not null,              -- '18k light gold chain'

    -- Axis 2: size value; unit is derived from the product's category
    size_value   numeric(6,2),               -- null when category.size_unit = 'none'

    -- economics + identifiers
    sku          text not null unique,
    barcode      text,
    price_cents  bigint not null check (price_cents >= 0),
    cost_cents   bigint check (cost_cents >= 0),
    weight_grams int,
    dimensions   text,                        -- '10x5x2 mm'
    status       text not null default 'active'
                 check (status in ('active','archived','draft')),

    created_at   timestamptz not null default now(),
    updated_at   timestamptz not null default now(),

    -- no two identical sellable combinations under one product
    unique (product_id, material, design, size_value)
);
create index product_variants_product_id_idx on public.product_variants (product_id);
```

### Cross-table size invariant — trigger (recommended enforcement)
```sql
-- Enforces: size_value is null iff the product's category is size-less,
-- and (if category_size_options has rows for that category) size_value must be
-- one of the allowed options. Runs on insert/update of product_variants.
create or replace function public.check_variant_size() returns trigger as $$
declare
    v_unit text;
    v_has_options boolean;
begin
    select c.size_unit into v_unit
      from public.products p
      join public.categories c on c.id = p.category_id
     where p.id = new.product_id;

    if v_unit = 'none' then
        if new.size_value is not null then
            raise exception 'category is size-less; size_value must be null';
        end if;
        return new;
    end if;

    if new.size_value is null then
        raise exception 'category % requires a size_value', v_unit;
    end if;

    select exists(select 1 from public.category_size_options o
                  join public.products p on p.category_id = o.category_id
                  where p.id = new.product_id) into v_has_options;
    if v_has_options and not exists(
        select 1 from public.category_size_options o
        join public.products p on p.category_id = o.category_id
        where p.id = new.product_id and o.size_value = new.size_value
    ) then
        raise exception 'size_value % is not an allowed option for this category', new.size_value;
    end if;
    return new;
end;
$$ language plpgsql;

create trigger product_variants_size_check
    before insert or update on public.product_variants
    for each row execute function public.check_variant_size();
```

### `product_media` — images/assets (object_store keys)
```sql
create table public.product_media (
    id          uuid primary key default gen_random_uuid(),
    product_id  uuid not null references public.products(id) on delete cascade,
    variant_id  uuid references public.product_variants(id) on delete cascade, -- optional variant-specific media
    storage_key text not null,               -- object_store key (local/S3/R2)
    kind        text not null default 'image'
                check (kind in ('image','video','model_3d')),
    alt_text    text,
    position    int  not null default 0,
    created_at  timestamptz not null default now()
);
create index product_media_product_id_idx on public.product_media (product_id);
```

### `channel.sql` — external marketplace listing mapping (currently empty)
```sql
create table public.channel_listings (
    id                  uuid primary key default gen_random_uuid(),
    channel             text not null check (channel in ('etsy','ebay','website')),
    external_listing_id text not null,        -- marketplace listing/offer id
    external_variant_id text,                 -- marketplace variation id (etsy) where applicable
    external_sku        text,
    product_id          uuid not null references public.products(id),
    variant_id          uuid references public.product_variants(id),
    raw                 jsonb,                -- last-seen raw payload for reconciliation
    status              text not null default 'active',
    last_synced_at      timestamptz,
    created_at          timestamptz not null default now(),
    updated_at          timestamptz not null default now(),
    unique (channel, external_listing_id, external_variant_id)
);
create index channel_listings_variant_id_idx on public.channel_listings (variant_id);
```

### `inventory.sql` — align to `product_variants` + one row per variant
- Rename references `public.product_variant` → `public.product_variants`.
- Add `unique (variant_id)` so each variant has exactly one inventory row.
- `product_id` becomes redundant (derivable via variant); **recommend dropping it**
  from `inventory` and keying purely on `variant_id`. Keep `inventory_adjustment`
  / `inventory_log` / `inventory_alert` as-is (they reference `inventory.id`).
- Consider `available int generated always as (quantity - reserved) stored`.

---

## DDD work in `src/domains/catalog/` (reads the schema above)

Pure-domain rules per AGENTS.md (no `serde`/`axum`/`sqlx` in `domain/`):

- `domain/`
  - `Size` value object encoding the category-governed unit invariant
    (`Inch(f32)`, `RingUs(f32)`, `None`) — **unit tests for invariants live here**
    (positive inches, ring sizes within a sane range, size-less ⇒ no value).
  - `VariantType` value object (`material`, `design`, derived `type_label`).
  - `Money` (cents) — promote to `common/` only if a second context needs it.
  - Entities: `Product`, `ProductVariant`, `Category`, `Collection`.
  - Ports: `#[async_trait] ProductRepository`, `InventoryRepository`,
    `ChannelListingRepository`.
- `application/` — `CreateProductUseCase`, `AddVariantUseCase`,
  `AdjustInventoryUseCase`, `ListCatalogUseCase`, `MapChannelListingUseCase`.
  Depends only on `domain/` ports.
- `infra/` — SeaORM-backed repositories implementing the ports (added once
  entities are regenerated — see Deferred).
- `dto/` — serde request/response structs for admin product maintenance + public
  catalog reads.
- `api/` — route handlers using `AuthUser`; admin writes behind the admin group,
  catalog reads on the public group.

Scope for Phase 1: schema + `domain/` + `application/` + `dto/` + `api/` wiring.
`infra/` repositories follow once entities exist.

---

## Deferred (explicitly out of scope now, per directive)

- Regenerating `src/entity/*` from the new SQL via `sea-orm-cli`.
- Authoring/repairing `src/migration/m*.rs` runner files.
- Backfilling/seeding real catalog data.
- `craft` modular-compatibility SKUs.

These are sequenced after the SQL is approved, since entities are generated
**from** the canonical SQL.

## New files expected

- `stellaux_server/src/domains/catalog/domain/{mod,size,variant_type,error}.rs`
- (later) `application/`, `infra/` once entities are regenerated

(Rewrite: `shared/models/catalog.sql` — **all** catalog tables + trigger in one
file. Fill: `shared/models/channel.sql`. Edit: `shared/models/inventory.sql`
(align to `product_variants`) and `shared/models/order_items.sql` (add nullable
`variant_id`).)

## Validation after implementation

1. Apply the SQL to a scratch Postgres (`psql -f`) — all files load clean, in FK
   order (`categories` → `collections` → `products` → `product_variants` →
   `product_media`/`channel_listings`/`inventory`).
2. Trigger sanity: inserting a ring variant with no `size_value` fails; a pendant
   variant with `size_value = 18` succeeds; an earring variant with a non-null
   `size_value` fails.
3. `cargo test --lib` — `Size`/`VariantType` invariant tests pass.
4. `cargo check` green for the `catalog` domain scaffolding.

## Locked Decisions (approved 2026-06-08)

1. **Size-unit enforcement**: **DB trigger, strictly.** App-level checks may
   surface early errors but the trigger is the final guard. Trigger raises with
   `ERRCODE 23514` (`check_violation`) so the application can catch it cleanly.
2. **`material` placement**: **keep `default_material` on `products`** (required).
   UX flow: the operator sets the product's display/default material first, then
   selects/creates variants relative to it. Authoritative per-SKU material still
   lives on the variant; document this clearly.
3. **Ring size scale**: **US ring sizes, half steps.** `numeric(6,2)`; the domain
   `Size` value object enforces `size_value % 0.5 == 0` and a sane range.
4. **Schema/style consistency**: **lowercase + `public`** for all new catalog
   tables; `private` preserved for inventory. Repo-wide unification of the legacy
   uppercase/unprefixed tables (`orders`, `users`) is a separate tech-debt ticket.
5. **`order_items` ↔ variant**: **add `variant_id` now** (this phase). Nullable —
   marketplace rows where the variant is unknown stay null and fall back to a
   SKU lookup at inventory-decrement time (Phase 2).

### Review refinements folded in
- **File organization**: one `catalog.sql` for all catalog tables (categories,
  size options, collections + junction, products, variants, media, trigger) —
  they are tightly coupled. `channel.sql` and `inventory.sql` stay separate.
- **Trigger**: `raise exception ... using errcode = '23514'` on each failure path.
- **`Size` value object**: receives `(size_value, unit)` from the repo (the repo
  must JOIN `categories` since SeaORM entities won't model the derivation or the
  trigger). Domain enforces business invariants; trigger is a separate integrity
  layer — do **not** duplicate trigger logic into the value object beyond the
  US half-step/range rules.
- **Validation add**: assert that changing a product's `category_id` re-evaluates
  the size rule on subsequent variant writes (trigger reads current category).
```
