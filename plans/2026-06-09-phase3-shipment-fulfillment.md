# Plan: Phase 3 — Shipment & Fulfillment Services

> Source of truth: `shared/models/*.sql`. New schema lands in
> `shared/models/shipments.sql`. SeaORM entities + the `src/migration/*.rs`
> runner are **deferred** (regenerated from SQL later), consistent with Phase 1.
> Follows the established conventions: lowercase DDL, `public` schema with RLS
> enabled (deny-by-default, no permissive policies), `gen_random_uuid()`.

## Bounded Context Affected

- **`fulfillment`** — **new** bounded context (the home for shipments, labels,
  tracking, delivery state). Recommended over `shipment` because it names the
  operational process, not just the noun; the core aggregate is `Shipment`.
- `webhooks` — harden the existing `/api/v1/webhooks/shippo` stub into a real
  signature-verified, idempotent ingress that dispatches to `fulfillment`.
- `admin` — operator endpoints for buying labels and viewing fulfillment state
  (mounted via the admin route group, `require_supabase_admin`).

## Cross-Cutting Areas Affected

- `shared/models/shipments.sql` — new canonical schema.
- `src/common/` — reuse only (no new shared types expected): `config.shippo`
  (`api_token`, `webhook_secret`), `storage::Storage`, `audit::events`,
  `metrics::external_call`, `idempotency::IdempotencyKey`.
- Object storage posture — **labels are PII (addresses); they must not reuse the
  public product-image bucket.** See "Label storage security".

## Goal

Operationalize, for marketplace-originated orders: buy a Shippo label, store the
label PDF, persist a shipment linked to the order, ingest Shippo tracking
webhooks into a normalized delivery timeline, and maintain a fulfillment status
model usable by support now and customer account views later.

---

## Dependencies & sequencing (read first)

- **Hard dependency on Phase 2 (orders).** `shipments.order_id` →
  `public.orders(id)`, and `shipment_items` → `public.order_items(id)`. Those
  tables exist in `shared/models/` (`orders.sql`, `order_items.sql`, the latter
  now carrying `variant_id`), but **Phase 2's ingestion/normalization use cases
  are the thing that populates them.** Phase 3 can be built and unit-tested
  against seeded orders, but end-to-end operation requires Phase 2 order rows.
- **Reuse boundary with `checkout`.** Per the overview, checkout touches Phase 3
  "only where shipping service code is reusable." The reusable surface is the
  `ShippingProvider` port (rate quoting + label purchase) living in
  `fulfillment/infra`; checkout will call it later. **Rate-at-checkout is not a
  Phase 3 deliverable** — marketplace handles customer checkout; we buy labels
  post-sale. Build `get_rates` on the provider only if it is needed to drive
  label purchase (see Open Decisions).
- **Webhook event log.** A central `webhook_events` table is not yet in
  `shared/models/` (the `webhooks` domain defines the repository trait but no SQL
  yet). Phase 3 does **not** block on it: durable idempotency is guaranteed by a
  `unique` on `tracking_events.source_event_id`, and the handler routes through
  the `IdempotencyKey` contract so it slots into the central log when it lands.

---

## SQL — `shared/models/shipments.sql` (authoritative DDL)

```sql
-- Shipment & fulfillment schema. Depends on: orders.sql, order_items.sql.
-- public schema + RLS (deny-by-default) because shipment/tracking state feeds
-- future customer account views, like orders. Label PDFs live in object storage
-- (private bucket); only the storage key is kept here.

create table public.shipments (
    id                    uuid primary key default gen_random_uuid(),
    order_id              uuid not null references public.orders(id) on delete cascade,

    -- carrier / service metadata
    carrier               text,                -- Shippo carrier token: 'usps','ups','fedex'
    service_level         text,                -- 'usps_priority', etc.

    -- Shippo references (provider is authoritative; mirrored here)
    shippo_transaction_id text unique,
    shippo_rate_id        text,

    -- tracking
    tracking_number       text,
    tracking_url          text,

    -- label asset: object-store key ONLY (private bucket); never a public URL
    label_storage_key     text,
    label_format          text default 'pdf',

    -- normalized fulfillment status (mirrors domain FulfillmentStatus)
    status                text not null default 'pending'
                          check (status in ('pending','label_purchased','in_transit',
                                            'out_for_delivery','delivered','returned',
                                            'failed','cancelled')),

    -- economics + parcel snapshots
    cost_cents            bigint check (cost_cents >= 0),
    weight_grams          int,
    parcel                jsonb,               -- parcel dims/weight sent to Shippo
    ship_to               jsonb,               -- address snapshot at purchase
    ship_from             jsonb,

    created_at            timestamptz not null default now(),
    updated_at            timestamptz not null default now(),
    label_purchased_at    timestamptz,
    shipped_at            timestamptz,
    delivered_at          timestamptz
);
create index shipments_order_id_idx        on public.shipments (order_id);
create index shipments_tracking_number_idx on public.shipments (tracking_number);
create index shipments_status_idx          on public.shipments (status);
alter table public.shipments enable row level security;

-- Line-level fulfillment: which order_items (and how many) ship in this parcel.
-- Supports partial / split shipments; a single-parcel order has one row per item.
create table public.shipment_items (
    id            uuid primary key default gen_random_uuid(),
    shipment_id   uuid not null references public.shipments(id) on delete cascade,
    order_item_id uuid not null references public.order_items(id),
    quantity      int  not null check (quantity > 0),
    unique (shipment_id, order_item_id)
);
create index shipment_items_shipment_id_idx on public.shipment_items (shipment_id);
alter table public.shipment_items enable row level security;

-- Normalized tracking timeline, appended from Shippo tracking webhooks.
-- source_event_id unique = durable idempotency guard, independent of any central
-- webhook log.
create table public.tracking_events (
    id              uuid primary key default gen_random_uuid(),
    shipment_id     uuid not null references public.shipments(id) on delete cascade,
    status          text not null,            -- normalized FulfillmentStatus string
    carrier_status  text,                      -- raw Shippo status: TRANSIT/DELIVERED/...
    status_detail   text,
    location        text,
    occurred_at     timestamptz not null,
    source_event_id text unique,              -- Shippo event id (idempotency)
    raw             jsonb,
    created_at      timestamptz not null default now()
);
create index tracking_events_shipment_id_idx on public.tracking_events (shipment_id);
alter table public.tracking_events enable row level security;
```

