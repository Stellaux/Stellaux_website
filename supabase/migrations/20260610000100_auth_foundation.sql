-- Auth and session foundation.
-- Mirrors canonical SQL from:
--   * shared/models/user.sql
--   * shared/models/email_token.sql
--   * shared/models/session.sql
--   * shared/models/guest.sql

create table public.users (
    id        uuid primary key default gen_random_uuid(),
    email     text not null unique,
    password_hash  text not null,
    full_name text,
    created_at timestamptz not null default now(),
    last_login_at timestamptz
);
alter table public.users enable row level security;

create table public.email_tokens (
    id              uuid primary key default gen_random_uuid(),
    email           text not null,
    token           text unique not null,
    expires_at      timestamp not null default (now() + interval '7 days'),
    used_at         timestamp null,
    created_at      timestamp default now()
);
alter table public.email_tokens enable row level security;

create table public.session (
    id                  uuid primary key default gen_random_uuid(),
    anonymous_token     text not null unique,
    created_at          timestamptz not null default now(),
    expires_at          timestamptz,
    last_seen_at        timestamptz not null default now()
);
alter table public.session enable row level security;

create table public.guest_profiles (
    id                   uuid primary key default gen_random_uuid(),
    email                text not null,
    session_id           uuid not null references public.session(id) on delete cascade unique,
    device_fingerprint   text,
    created_at           timestamptz not null default now(),
    converted_to_user_id uuid null references public.users(id) on delete set null
);
alter table public.guest_profiles enable row level security;
