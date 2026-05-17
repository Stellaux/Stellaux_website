create table public.products {
    id uuid primary key default gen_random_uuid(),
    handle text not null unique,
    name text not null,
    description text,
    collection text,
    category text not null,                --bracelet | necklace | earring | charms
    material text not null,                --18k gold | vermil | silver 
    tags text[],                          -- ["gift", "birthday", "anniversary"]
    status text not null default "active"  -- active, archived, draft

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
}

create table public.product_variant {
    id uuid primary key default get_random_uuid(),
    handle text not null unique,
}

create table public.product_display {
    id uuid primary key default gen_random_uuid(),
    product_id uuid not null references public.products(id),
    variant_id uuid not null references public.product_variant(id),
    price int not null, -- in cents
    
}

create table public.product_details {
    id uuid primary key default gen_random_uuid(),
    product_id uuid not null references public.products(id),
    variant_id uuid not null references public.product_variant(id),
    weight int, -- in grams
    dimensions text, -- "10x5x2 cm"
    cost int, -- in cents
    sku text,
    barcode text,

}