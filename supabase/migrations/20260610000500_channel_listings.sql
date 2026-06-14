-- Channel listing mappings.
-- Mirrors canonical SQL from shared/models/channel.sql

create table public.channel_listings (
    id                  uuid primary key default gen_random_uuid(),
    channel             text not null check (channel in ('etsy','ebay','website')),
    external_listing_id text not null,
    external_variant_id text,
    external_sku        text,
    product_id          uuid not null references public.products(id),
    variant_id          uuid references public.product_variants(id),
    raw                 jsonb,
    status              text not null default 'active'
                        check (status in ('active','inactive','error')),
    last_synced_at      timestamptz,
    created_at          timestamptz not null default now(),
    updated_at          timestamptz not null default now(),
    unique (channel, external_listing_id, external_variant_id)
);
create index channel_listings_product_id_idx on public.channel_listings (product_id);
create index channel_listings_variant_id_idx on public.channel_listings (variant_id);
alter table public.channel_listings enable row level security;
