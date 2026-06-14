-- Inventory — per-variant stock levels and the adjustment audit trail.
-- Lives in the `private` schema (internal-only; not exposed via the public API).
-- The foundational Supabase migration must create the `private` schema before
-- this file is applied.
--
-- Depends on: catalog.sql (product_variants).
--
-- One inventory row per variant. `product_id` is intentionally omitted: it is
-- derivable via product_variants and keeping it here would invite drift.

create table private.inventory (
    id          uuid primary key default gen_random_uuid(),
    variant_id  uuid not null unique references public.product_variants(id),
    quantity    int not null default 0 check (quantity >= 0),
    reserved    int not null default 0 check (reserved >= 0),
    -- convenience for read queries; never write directly
    available   int generated always as (quantity - reserved) stored,

    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);

create table private.inventory_adjustment (
    id            uuid primary key default gen_random_uuid(),
    inventory_id  uuid not null references private.inventory(id),
    channel       text not null check (channel in ('website', 'etsy', 'ebay')),
    change        int not null,
    reason        text not null,

    actor_user_id uuid not null,
    notes         text,
    created_at    timestamptz not null default now()
);

create table private.inventory_log (
    id            uuid primary key default gen_random_uuid(),
    inventory_id  uuid not null references private.inventory(id),
    adjustment_id uuid references private.inventory_adjustment(id),
    reason        text not null,

    created_at    timestamptz not null default now()
);

create table private.inventory_alert (
    id            uuid primary key default gen_random_uuid(),
    inventory_id  uuid not null references private.inventory(id),
    quantity      int not null,
    reason        text not null,

    created_at    timestamptz not null default now()
);