Carrier/service stay as text for v1 (a `shipping_carriers` lookup is overkill
until multi-carrier rate shopping matters — defer).

---

## Domain layer — `src/domains/fulfillment/domain/` (pure: no serde/axum/sqlx)

- `FulfillmentStatus` enum: `Pending`, `LabelPurchased`, `InTransit`,
  `OutForDelivery`, `Delivered`, `Returned`, `Failed`, `Cancelled`.
  - `from_shippo(status, substatus)` mapping (the policy):
    `PRE_TRANSIT → LabelPurchased`, `TRANSIT → InTransit`
    (`substatus=out_for_delivery → OutForDelivery`), `DELIVERED → Delivered`,
    `RETURNED → Returned`, `FAILURE → Failed`, `UNKNOWN → Pending`.
  - `can_transition_to()` guard: forbids regressing out of terminal states
    (`Delivered`/`Returned`/`Cancelled`) and backward hops
    (e.g. `Delivered → InTransit`); allows exception transitions.
  - **Unit tests for invariants** (AGENTS rule): full Shippo→internal mapping
    table, monotonic-progression guard, terminal-state immutability.
- Entities/value objects: `Shipment`, `ShipmentDraft` (carrier, service, parcel,
  ship_to/from), `TrackingEvent`, `LabelRef` (storage key + format).
- `FulfillmentError` (thiserror) — `LabelPurchaseFailed`, `ProviderUnavailable`,
  `ShipmentNotFound`, `InvalidTransition`, `MissingShipAddress`.
- Ports (`#[async_trait]`):
  - `ShipmentRepository` — create/find/list-by-order, append tracking event +
    apply status transition.
  - `ShippingProvider` — `purchase_label(draft) -> LabelPurchase`
    (transaction id, rate id, tracking number/url, label bytes-or-url, cost);
    optional `get_rates(...)` (see Open Decisions).
  - `LabelStore` — `put(key, bytes)`, `get(key)` over `common::storage::Storage`,
    scoped to the private label bucket/prefix.

## Application layer — `src/domains/fulfillment/application/`

Depends only on `domain/` ports. Naming per AGENTS (`{Action}{Resource}UseCase`):

- `PurchaseLabelUseCase` — load order + address, build `ShipmentDraft`, call
  `ShippingProvider::purchase_label`, store the label PDF via `LabelStore`,
  persist the `Shipment` (`status=LabelPurchased`, `label_storage_key`,
  `shippo_transaction_id`, tracking fields), write `shipment_items`, emit
  `audit::events::order_fulfilled` + `metrics::external_call("shippo","purchase_label",…)`.
- `RecordTrackingEventUseCase` — given a normalized event from the webhook,
  append a `tracking_events` row (idempotent on `source_event_id`), compute the
  `FulfillmentStatus` transition via the domain guard, update the shipment
  (`shipped_at`/`delivered_at` as applicable). Called by the webhooks handler.
- `GetShipmentUseCase` / `ListShipmentsForOrderUseCase` — admin + future account.
- `StreamLabelUseCase` — fetch label bytes via `LabelStore` for the authenticated
  admin label endpoint (never a public URL).

## Infra layer — `src/domains/fulfillment/infra/`

- `ShippoClient` (impl `ShippingProvider`) — uses the shared `state.http`
  reqwest client + `config.shippo.api_token`; wraps Shippo transactions/labels.
  Fetches the label PDF from Shippo's `label_url` and hands bytes to `LabelStore`.
  Every outbound call timed into `metrics::external_call("shippo", op, ms, ok)`.
