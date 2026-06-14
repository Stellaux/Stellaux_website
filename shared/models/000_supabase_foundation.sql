-- Supabase foundation and schema-boundary conventions.
--
-- Phase 0 rule:
--   * `public`  = tables that may need controlled Supabase-client visibility
--   * `private` = internal operational tables not exposed via Supabase's
--                 auto-generated APIs
--
-- The Rust API remains the primary business interface. Keeping a table in
-- `public` does NOT mean it is anonymously readable.

create extension if not exists pgcrypto;

create schema if not exists private;

comment on schema private is
    'Internal operational tables for inventory, reconciliation, audit, webhooks, and other non-PostgREST-facing data.';
