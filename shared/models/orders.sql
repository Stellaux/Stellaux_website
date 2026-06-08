CREATE TABLE orders (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id   UUID NULL REFERENCES users(id) ON DELETE set NULL,
    guest_profile_id  UUID NULL REFERENCES guest_profiles(id) ON DELETE set NULL,

    CONSTRAINT order_owner_check CHECK (
       (user_id IS NOT NULL AND guest_profile_id IS NULL) OR
       (user_id IS  NULL AND guest_profile_id IS NOT NULL)
    ),


    order_number TEXT UNIQUE NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending | paid | shipped | cancelled
    currency TEXT NOT NULL DEFAULT 'USD',
    subtotal_cents BIGINT NOT NULL, -- sum of all line items
    tax_cents BIGINT NOT NULL, -- tax on subtotal
    shipping_cents BIGINT NOT NULL, -- shipping on subtotal
    total_cents BIGINT NOT NULL, -- subtotal + tax + shipping

    
    total INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    shipping_address JSONB NOT NULL,
    billing_address JSONB NOT NULL,

    placed_at TIMESTAMPTZ DEFAULT NOW(),
    shipped_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ


);

CREATE INDEX orders_user_id_idx ON orders (user_id);
CREATE INDEX orders_guest_profile_id_idx ON orders (guest_profile_id);
CREATE INDEX orders_order_number_idx ON orders (order_number);