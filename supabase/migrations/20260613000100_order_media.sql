-- Order media schema + private storage bucket.
-- Mirrors canonical SQL from shared/models/order_media.sql
--
-- Photographs of the actual purchased / custom-crafted piece for an order line,
-- uploaded from the internal dashboard. Anchored on order_item_id (which encodes
-- order + variant + size). Bucket is private; access is mediated by the Rust API.

-- ---------------------------------------------------------------------------
-- 1. Table
-- ---------------------------------------------------------------------------

create table public.order_media (
    id            uuid primary key default gen_random_uuid(),
    order_id      uuid not null references public.orders(id) on delete cascade,
    order_item_id uuid not null references public.order_items(id) on delete cascade,
    variant_id    uuid references public.product_variants(id),
    size_value    numeric(6,2),
    storage_key   text not null unique,
    kind          text not null default 'image'
                  check (kind in ('image','video','model_3d')),
    alt_text      text,
    position      int not null default 0,
    is_current    boolean not null default true,
    captured_by   uuid references public.users(id),
    created_at    timestamptz not null default now()
);

create index order_media_order_id_idx on public.order_media (order_id);

create index order_media_item_current_idx
    on public.order_media (order_item_id)
    where is_current;

create unique index order_media_current_slot_uniq
    on public.order_media (order_item_id, position)
    where is_current;

alter table public.order_media enable row level security;
-- No PostgREST policies: closed to anon/authenticated. Rust API (service role) bypasses RLS.

-- ---------------------------------------------------------------------------
-- 2. Private storage bucket
-- ---------------------------------------------------------------------------

insert into storage.buckets (id, name, public, file_size_limit, allowed_mime_types)
values (
    'order-media',
    'order-media',
    false,                                   -- private: no public URL
    10485760,                                -- 10 MB per object
    array['image/jpeg','image/png','image/webp','image/avif']
)
on conflict (id) do nothing;

-- No storage.objects policies are created for this bucket, so anon/authenticated roles
-- cannot read or write it. The internal dashboard and the Rust API operate via the
-- service role (which bypasses storage RLS) and serve customer reads as signed URLs.
