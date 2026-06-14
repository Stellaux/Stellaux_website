-- Catalog schema.
-- Mirrors canonical SQL from shared/models/catalog.sql

create table public.categories (
    id          uuid primary key default gen_random_uuid(),
    slug        text not null unique,
    name        text not null,
    size_unit   text not null default 'none'
                check (size_unit in ('none','inch','ring_us','mm')),
    sort_order  int not null default 0,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);
alter table public.categories enable row level security;

create table public.category_size_options (
    id          uuid primary key default gen_random_uuid(),
    category_id uuid not null references public.categories(id) on delete cascade,
    size_value  numeric(6,2) not null,
    label       text,
    sort_order  int not null default 0,
    unique (category_id, size_value)
);
alter table public.category_size_options enable row level security;

create table public.collections (
    id          uuid primary key default gen_random_uuid(),
    slug        text not null unique,
    name        text not null,
    description text,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);
alter table public.collections enable row level security;

create table public.products (
    id               uuid primary key default gen_random_uuid(),
    handle           text not null unique,
    name             text not null,
    description      text,
    category_id      uuid not null references public.categories(id),
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
    product_id    uuid not null references public.products(id) on delete cascade,
    collection_id uuid not null references public.collections(id) on delete cascade,
    primary key (product_id, collection_id)
);
alter table public.product_collections enable row level security;

create table public.product_variants (
    id           uuid primary key default gen_random_uuid(),
    product_id   uuid not null references public.products(id) on delete cascade,
    material     text not null,
    design       text,
    type_label   text not null,
    size_value   numeric(6,2),
    sku          text not null unique,
    barcode      text,
    price_cents  bigint not null check (price_cents >= 0),
    cost_cents   bigint check (cost_cents >= 0),
    weight_grams int,
    dimensions   text,
    status       text not null default 'active'
                 check (status in ('active','archived','draft')),
    created_at   timestamptz not null default now(),
    updated_at   timestamptz not null default now(),
    unique (product_id, material, design, size_value)
);
create index product_variants_product_id_idx on public.product_variants (product_id);
alter table public.product_variants enable row level security;

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

create table public.product_media (
    id          uuid primary key default gen_random_uuid(),
    product_id  uuid not null references public.products(id) on delete cascade,
    variant_id  uuid references public.product_variants(id) on delete cascade,
    storage_key text not null,
    kind        text not null default 'image'
                check (kind in ('image','video','model_3d')),
    alt_text    text,
    position    int not null default 0,
    created_at  timestamptz not null default now()
);
create index product_media_product_id_idx on public.product_media (product_id);
alter table public.product_media enable row level security;
