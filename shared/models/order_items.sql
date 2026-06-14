CREATE TABLE public.order_items (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id     UUID NOT NULL REFERENCES public.orders(id) ON DELETE CASCADE,
    product_id   UUID NOT NULL REFERENCES public.products(id),
    -- Sellable SKU reference. Nullable: marketplace-ingested rows may not resolve
    -- to an internal variant immediately and fall back to a `sku` lookup at
    -- inventory-decrement time (Phase 2). `sku`/`name` remain purchase-time snapshots.
    variant_id   UUID REFERENCES public.product_variants(id),
    sku          TEXT NOT NULL,
    name         TEXT NOT NULL,           -- product name at purchase time
    quantity     INT NOT NULL CHECK (quantity > 0),
    unit_price_cents BIGINT NOT NULL,
    total_cents  BIGINT NOT NULL
);
CREATE INDEX order_items_order_id_idx ON public.order_items (order_id);
CREATE INDEX order_items_variant_id_idx ON public.order_items (variant_id);
ALTER TABLE public.order_items ENABLE ROW LEVEL SECURITY;
