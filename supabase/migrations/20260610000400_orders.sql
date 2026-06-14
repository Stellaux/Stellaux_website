-- Orders schema.
-- Mirrors canonical SQL from:
--   * shared/models/orders.sql
--   * shared/models/order_items.sql

create table public.orders (
    id               uuid primary key default gen_random_uuid(),
    user_id          uuid null references public.users(id) on delete set null,
    guest_profile_id uuid null references public.guest_profiles(id) on delete set null,
    constraint order_owner_check check (
       (user_id is not null and guest_profile_id is null) or
       (user_id is null and guest_profile_id is not null)
    ),
    order_number     text unique not null,
    status           text not null default 'pending',
    currency         text not null default 'USD',
    subtotal_cents   bigint not null,
    tax_cents        bigint not null,
    shipping_cents   bigint not null,
    total_cents      bigint not null,
    total            int not null,
    created_at       timestamptz not null default now(),
    updated_at       timestamptz not null default now(),
    shipping_address jsonb not null,
    billing_address  jsonb not null,
    placed_at        timestamptz default now(),
    shipped_at       timestamptz,
    paid_at          timestamptz
);
create index orders_user_id_idx on public.orders (user_id);
create index orders_guest_profile_id_idx on public.orders (guest_profile_id);
create index orders_order_number_idx on public.orders (order_number);
alter table public.orders enable row level security;

create table public.order_items (
    id               uuid primary key default gen_random_uuid(),
    order_id         uuid not null references public.orders(id) on delete cascade,
    product_id       uuid not null references public.products(id),
    variant_id       uuid references public.product_variants(id),
    sku              text not null,
    name             text not null,
    quantity         int not null check (quantity > 0),
    unit_price_cents bigint not null,
    total_cents      bigint not null
);
create index order_items_order_id_idx on public.order_items (order_id);
create index order_items_variant_id_idx on public.order_items (variant_id);
alter table public.order_items enable row level security;
