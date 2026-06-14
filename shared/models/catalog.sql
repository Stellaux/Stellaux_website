-- Catalog schema — canonical source of truth (SeaORM entities are generated FROM
-- this, never the other way around). One file: categories, size options,
-- collections (+ junction), products, product_variants, product_media, and the
-- variant-size integrity trigger. They are tightly coupled, so they live together.
--
-- Variant model (two axes):
--   1. "type"  = material + design   (e.g. "18k light gold" + "chain")
--   2. "size"  = a numeric value whose UNIT is governed by the product's category
--               (ring -> US ring size; necklace/pendant/bracelet -> inches;
--                earring/charm -> size-less). The unit is stored ONLY on
--                `categories` (single source of truth) and derived via join.
--
-- Supabase hardening note:
-- Every `public` business table in this file has RLS enabled with no
-- permissive policies defined here. Direct Supabase-client reads must be added
-- intentionally later.

-- ─── Categories: declare the size unit per jewelry type ──────────────────────
create table public.categories (
    id          uuid primary key default gen_random_uuid(),
    slug        text not null unique,        -- 'ring','necklace','pendant','bracelet','earring','charm'
    name        text not null,
    -- The single source of truth for "which size option applies": rings use a
    -- numbered (US) scale, chains/bracelets use inches, earrings/charms are size-less.
    size_unit   text not null default 'none'
                check (size_unit in ('none','inch','ring_us','mm')),
    sort_order  int  not null default 0,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);
alter table public.categories enable row level security;

-- ─── Allowed size values per category (validation + UI option lists) ─────────
-- e.g. ring -> 3,3.5,...,13 (US half steps); necklace -> 16,18,20,24 (inches).
-- When a category has NO rows here, any numeric size_value is accepted.
create table public.category_size_options (
    id          uuid primary key default gen_random_uuid(),
    category_id uuid not null references public.categories(id) on delete cascade,
    size_value  numeric(6,2) not null,       -- 18.00 | 7.00
    label       text,                        -- '18"' | 'US 7'
    sort_order  int  not null default 0,
    unique (category_id, size_value)
);
alter table public.category_size_options enable row level security;

-- ─── Collections (many-to-many with products) ───────────────────────────────
create table public.collections (
    id          uuid primary key default gen_random_uuid(),
    slug        text not null unique,
    name        text not null,
    description text,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);
alter table public.collections enable row level security;

-- ─── Products: the design/concept (not directly sellable; variants are) ──────
create table public.products (
    id               uuid primary key default gen_random_uuid(),
    handle           text not null unique,
    name             text not null,
    description      text,
    category_id      uuid not null references public.categories(id),
    -- Display/filter default. The operator sets this first, then selects variants
    -- relative to it. AUTHORITATIVE per-SKU material lives on the variant.
    default_material text,
    tags             text[] not null default '{}',
    status           text not null default 'draft'
                     check (status in ('draft','active','archived')),
    created_at       timestamptz not null default now(),
    updated_at       timestamptz not null default now()
);
create index products_category_id_idx on public.products (category_id);
alter table public.products enable row level security;

create table public.product_collections (
    product_id    uuid not null references public.products(id)    on delete cascade,
    collection_id uuid not null references public.collections(id) on delete cascade,
    primary key (product_id, collection_id)
);
alter table public.product_collections enable row level security;

-- ─── Product variants: one row per sellable SKU (product × type × size) ──────
create table public.product_variants (
    id           uuid primary key default gen_random_uuid(),
    product_id   uuid not null references public.products(id) on delete cascade,

    -- Axis 1: type = material + design
    material     text not null,              -- '18k light gold'
    design       text,                       -- 'chain' (nullable for size/material-only variants)
    type_label   text not null,              -- '18k light gold chain' (display / marketplace title)

    -- Axis 2: size value; the UNIT is derived from the product's category.
    -- null exactly when the category is size-less (enforced by trigger below).
    size_value   numeric(6,2),

    -- economics + identifiers
    sku          text not null unique,
    barcode      text,
    price_cents  bigint not null check (price_cents >= 0),
    cost_cents   bigint check (cost_cents >= 0),
    weight_grams int,
    dimensions   text,                       -- '10x5x2 mm'
    status       text not null default 'active'
                 check (status in ('active','archived','draft')),

    created_at   timestamptz not null default now(),
    updated_at   timestamptz not null default now(),

    -- No two identical sellable combinations under one product. NOTE: NULL design
    -- and NULL size_value are distinct under SQL semantics, which matches the
    -- domain (a size-less / design-less variant is a single legitimate row).
    unique (product_id, material, design, size_value)
);
create index product_variants_product_id_idx on public.product_variants (product_id);
alter table public.product_variants enable row level security;

-- ─── Variant-size integrity trigger ─────────────────────────────────────────
-- Cross-table invariant (CHECK constraints can't reference other tables):
--   * size_value IS NULL  iff  the product's category is size-less ('none')
--   * if the category defines option rows, size_value must be one of them
-- Raises with ERRCODE 23514 (check_violation) so the app layer can catch it.
create or replace function public.check_variant_size() returns trigger as $$
declare
    v_unit        text;
    v_category_id uuid;
    v_has_options boolean;
begin
    select p.category_id, c.size_unit
      into v_category_id, v_unit
      from public.products p
      join public.categories c on c.id = p.category_id
     where p.id = new.product_id;

    if v_unit = 'none' then
        if new.size_value is not null then
            raise exception 'category is size-less; size_value must be null'
                using errcode = '23514';
        end if;
        return new;
    end if;

    if new.size_value is null then
        raise exception 'category (unit %) requires a size_value', v_unit
            using errcode = '23514';
    end if;

    select exists(
        select 1 from public.category_size_options o
        where o.category_id = v_category_id
    ) into v_has_options;

    if v_has_options and not exists(
        select 1 from public.category_size_options o
        where o.category_id = v_category_id
          and o.size_value = new.size_value
    ) then
        raise exception 'size_value % is not an allowed option for this category', new.size_value
            using errcode = '23514';
    end if;

    return new;
end;
$$ language plpgsql;

create trigger product_variants_size_check
    before insert or update on public.product_variants
    for each row execute function public.check_variant_size();

-- ─── Product media (object_store keys; optional variant-specific media) ──────
create table public.product_media (
    id          uuid primary key default gen_random_uuid(),
    product_id  uuid not null references public.products(id) on delete cascade,
    variant_id  uuid references public.product_variants(id) on delete cascade,
    storage_key text not null,               -- Supabase Storage object key
    kind        text not null default 'image'
                check (kind in ('image','video','model_3d')),
    alt_text    text,
    position    int  not null default 0,
    created_at  timestamptz not null default now()
);
create index product_media_product_id_idx on public.product_media (product_id);
alter table public.product_media enable row level security;