- `SeaShipmentRepository` (impl `ShipmentRepository`) — SeaORM over the regenerated
  entities (after the deferred entity step; until then, this module is stubbed
  so the domain/application/api compile).
- `LabelObjectStore` (impl `LabelStore`) — thin adapter over `Storage` with the
  private label prefix.

## API layer — `src/domains/fulfillment/api/` (+ webhooks change)

Admin group (`require_supabase_admin`):
- `POST /api/v1/admin/orders/{order_id}/shipments` → `PurchaseLabelUseCase`
- `GET  /api/v1/admin/orders/{order_id}/shipments` → `ListShipmentsForOrderUseCase`
- `GET  /api/v1/admin/shipments/{id}` → shipment + tracking timeline
- `GET  /api/v1/admin/shipments/{id}/label` → `StreamLabelUseCase` (streams the
  PDF from storage through the authenticated endpoint; **no public URL**)

Handlers use `AuthUser` and DTOs only — never call infra directly (AGENTS rule).

Webhooks group (public, signature-verified) — **replace the current stub**
[webhooks/api/routes.rs:22](stellaux_server/src/domains/webhooks/api/routes.rs#L22):
- read the raw body, verify `x-shippo-signature` HMAC against
  `config.shippo.webhook_secret`, reject on mismatch;
- derive an `IdempotencyKey` from the Shippo event id; drop duplicates;
- parse + normalize the tracking payload and call
  `fulfillment::RecordTrackingEventUseCase`.
- Dependency direction stays clean: **webhooks api → fulfillment application**
  (ingress/idempotency owned by `webhooks`; business meaning owned by
  `fulfillment`). `fulfillment` never imports `webhooks`.

Future account endpoint (model-ready, lands with Phase 4 `account`):
`GET /api/v1/account/orders/{id}/tracking` over the same shipment/tracking model.

---

## Label storage security (must get right)

`storage-decision.md` scoped Supabase Storage to **product images** and
explicitly left labels open. Labels embed customer names/addresses, so:
- store label PDFs in a **private** bucket/prefix (e.g. `labels/shipments/{id}.pdf`),
  **not** the public product-image path;
- the DB holds only `label_storage_key`; never call `Storage::public_url` on it;
- operators retrieve labels through the authenticated
  `GET /admin/shipments/{id}/label` proxy (or a short-lived signed URL).

This likely needs a second storage scope/bucket alongside the public product
bucket — flagged as an Open Decision since it extends the storage contract.

---

## Deferred (out of scope now)

- Regenerating `src/entity/*` from `shipments.sql` and authoring the
  `src/migration/m*.rs` runner (done once SQL is approved).
- Rate-shopping / cheapest-rate selection across carriers.
- Customer-facing tracking endpoint (model is ready; ships with Phase 4).
- Return labels / label voiding (Shippo refunds) — add when ops needs it.

## New files expected

- `shared/models/shipments.sql`
- `src/domains/fulfillment/{mod,domain,application,infra,dto,api}/…`
  - `domain/{mod,fulfillment_status,shipment,ports,error}.rs`
  - `application/{mod,purchase_label,record_tracking_event,queries}.rs`
  - `infra/{mod,shippo_client,sea_shipment_repository,label_store}.rs`
  - `api/{mod,routes}.rs`, `dto/mod.rs`
- Register `pub mod fulfillment;` in `src/domains/mod.rs`; mount admin routes in
  `server.rs`; replace the `shippo` webhook stub.

## Validation after implementation

1. `cargo check` + `cargo clippy --lib` green.
2. `cargo test --lib` — `FulfillmentStatus` mapping + transition-guard tests pass
   (no DB/network needed; the policy lives in the pure domain).
3. Against Supabase (once schema applied): `shipments.sql` loads in FK order
   after `orders`/`order_items`; RLS enabled on all three tables.
4. Webhook path: a signed Shippo tracking payload creates exactly one
   `tracking_events` row; re-posting the same event id is a no-op (idempotent);
   a `DELIVERED` event drives the shipment to `delivered` and stamps
   `delivered_at`; a bad signature is rejected.
5. Label flow: `PurchaseLabelUseCase` stores a PDF reachable only via the
   authenticated admin endpoint; the public product URL builder is never used.

## Open Decisions (for plan review)

1. **Bounded context name**: `fulfillment` (recommended) vs `shipment`.
2. **Label storage**: a dedicated **private** bucket/scope + authenticated proxy
   (recommended; required for address PII) vs reusing the product bucket under a
   private prefix. Either way, extends the storage contract — confirm.
3. **Partial/split shipments**: include `shipment_items` now (recommended; cheap
   and future-proof) vs single-shipment-per-order for v1.
4. **Rate quoting**: build `ShippingProvider::get_rates` now for `checkout`
   reuse vs defer until Phase 6 (label purchase alone doesn't strictly need it if
   a fixed service level is used for marketplace fulfillment).
5. **Tracking idempotency**: rely on `tracking_events.source_event_id` unique now
   (recommended) and fold into the central `webhook_events` log when Phase 2/0
   introduces it.
```
