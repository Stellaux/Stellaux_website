-- Order media: photographs of the *actual* purchased / custom-crafted piece for an
-- order line. Distinct from product_media (catalog stock photography).
--
-- Anchor: order_item_id. An order_item already identifies (order, product, variant,
-- sku); the variant carries the size (product_variants.size_value). variant_id and
-- size_value are denormalized here so the internal dashboard can filter/label without
-- a join, and so the row survives if the variant is later archived.
--
-- Access model: the bucket is PRIVATE. This codebase authenticates through the Rust
-- API (public.users + password_hash), not Supabase auth.uid(), so storage RLS by
-- auth.uid() does not apply. Reads are served as short-lived signed URLs minted by the
-- Rust API after the ownership guard; writes come from the internal dashboard via the
-- Rust admin API using the service role. No anon/authenticated storage policies exist,
-- so the bucket is closed by default.

create table public.order_media (
    id            uuid primary key default gen_random_uuid(),
    order_id      uuid not null references public.orders(id) on delete cascade,
    order_item_id uuid not null references public.order_items(id) on delete cascade,
    variant_id    uuid references public.product_variants(id),       -- denormalized
    size_value    numeric(6,2),                                      -- snapshot at capture
    storage_key   text not null unique,                              -- path within 'order-media'
    kind          text not null default 'image'
                  check (kind in ('image','video','model_3d')),
    alt_text      text,
    position      int not null default 0,                            -- slot within the line
    is_current    boolean not null default true,                     -- active version pointer
    captured_by   uuid references public.users(id),                  -- dashboard operator
    created_at    timestamptz not null default now()
);

create index order_media_order_id_idx on public.order_media (order_id);

-- Hot read: "current media for the items of one order". Partial index keeps it tiny.
create index order_media_item_current_idx
    on public.order_media (order_item_id)
    where is_current;

-- One current image per (line, slot). Replacement flips the old row's is_current to
-- false and inserts a new versioned row inside a transaction; this guarantees a single
-- live image per slot while retaining history.
create unique index order_media_current_slot_uniq
    on public.order_media (order_item_id, position)
    where is_current;

alter table public.order_media enable row level security;
-- No policies: closed to anon/authenticated PostgREST. Service role (Rust API) bypasses
-- RLS for reads/writes.
