CREATE TABLE users (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email     TEXT NOT NULL UNIQUE,
    password_hash  TEXT NOT NULL,
    full_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ  
);

