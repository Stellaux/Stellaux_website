-- Session records for guest cart and browser identity continuity.
-- This is intentionally minimal in Phase 0: it exists primarily so
-- `guest_profiles` and order ownership can reference a stable browser session.

create table public.session (
    id                  uuid primary key default gen_random_uuid(),
    anonymous_token     text not null unique,
    created_at          timestamptz not null default now(),
    expires_at          timestamptz,
    last_seen_at        timestamptz not null default now()
);

alter table public.session enable row level security;
