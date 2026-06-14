# Overview Timeline

## Planning Intent

This timeline reflects the updated delivery strategy:

- eBay and Etsy are the live storefronts in the near term
- the Rust backend and Postgres schema become the operational source of truth
- product, order, shipment, and customer-service capabilities come before the full first-party website
- the TanStack Start storefront continues in the background so it can attach to a stable backend later

## Core Component Map

| Layer | Implementation | Source of truth for |
|------|----------------|---------------------|
| Storefront UI | TanStack Start (React 19, Vite), Radix UI primitives, Tailwind v4. Deployed to Cloudflare Workers via `@cloudflare/vite-plugin`. | Presentation only |
| API server | Rust 2024 edition, Axum 0.8 on Tokio, SeaORM 1.1 (`rustls` + `sqlx-postgres`). One static binary; `tower-http` middleware stack. | Business logic, authorization, idempotency |
| Database | Supabase Postgres 16 (managed). SeaORM migrations tracked in `seaql_migrations`; applied at process boot before `axum::serve`. Supabase Realtime channels are used by the internal dashboard on a per-table basis. | Catalog, inventory, carts, orders, customers, webhook log, audit log |
| Object storage | Supabase Storage for product images, consumed through the backend storage contract. | Product-image binaries; metadata remains in Postgres |
| Auth | JWT (`jsonwebtoken`) signed by the server, or Supabase JWTs validated via JWKS cache. Argon2 for password storage. | Session tokens; user records live in Postgres |
| Payments | Stripe Checkout Sessions; webhook handler at `/api/v1/webhooks/stripe` verifies signature on the raw body. | Stripe is authoritative for payment state, mirrored to Postgres |
| Shipping | Shippo: rate quotes at checkout, label purchase post-payment, tracking via webhook. | Shippo is authoritative for tracking, mirrored to Postgres |
| Email | Resend, called from the API server. Templates planned in `react-email` compiled at build time. | Transactional sends only; no marketing list in v1 |

## Implementation Order Shift

### Previous assumption

- The first-party website would lead the product rollout, with marketplace channels acting as secondary sales surfaces.

### Updated assumption

- Marketplace channels are the active customer-facing storefront now.
- The backend must first support operations: catalog, inventory, orders, fulfillment, customer support, and reconciliation.
- The first-party website is still being built, but it should consume stable backend services rather than drive their design.

## Delivery Principles

- Treat Supabase Postgres as the internal operational truth even when eBay, Etsy, Stripe, and Shippo remain external authorities for parts of the workflow.
- Treat Supabase Storage as the selected product-image binary store, with Postgres retaining metadata, ordering, and domain relationships for those assets.
- Design backend domains so marketplace ingestion and the future website share the same order, catalog, inventory, and customer services.
- Defer UX-heavy website work until the API, schema, and operational tooling for fulfillment are stable.
- Prioritize idempotent webhook ingestion, auditability, and admin visibility early, since those are required whether orders originate on marketplaces or the eventual website.

## High-Level Phases

## Phase 0: Platform Foundation and Contracts

### Goal

Lock the system boundaries so product/order/fulfillment work can proceed without being blocked by frontend decisions.

### Primary outcomes

- Confirm runtime architecture for Rust API, Supabase Postgres, object storage, Stripe, Shippo, Resend, and Supabase auth/JWKS
- Finalize core bounded contexts and table ownership
- Establish migration discipline, webhook logging, audit logging, and idempotency conventions
- Keep the website and dashboard teams aligned on API contracts, but do not let frontend polish set backend priority

### Main areas

- `src/common/`
- `src/domains/auth/`
- `src/domains/webhooks/`
- SQL schema under `shared/models/`
- SeaORM migration runner in `stellaux_server/src/migration/`

## Phase 1: Product and Inventory Backbone

### Goal

Build the catalog and inventory model that supports marketplace operations first and the website later.

### Why this comes first now

- Marketplace orders cannot be fulfilled reliably without internal product, variant, SKU, inventory, and media records
- Website catalog pages can wait, but operational product data cannot

### Primary outcomes

- Canonical product, variant, collection, category, and media schema
- Inventory levels and reservation model
- Mapping strategy for external marketplace listings back to internal products/variants
- Admin-safe product maintenance workflows and import/sync paths

### Main domains

- `catalog`
- `admin`
- `craft` if modular compatibility affects sellable SKUs

### Data focus

- products
- variants
- images/assets
- collections/categories
- inventory levels
- external channel listing references

## Phase 2: Order Lifecycle and Operational Database

### Goal

Implement the order model and processing rules for orders originating on eBay and Etsy before first-party checkout is complete.

