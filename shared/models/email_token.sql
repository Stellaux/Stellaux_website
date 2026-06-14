CREATE TABLE public.email_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT NOT NULL,
    token           TEXT UNIQUE NOT NULL,
    expires_at      TIMESTAMP NOT NULL DEFAULT (NOW() + INTERVAL '7 days'),
    used_at         TIMESTAMP NULL,
    created_at      TIMESTAMP DEFAULT NOW()
);
ALTER TABLE public.email_tokens ENABLE ROW LEVEL SECURITY;
