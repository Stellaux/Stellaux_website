-- Inventory schema.
-- Mirrors canonical SQL from shared/models/inventory.sql

create table private.inventory (
    id          uuid primary key default gen_random_uuid(),
    variant_id  uuid not null unique references public.product_variants(id),
    quantity    int not null default 0 check (quantity >= 0),
    reserved    int not null default 0 check (reserved >= 0),
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