### Why this moves ahead of storefront work

- Orders already exist through external channels, so the backend must normalize and manage them now
- This is the operational core the future website will reuse

### Primary outcomes

- Internal order, order-item, payment snapshot, address, and status models
- Marketplace order ingestion flow with idempotent reconciliation
- Unified order timeline regardless of order source
- Audit trail for status changes, exceptions, and operator actions

### Main domains

- `orders` or equivalent new bounded context
- `admin`
- `webhooks`
- `account` for future customer visibility

### Data focus

- orders
- order items
- order source/channel
- status transitions
- payment mirrors
- audit log

## Phase 3: Shipment and Fulfillment Services

### Goal

Operationalize label purchase, storage, tracking, and delivery status updates for marketplace-originated orders.

### Why this is top priority

- Shipping is part of the live revenue path now
- Fulfillment logic is needed immediately even before the website launches checkout

### Primary outcomes

- Shipment records linked to orders
- Shippo label purchase and label PDF storage
- Tracking webhook ingestion and normalized delivery events
- Fulfillment status model usable by support and future customer account views

### Main domains

- `checkout` only where shipping service code is reusable
- `webhooks`
- `admin`
- likely a dedicated `shipment` or `fulfillment` bounded context

### Data focus

- shipments
- labels/assets
- tracking events
- delivery states
- carrier/service metadata

## Phase 4: Customer Service and Post-Purchase Support

### Goal

Give the business a reliable internal view of customers, their orders, shipment state, and support actions before self-service features are finished.

### Why this precedes the full customer-facing site

- Customer support is required immediately for marketplace orders
- Internal service tooling delivers value even without public account pages

### Primary outcomes

- Customer profile model unified across marketplace and future direct-site buyers where possible
- Internal customer lookup by email, order id, and channel
- Order status, shipment status, refunds/cancellations notes, and support audit trail
- Email-trigger points for transactional notifications

### Main domains

- `account`
- `admin`
- `auth`
- `webhooks`
- likely a dedicated `customer` or `support` bounded context

### Data focus

- customer profiles
- contact points
- linked orders
- service notes
- communication history

## Phase 5: Marketplace Integration Hardening

### Goal

Stabilize all external channel and webhook flows so the system can operate reliably while the website is still in development.

### Primary outcomes

- eBay/Etsy import and reconciliation jobs or adapters
- resilient webhook/event ingestion with replay safety
- idempotent external event log
- operational dashboards and alerts for stuck orders, failed syncs, and shipment issues

### Main domains

- `webhooks`
- `admin`
- `common`
- integration-specific infra modules

## Phase 6: First-Party Storefront Build in Parallel

### Goal

Continue building the website on top of stable backend primitives without forcing backend sequencing.

### Primary outcomes

- Public catalog pages backed by the same canonical product data
- Auth flows connected to the unified identity approach
- Cart and checkout flows that reuse the order, payment, and shipment services already built for operations
- Customer account pages that sit on top of the support-ready customer/order model

### Main areas

- TanStack Start storefront
- `catalog`
- `auth`
- `cart`
- `checkout`
- `account`

### Delivery note

- This phase runs in parallel after Phase 1 starts, but it should not preempt Phases 2 through 4 unless a dependency is blocking shared contracts.

## Phase 7: Internal Dashboard and Realtime Operations

### Goal

Add higher-leverage operational tooling once the core product/order/shipment/customer data model is trustworthy.

### Primary outcomes

- Internal dashboard backed by Supabase Realtime where appropriate
- Order queue visibility
- fulfillment exception monitoring
- inventory visibility
- customer-service workflow support

## Suggested Near-Term Sequence

1. Finalize the product and inventory schema, including marketplace listing mapping.
2. Implement the order model and marketplace ingestion path.
3. Implement shipment persistence, Shippo label flow, and tracking updates.
4. Implement customer-service data model and internal support workflows.
5. Harden marketplace integrations, webhook idempotency, and operational visibility.
6. Continue the first-party storefront against those stable services.

## Dependency Guidance

- The website should depend on the catalog, order, shipment, auth, and customer APIs being stable.
- Shipment work depends on order normalization being in place.
- Customer-service tooling depends on orders and shipments being queryable.
- Marketplace adapters should write into the same internal models the website will eventually use.
- Admin/dashboard features should be built around the operational domains, not around frontend page structure.

## Success Criteria For The New Order

- The team can operate eBay and Etsy sales through the internal backend without relying on spreadsheet-style manual reconciliation.
- Product, order, shipment, and customer data live in stable internal models before the first-party storefront reaches full checkout readiness.
- The future website becomes a new sales surface on top of existing services, not a parallel system with different business rules.
