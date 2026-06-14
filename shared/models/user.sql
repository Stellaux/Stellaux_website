CREATE TABLE public.users (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email     TEXT NOT NULL UNIQUE,
    password_hash  TEXT NOT NULL,
    full_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ  
);
ALTER TABLE public.users ENABLE ROW LEVEL SECURITY;
