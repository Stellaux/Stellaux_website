CREATE TABLE order_items (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id     UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id   UUID NOT NULL REFERENCES products(id),
    sku          TEXT NOT NULL,
    name         TEXT NOT NULL,           -- product name at purchase time
    quantity     INT NOT NULL CHECK (quantity > 0),
    unit_price_cents BIGINT NOT NULL,
    total_cents  BIGINT NOT NULL
);