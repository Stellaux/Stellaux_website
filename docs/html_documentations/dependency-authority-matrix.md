# Dependency And Authority Matrix

## Goal

This document makes provider dependencies explicit so future agents and implementers do not have to infer which system owns what.

## Core Dependencies

| Dependency | Role in architecture | Required for Phase 0+ | Source of truth for | Notes |
|------------|----------------------|------------------------|---------------------|-------|
| Supabase Postgres | managed relational database | yes | internal operational data | canonical store for catalog, orders, customers, audit, webhook log |
| Supabase Auth | human identity provider | yes | identity issuance | Rust API verifies Supabase JWTs and resolves roles from internal DB |
| Supabase Realtime | internal dashboard transport | later/conditional | none | transport only, not domain truth |
| Rust API (`stellaux_server`) | orchestration and policy engine | yes | business rules | owns normalization, authorization, idempotency |
| Stripe | payment provider | phase-dependent but core to checkout | payment provider events | external authority mirrored internally |
| Shippo | shipping provider | phase-dependent but core to fulfillment | shipment-provider events | external authority mirrored internally |
| Resend | email delivery provider | phase-dependent | none | delivery only, not operational truth |
| Supabase Storage | product-image binary storage | yes | stored product-image files | selected storage provider for catalog imagery |

## Ownership Principles

### Internal truth

The internal source of truth lives in:

- Supabase Postgres for relational data
- object storage for binary assets
- Rust API contracts for business rules and normalization

### External truth

External systems remain authoritative only for the events they originate:

- Stripe for payment events
- Shippo for carrier-facing shipping events
- Supabase Auth for identity issuance

The system should mirror those events internally rather than let downstream consumers integrate each vendor directly.

## What Agents Should Assume

- Supabase is not “the backend” by itself; it is one platform dependency used by the Rust backend.
- The Rust API remains the place where cross-provider business rules belong.
- Marketplace channels, Stripe, and Shippo are integrations, not owners of internal domain models.
- Product-image storage is a finalized Phase 0 decision: Supabase Storage for binaries, Supabase Postgres for metadata and domain relationships.
