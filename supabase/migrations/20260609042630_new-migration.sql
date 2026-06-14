-- Supabase foundation hardening.
--
-- This migration establishes the internal-only schema boundary used by
-- operational tables such as inventory, reconciliation, audit, and webhook
-- persistence. Business tables that remain in `public` must still declare an
-- explicit RLS posture in their own schema migrations.

create extension if not exists pgcrypto;
create schema if not exists private;

comment on schema private is
    'Internal operational tables for inventory, reconciliation, audit, webhooks, and other non-PostgREST-facing data.';
