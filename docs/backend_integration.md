# Backend Integration Plan

**Status:** Draft v1 (major revision — supersedes prior inventory notes)
**Architecture:** Self-hosted commerce backend on existing TanStack Start + Supabase stack
**Payments:** Stripe (Checkout Sessions + Webhooks)
**Shipping:** Shippo (rate quotes, label purchase, tracking)
**Multi-channel:** Designed to coexist with Etsy + eBay as parallel sales channels

---

## 1. Decision Rationale

After evaluating Shopify (headless), Medusa (self-hosted), and a fully custom Stripe-based backend, we are proceeding with the custom path.

| Constraint | Why Shopify was ruled out |
|---|---|
| **Recurring cost** | Basic plan at $39/mo + apps + 2.9% + 30¢ adds ~$500–1500/mo at launch volumes before any value is captured. |
| **Multi-channel reality** | We already sell on Etsy and eBay. Shopify wants to be the source of truth, and its sync to other marketplaces requires apps (Codisto, LitCommerce) at $20–80/mo that add fragility. With our own catalog in Supabase, all three channels (site + Etsy + eBay) consume the same inventory data without a middleman. |
| **Craft builder model** | Per-component compatibility rules fight Shopify's product/variant schema; the workaround is metafields + a bundle app, again a recurring cost. |
| **Data ownership** | Customer + order history stays in our Postgres, exportable and portable. No vendor lock-in. |

**What we're accepting:** ~8–12 weeks of engineering build, ongoing maintenance for inventory concurrency, refund flows, and admin UI. Trade is bought back because the *only* recurring fees are Stripe (transaction-based), Shippo (per-label), Supabase (~$25/mo), and Resend (~$20/mo).

---

## 2. Architecture Overview

```
                            ┌──────────────────────────────┐
                            │  Cloudflare Workers (Edge)   │
                            │  TanStack Start storefront   │
                            │  + API routes                │
                            └──────────────┬───────────────┘
                                           │
        ┌──────────────────────────────────┼──────────────────────────────────┐
        │                                  │                                  │
        ▼                                  ▼                                  ▼
┌──────────────┐                  ┌──────────────┐                  ┌──────────────┐
│   Supabase   │                  │    Stripe    │                  │    Shippo    │
│   Postgres   │  ◀──webhooks──▶  │  Checkout +  │  ◀──webhooks──▶  │   Labels +   │
│   + Auth     │                  │  PaymentInt. │                  │   Tracking   │
│   + Storage  │                  └──────────────┘                  └──────────────┘
└──────┬───────┘
       │
       ├── (future) Etsy API sync worker
       └── (future) eBay API sync worker
                                                      ┌──────────────┐
                                                      │    Resend    │
                                                      │ (txn emails) │
                                                      └──────────────┘
```

**Source of truth (catalog + inventory + customers):** Supabase Postgres
**Payment state of truth:** Stripe (mirrored to Postgres via webhook)
**Fulfillment state of truth:** Shippo (label + tracking mirrored to Postgres via webhook)

---

## 3. Tech Stack

| Layer | Service | Plan / Cost |
|---|---|---|
| Frontend | TanStack Start (existing) | n/a |
| Hosting | Cloudflare Workers | Free tier up to 100K req/day; ~$5/mo after |
| Database / Auth | Supabase | Free → $25/mo Pro at scale |
| Object storage (product images) | Supabase Storage or Cloudflare R2 | R2 ~$0 for typical e-comm volumes |
| Payments | Stripe Checkout Sessions | 2.9% + 30¢ per US card; Stripe Tax 0.5% optional |
| Shipping | Shippo | Pay-as-you-go $0.05/label; carrier costs at Commercial Plus rates |
| Transactional email | Resend | Free up to 3K emails/mo, $20/mo above |
| Address validation | Shippo (built-in) | Free |
| Analytics | Plausible or Cloudflare Web Analytics | $9/mo or free |
| Admin BI / SQL | Metabase (self-host) or none in v1 | Free / deferred |

**Estimated monthly cost at $50K GMV / 100 orders:**

| Item | Cost |
|---|---|
| Stripe (2.9% + 30¢) | ~$1,480 |
| Shippo labels (100 × $0.05) | $5 |
| Supabase Pro | $25 |
| Cloudflare Workers | $5 |
| Resend | $20 |
| **Total** | **~$1,535/mo** |

Versus Shopify Basic equivalent at the same volume: ~$1,520/mo running cost **plus** $30–150/mo in apps (bundles, marketplace sync). Roughly comparable on cash but with full ownership and no marketplace-sync fragility.

---

## 4. Schema Additions

All new tables under `public` schema with row-level security. Migrations authored as additive deltas to the existing schema (don't modify the existing 20260514 migration; add new ones).

### 4.1 Catalog

```sql
create table public.products (
  id uuid primary key default gen_random_uuid(),
  handle text not null unique,                  -- slug, used in URLs
  name text not null,
  description text,
  collection text,                              -- "Vol. I" / "Vol. II" / "Atelier"
  category text not null,                       -- rings | necklaces | earrings | bracelets
  material text not null,                       -- 18k Gold | Vermeil | Silver | Platinum
  status text not null default 'active',        -- active | archived | draft
  popularity int not null default 0,
  craft_role text,                              -- 'base' | 'accessory' | null
  craft_base_type text,                         -- 'pendant' | 'chain' | 'trunk' | null
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table public.product_variants (
  id uuid primary key default gen_random_uuid(),
  product_id uuid not null references public.products(id) on delete cascade,
  sku text not null unique,
  size text,                                    -- nullable for non-sized items
  price_cents int not null,
  weight_grams int not null,                    -- required for Shippo rate quotes
  dimensions_mm jsonb,                          -- { l, w, h }
  created_at timestamptz not null default now(),
  unique (product_id, size)
);

create table public.product_images (
  id uuid primary key default gen_random_uuid(),
  product_id uuid not null references public.products(id) on delete cascade,
  url text not null,
  alt text,
  position int not null default 0
);

create table public.inventory_levels (
  variant_id uuid primary key references public.product_variants(id) on delete cascade,
  on_hand int not null default 0,               -- physical units in warehouse
  reserved int not null default 0,              -- held by active checkout sessions
  -- available = on_hand - reserved (computed in app or generated column)
  updated_at timestamptz not null default now()
);

create table public.inventory_adjustments (
  id uuid primary key default gen_random_uuid(),
  variant_id uuid not null references public.product_variants(id),
  delta int not null,                           -- +N restock, -N shrinkage
  reason text not null,                         -- 'restock' | 'shrinkage' | 'return' | 'manual' | 'channel_sync'
  channel text,                                 -- 'website' | 'etsy' | 'ebay' | null
  actor_user_id uuid references auth.users(id),
  notes text,
  created_at timestamptz not null default now()
);

create table public.channel_listings (
  id uuid primary key default gen_random_uuid(),
  variant_id uuid not null references public.product_variants(id) on delete cascade,
  channel text not null,                        -- 'etsy' | 'ebay'
  external_listing_id text not null,
  last_synced_at timestamptz,
  sync_status text,                             -- 'ok' | 'pending' | 'error'
  unique (channel, external_listing_id)
);
```

**RLS:**
- `products`, `product_variants`, `product_images`: public read (`for select to anon, authenticated using (status = 'active')`)
- All writes: `has_role(auth.uid(), 'admin')` only
- `inventory_levels`: read public (for "in stock" badges); write admin or service role
- `inventory_adjustments`, `channel_listings`: read admin only

### 4.2 Craft compatibility

```sql
create table public.craft_compatibility (
  base_product_id uuid not null references public.products(id) on delete cascade,
  accessory_product_id uuid not null references public.products(id) on delete cascade,
  primary key (base_product_id, accessory_product_id)
);
alter table public.craft_compatibility enable row level security;
create policy "Compatibility public read" on public.craft_compatibility
  for select to anon, authenticated using (true);
```

Compatibility seeded from current hardcoded rules in [src/routes/craft.tsx](../src/routes/craft.tsx).

### 4.3 Carts

```sql
create table public.carts (
  id uuid primary key default gen_random_uuid(),
  user_id uuid references auth.users(id) on delete set null,  -- null for guest
  anonymous_token text unique,                                -- cookie value for guests
  currency text not null default 'USD',
  status text not null default 'open',          -- open | converted | abandoned
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table public.cart_items (
  id uuid primary key default gen_random_uuid(),
  cart_id uuid not null references public.carts(id) on delete cascade,
  variant_id uuid not null references public.product_variants(id),
  qty int not null check (qty > 0),
  unit_price_cents int not null,                -- snapshotted at add-to-cart
  assembly_id uuid,                             -- groups Craft components
  assembly_role text,                           -- 'base' | 'accessory' | null
  created_at timestamptz not null default now()
);
```

RLS: users see only their own carts (or via `anonymous_token` matching cookie); admin sees all.

### 4.4 Orders (extend existing)

The existing [orders + order_items](../supabase/migrations/20260514032902_315eb075-75c6-4a54-835d-5d4ffd972fa4.sql) cover the basics. Additive changes:

```sql
alter table public.orders
  add column stripe_payment_intent_id text unique,
  add column stripe_checkout_session_id text unique,
  add column shippo_order_id text,
  add column shippo_transaction_id text,        -- the purchased label
  add column tracking_number text,
  add column tracking_carrier text,
  add column tracking_url text,
  add column shipping_method text,              -- 'usps_priority' | 'ups_ground' | ...
  add column shipping_cost_cents int not null default 0,
  add column tax_cents int not null default 0,
  add column source text not null default 'website',   -- 'website' | 'etsy' | 'ebay'
  add column external_order_id text,            -- Etsy/eBay order id when source != website
  add column fulfillment_status text not null default 'unfulfilled',
                                                -- unfulfilled | fulfilled | partial | refunded
  add column notes text;

create index orders_source_external on public.orders(source, external_order_id);
create index orders_tracking on public.orders(tracking_number) where tracking_number is not null;

alter table public.order_items
  add column variant_id uuid references public.product_variants(id),
  add column assembly_id uuid,
  add column assembly_role text;
```

### 4.5 Webhook event log

For idempotency and debugging:

```sql
create table public.webhook_events (
  id uuid primary key default gen_random_uuid(),
  source text not null,                         -- 'stripe' | 'shippo' | 'etsy' | 'ebay'
  external_id text not null,                    -- provider's event id
  event_type text not null,
  payload jsonb not null,
  processed_at timestamptz,
  error text,
  received_at timestamptz not null default now(),
  unique (source, external_id)
);
```

Every webhook handler does an INSERT with `ON CONFLICT (source, external_id) DO NOTHING` first, then processes, then sets `processed_at`. Replays are safe.

---

## 5. Component Plan

### 5.1 Product catalog loader

**Replace** [src/data/products.ts](../src/data/products.ts) with TanStack Query loaders against Supabase.

**New files:**
- `src/lib/catalog.ts` — query builders (`loadProducts(filters)`, `loadProductByHandle`, `loadCollections`)
- `src/lib/catalog.types.ts` — TypeScript types matching the schema (auto-generate via `supabase gen types typescript`)

**Pages to update:**
- [src/routes/shop.index.tsx](../src/routes/shop.index.tsx) — use route loader + TanStack Query
- [src/routes/shop.$slug.tsx](../src/routes/shop.$slug.tsx) — load by handle; render variants for size selector
- [src/routes/craft.tsx](../src/routes/craft.tsx) — load bases + accessories filtered by `craft_role`

**Migration of existing 12 products:** one-time seed script that converts the current `seed` array in `products.ts` into INSERTs, authored as a new migration file.

### 5.2 Cart (server-side)

[src/context/CartContext.tsx](../src/context/CartContext.tsx) keeps its API surface but the implementation talks to Supabase via TanStack Start server functions:

| Operation | Server function |
|---|---|
| Read cart | `getCart()` — by user_id (if authed) or anonymous_token cookie |
| Add line | `addCartItem({ variantId, qty, assemblyId? })` — validates `available > 0`, snapshots price |
| Update qty | `updateCartItem({ itemId, qty })` |
| Remove line | `removeCartItem({ itemId })` |
| Clear | `clearCart()` — marks cart converted/abandoned |
| Merge on login | `mergeAnonymousCart(token)` — called once after auth state changes |

Anonymous token: signed cookie set on first cart action (HttpOnly, SameSite=Lax, 30-day expiry).

LocalStorage retained only as offline fallback / optimistic UI; server is source of truth.

### 5.3 Checkout flow

**Pre-checkout (our site):**
1. User clicks Checkout from cart drawer.
2. Routed to `/checkout` ([src/routes/checkout.tsx](../src/routes/checkout.tsx)).
3. Address form (logged-in users see saved addresses; guests enter once + optional save).
4. Server function `getShippingOptions({ cartId, address })`:
   - Computes total weight + dimensions from cart line items
   - Calls Shippo `shipments` API to get rate quotes (USPS Priority, UPS Ground, expedited)
   - Returns 2–3 service levels with cents-precise pricing
5. User selects shipping method.
6. Server function `createCheckoutSession({ cartId, addressId, shippoRateId })`:
   - **Reserves inventory** inside a DB transaction: `update inventory_levels set reserved = reserved + qty` for each line; advisory lock on each variant_id to serialize concurrent reservations
   - Creates Stripe Checkout Session with:
     - `line_items` = cart lines, prices in cents from variant snapshot
     - One additional `line_item` for shipping cost (or use `shipping_options`)
     - `shipping_address_collection` disabled (we already collected)
     - `metadata: { cart_id, address_id, shippo_rate_id }`
     - `payment_intent_data.metadata: { cart_id }`
     - `automatic_tax: { enabled: true }` if using Stripe Tax
     - `success_url`, `cancel_url`
   - Stores `stripe_checkout_session_id` on the cart row
   - Returns the Stripe-hosted URL
7. Redirect to Stripe Checkout.

**Post-payment:**
8. Stripe redirects back to `/order-confirmed?session_id=...`.
9. Webhook (asynchronously, see 5.4) is the authoritative source for order creation.
10. Confirmation page polls (or subscribes via Supabase Realtime on the orders table) until the order row appears.

**Cancel / expire path:**
- On `checkout.session.expired` (24h default) or `payment_intent.payment_failed`: release reserved inventory; mark cart abandoned.

### 5.4 Stripe webhook handler

**New file:** `src/routes/api.stripe.webhook.ts` (TanStack Start API route).

```
POST /api/stripe/webhook
```

Implementation outline:
1. Read raw body: `const body = await request.text();`
2. Verify signature with `stripe.webhooks.constructEvent(body, sig, STRIPE_WEBHOOK_SECRET)`.
3. INSERT into `webhook_events` with `ON CONFLICT DO NOTHING`. If conflict, return 200 (already processed).
4. Switch on event type:

| Event | Action |
|---|---|
| `checkout.session.completed` | Create `orders` row + `order_items` rows from cart; decrement `inventory_levels.on_hand` and `reserved`; mark cart `converted`; trigger Shippo order creation; queue order confirmation email |
| `checkout.session.expired` | Release reserved inventory; mark cart `abandoned` |
| `payment_intent.payment_failed` | Release reservations; log; no email by default |
| `charge.refunded` | Update `orders.fulfillment_status = 'refunded'`; restock inventory if pre-fulfillment; queue refund email |
| `charge.dispute.created` | Notify admin via email; do not auto-refund |

5. Update `webhook_events.processed_at`. On error, set `error` column and return 500 so Stripe retries (Stripe retries up to 3 days).

**Idempotency:** all DB writes guarded by the unique `webhook_events` row + `orders.stripe_checkout_session_id` unique constraint.

**Worker compatibility note:** use Stripe Node SDK with `httpClient: Stripe.createFetchHttpClient()` and `cryptoProvider: Stripe.createSubtleCryptoProvider()`.

### 5.5 Shippo integration

**Three points of contact:**

**1. Rate quote at checkout** (synchronous, see 5.3)
- Endpoint: `POST https://api.goshippo.com/shipments/`
- Input: `address_from` (warehouse, from env vars), `address_to` (customer), `parcels` (computed total weight + box dims)
- Output: `rates[]` — filter to a preset of 2–3 service levels (e.g., USPS Priority, UPS Ground, FedEx Express)
- Surface to user with carrier logo + estimated delivery date + price

**2. Label purchase after payment** (asynchronous, triggered by `checkout.session.completed`)
- New module: `src/integrations/shippo/labels.ts` exposes `purchaseLabel(orderId)`
- Looks up `orders.shippo_rate_id` (stored from checkout session metadata)
- Calls Shippo `POST /transactions` with `rate` + `label_file_type: 'PDF_4x6'`
- On success, stores `shippo_transaction_id`, `tracking_number`, `tracking_carrier`, `tracking_url` on `orders`
- Downloads label PDF, uploads to Supabase Storage (or R2) bucket `shipping-labels/{order_id}.pdf`
- Queues `ShippingNotification` email
- On failure, retries up to 3 times with exponential backoff; if still failing, sets `orders.fulfillment_status = 'unfulfilled'` and emails admin

**3. Tracking webhook**
- Endpoint: `POST /api/shippo/webhook` (new TanStack Start API route)
- Verify signature: HMAC-SHA256 with `SHIPPO_WEBHOOK_SECRET` over raw body; compare with `crypto.subtle` constant-time
- Event of interest: `track_updated`
- Map Shippo's `tracking_status.status` → our `orders.status`:
  - `PRE_TRANSIT` → `paid` (no change)
  - `TRANSIT` → `shipped`
  - `DELIVERED` → `delivered`
  - `RETURNED`, `FAILURE` → admin alert
- On `delivered`, queue `DeliveredFollowup` email scheduled for +3 days (use a cron table or schedule a Cloudflare Cron Trigger)

**Manifest / batch printing** (admin convenience): deferred to v2; admin can print labels individually for now.

### 5.6 Admin UI

New route group: `src/routes/_admin/*` gated by middleware checking `has_role(uid, 'admin')`. Admin layout reuses [Header.tsx](../src/components/Header.tsx) shell with an admin-only nav.

**v1 admin screens:**

| Route | Purpose |
|---|---|
| `/admin` | Dashboard: today's orders, revenue, low-stock variants |
| `/admin/products` | List products; CRUD; image upload; metafield editor for `craft_role` |
| `/admin/products/$id` | Edit product + variants + per-variant inventory + dimensions |
| `/admin/inventory` | Bulk view + adjust on-hand counts; reason-coded `inventory_adjustments` |
| `/admin/orders` | List with filters: status, source (website/etsy/ebay), date, tracking carrier |
| `/admin/orders/$id` | Order detail: line items, customer, shipping address, payment, label PDF link, tracking; actions: refund, cancel, mark fulfilled, reprint label, add note |
| `/admin/customers` | List + detail; lifetime value; recent orders |
| `/admin/craft` | Manage `craft_compatibility` pairs |
| `/admin/channels` | View `channel_listings`; trigger manual sync; see sync errors |
| `/admin/settings` | Warehouse address, carrier preferences, tax settings |

**Reporting:** punt on dashboard charts in v1. Wire Metabase to Supabase later for BI.

### 5.7 Transactional emails

**Provider:** Resend. Templates authored in `react-email`.

**New module:** `src/lib/email.ts` exposes `sendEmail({ to, template, data })`.

Templates (in `src/emails/`):

| Template | Trigger |
|---|---|
| `OrderConfirmation` | After `checkout.session.completed` |
| `ShippingNotification` | After Shippo label purchased; includes tracking link |
| `DeliveredFollowup` | 3 days after Shippo `delivered` event |
| `RefundConfirmation` | After `charge.refunded` |
| `LowStockAdminAlert` | Cron: variants with `available < 3` |
| `PasswordReset` | Existing Supabase Auth flow (already handled by Supabase) |

### 5.8 Returns / RMA

**v1 scope:** manual via admin UI.
- Customer emails support → admin creates a return in `/admin/orders/$id`
- Admin generates a Shippo return label (`is_return: true` on `POST /transactions`)
- Email label PDF to customer
- On scan-in: admin marks return received, optionally restocks inventory (`inventory_adjustments` with `reason='return'`), processes Stripe refund via order detail page

**v2:** customer-facing self-service RMA portal.

### 5.9 Craft builder integration

[src/routes/craft.tsx](../src/routes/craft.tsx) refactor:
- Load bases + accessories from `products` filtered by `craft_role`
- Load compatibility from `craft_compatibility` (TanStack Query with low staleTime)
- Each base + accessory has a real `product_variants` row with SKU + inventory
- "Add assembly to bag" generates a client-side UUID `assembly_id`, then calls `addCartItem` once per component with the shared `assembly_id` and appropriate `assembly_role`

**Cart UI grouping:** [CartDrawer.tsx](../src/components/CartDrawer.tsx) groups lines by `assembly_id` for display: "Custom Assembly · 1 base + 2 accessories" with a collapsed component list and a combined price.

**Fulfillment:** packer sees individual SKUs in the order plus an `assembly_id` annotation in admin UI; physically combines components into one package; one Shippo label per order.

---

## 6. Multi-channel (Etsy + eBay)

**Principle:** Supabase is the inventory and product source of truth; Etsy and eBay are *also* sales channels with their own listings that must reflect our inventory.

The `channel_listings` and `inventory_adjustments` tables (4.1) and the `orders.source` + `external_order_id` columns (4.4) are designed to support this from day one.

**v1 (manual sync, ship at launch)**
- Maintain SKUs identically across all three channels (our `product_variants.sku` matches the Etsy + eBay listing SKU)
- When an order arrives on Etsy/eBay, admin manually records it via `/admin/orders/new` with `source = 'etsy' | 'ebay'` and `external_order_id`
- After fulfillment, admin manually adjusts inventory in the other channels' dashboards
- The admin UI exposes a "Currently listed on" indicator per variant via `channel_listings`

**v2 (one-way push, ~2 weeks build)**
- Cloudflare Cron Trigger every 15 min: detect inventory deltas (Supabase changefeed or `updated_at` since last sync), push to Etsy `updateInventory` API + eBay `reviseInventoryStatus`
- Direction: we push, marketplaces pull from us
- Failures logged in `webhook_events` with `source='etsy_sync'` for replayability

**v3 (full bidirectional, ~3 weeks build)**
- Listen to Etsy + eBay order webhooks; auto-insert into `orders` with appropriate `source`
- Decrement inventory on every channel order regardless of source
- This is where a tool like LitCommerce becomes worth its keep if engineering time is constrained — defer until volume justifies

**No schema changes required between v1 and v2/v3.** The data model is already shaped for the bidirectional case; we are just deferring the connector code.

---

## 7. Phased Implementation

Estimate based on one engineer, full-time. Add ~25% if part-time.

| Phase | Scope | Estimate |
|---|---|---|
| **Phase 1: Catalog** | Migrate hardcoded products to Supabase; product/variant/inventory schema; loader functions; refactor shop pages | 1 week |
| **Phase 2: Server-side cart** | Cart schema; server functions; refactor `CartContext`; anonymous cookie + login merge | 4 days |
| **Phase 3: Stripe Checkout** | Server functions; address form on `/checkout`; redirect to hosted checkout; success/cancel routing | 3 days |
| **Phase 4: Shippo rates** | `getShippingOptions` server fn; rate display UI on checkout page | 3 days |
| **Phase 5: Stripe webhook + orders** | Webhook endpoint; signature verification; order creation; inventory decrement; idempotency | 4 days |
| **Phase 6: Shippo label purchase + tracking webhook** | Triggered from Stripe webhook; tracking webhook handler; order status updates | 3 days |
| **Phase 7: Transactional emails** | Resend setup; 4 core templates; trigger wiring | 3 days |
| **Phase 8: Admin UI** | Route gating; products CRUD; orders list + detail; inventory adjustments; refund button | 2 weeks |
| **Phase 9: Craft refactor** | Load from DB; cart integration with `assembly_id`; admin compatibility editor | 4 days |
| **Phase 10: Returns flow (manual)** | Admin RMA button; Shippo return label; refund + restock | 2 days |
| **Phase 11: QA + soft launch** | End-to-end test orders; tax verification; mobile checkout; intl shipping; webhook replay testing | 1 week |
| **Total** | | **~10 weeks** |

Multi-channel automation (Etsy/eBay push, Section 6 v2) is explicitly out of v1 scope — add 2–3 weeks when prioritized.

---

## 8. Environment Variables

To be added to `.env`, Cloudflare Workers secrets (`wrangler secret put`), and Supabase config:

```bash
# Existing
SUPABASE_URL=...
SUPABASE_PUBLISHABLE_KEY=...
SUPABASE_SERVICE_ROLE_KEY=...                 # server-only
VITE_SUPABASE_URL=...
VITE_SUPABASE_PUBLISHABLE_KEY=...

# Stripe
STRIPE_SECRET_KEY=sk_live_...                 # server-only
STRIPE_PUBLISHABLE_KEY=pk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...               # server-only
VITE_STRIPE_PUBLISHABLE_KEY=pk_live_...

# Shippo
SHIPPO_API_TOKEN=shippo_live_...              # server-only
SHIPPO_WEBHOOK_SECRET=...                     # server-only

# Resend
RESEND_API_KEY=re_...                         # server-only
RESEND_FROM_EMAIL=orders@themaisonaure.com

# Warehouse (origin address for Shippo rates)
WAREHOUSE_NAME=...
WAREHOUSE_STREET=...
WAREHOUSE_CITY=...
WAREHOUSE_STATE=...
WAREHOUSE_POSTAL_CODE=...
WAREHOUSE_COUNTRY=US
WAREHOUSE_PHONE=...
```

---

## 9. Security Considerations

- **Stripe webhook signature verification** must use the raw request body bytes, not a parsed object. On Cloudflare Workers, use `await request.text()` for the body. Use `Stripe.createFetchHttpClient()` + `Stripe.createSubtleCryptoProvider()` when constructing the Stripe SDK on Workers.
- **Shippo webhook signature:** HMAC-SHA256 with `SHIPPO_WEBHOOK_SECRET` over raw body; compare via constant-time function backed by `crypto.subtle`.
- **Service role key** (`SUPABASE_SERVICE_ROLE_KEY`) never crosses the client/server boundary; only used in TanStack Start server functions and API routes ([client.server.ts](../src/integrations/supabase/client.server.ts)).
- **Admin gating:** all `_admin/*` routes use `requireSupabaseAuth` middleware + a `has_role(uid, 'admin')` server check. Never trust client-side role state.
- **Inventory reservations** prevent oversell under concurrent checkout. Use Postgres `select ... for update` inside the reservation transaction; consider advisory locks per variant_id for high-contention SKUs.
- **Idempotency keys:** every webhook handler is keyed on `(source, external_id)` in `webhook_events`. Replay is safe.
- **PII:** do not log full shipping addresses or card details. Stripe gives us `last4` and brand for display; that's enough.
- **Cart cookie:** signed (HMAC) with a server-side secret, HttpOnly, SameSite=Lax. Anonymous token is not security-sensitive but should be unguessable.

---

## 10. Open Questions

1. **Who is operating the admin day-to-day?** If non-technical founder, the admin UI investment in Phase 8 needs more polish (bulk actions, search, drafts). If engineer-founder, current scope is fine.
2. **Tax strategy:** Stripe Tax (0.5% per order) handles nexus + filing automatically. Alternative is TaxJar. Default to Stripe Tax unless TaxJar is preferred for multi-channel reconciliation.
3. **International shipping at launch:** US only first, or EU + Canada day 1? Affects Phase 4 scope (customs forms via Shippo) and Phase 8 (tax per region).
4. **3PL or self-fulfillment:** are orders shipping from a home/studio or a 3PL? If 3PL (e.g., ShipBob), Shippo may be replaced by direct 3PL integration. Affects Phase 6 design.
5. **Warranty / repair workflow:** the brand promises "lifetime polish & re-plating service." Does that need a workflow in admin (intake form, repair status), or handled manually via email for now?
6. **Loyalty / referrals:** not in v1 scope; flag for v2.
7. **Etsy + eBay listing volume:** how many SKUs are listed across both today? Determines whether v1 manual sync is sustainable or v2 must ship sooner.

---

## 11. Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Inventory oversell under concurrent checkout | Medium | Reservation pattern with DB transaction + advisory lock; expire reservations on session timeout (24h) |
| Stripe webhook missed or out-of-order | Low | Stripe retries up to 3 days; idempotent handlers via `webhook_events` table; replay via Stripe dashboard |
| Shippo label purchase fails after payment captured | Medium | Retry on failure with backoff; admin alert if 3 retries fail; manual label fallback in admin UI |
| Multi-channel inventory drift in v1 (manual sync) | High | Manual SOP for ops; daily audit query (`select sku, on_hand from inventory_levels` reconciled against Etsy/eBay dashboards); ship v2 push sync as soon as practical |
| Edge runtime incompatibility (Node-only libs) | Medium | Stripe + Shippo + Resend SDKs are fetch-based and Worker-compatible; avoid `crypto` Node module — use `crypto.subtle` |
| Refund flow bugs costing real money | Medium | Refund button gated behind confirmation modal; all refunds logged with `actor_user_id`; only admin role |
| Initial catalog data quality | Medium | One-time CSV review before migration; image alt text, SEO, dimensions/weights filled before launch |
| Cloudflare Workers cold start on rate quote | Low | Workers cold starts are <5ms; the actual latency is Shippo's rate API (~500–800ms) — show skeleton state |

---

## 12. Out of Scope (v1)

Explicitly deferred — listed so they're not forgotten:

- Discount codes / promotions engine
- Gift cards
- Subscription / recurring orders
- Multi-currency display (USD only at launch)
- B2B / wholesale pricing tiers
- Abandoned cart recovery emails
- Product reviews / UGC
- Self-service RMA portal
- Mobile admin app
- Inventory forecasting / reorder suggestions
- Etsy + eBay auto-sync (covered in Section 6 v2/v3)
- Custom Stripe Checkout / embedded Payment Element (sticking with hosted)

---

## 13. Rollout

1. **Build behind a flag.** Keep current hardcoded catalog and placeholder checkout live until full backend is QA'd.
2. **Staging environment:** separate Supabase project + Stripe test mode + Shippo test mode for end-to-end testing.
3. **Soft launch sequence:**
   - Day 0: deploy backend, internal test orders only (real Stripe, refund immediately)
   - Day 1–3: friends & family test, real shipping to 3–5 addresses
   - Day 4: enable for all visitors
4. **Rollback plan:** keep the hardcoded-catalog branch deployable; checkout placeholder is innocuous if reverted to.

---

## 14. Appendix — Existing Code Touchpoints

Files to be modified or replaced:

| File | Action |
|---|---|
| [src/data/products.ts](../src/data/products.ts) | Delete; replace with `src/lib/catalog.ts` |
| [src/context/CartContext.tsx](../src/context/CartContext.tsx) | Rewrite to call server functions; keep API surface |
| [src/routes/shop.index.tsx](../src/routes/shop.index.tsx) | Refactor data source |
| [src/routes/shop.$slug.tsx](../src/routes/shop.$slug.tsx) | Refactor data source |
| [src/routes/craft.tsx](../src/routes/craft.tsx) | Refactor to load from DB |
| [src/routes/checkout.tsx](../src/routes/checkout.tsx) | Replace placeholder with real flow |
| [src/routes/_authenticated/account.tsx](../src/routes/_authenticated/account.tsx) | Keep; backed by extended orders/addresses schema |
| [src/integrations/lovable/index.ts](../src/integrations/lovable/index.ts) | Keep (OAuth wrapper still useful) |

Files to be added:

```
src/
├── integrations/
│   ├── stripe/
│   │   ├── client.ts                  # Stripe client (server-only)
│   │   └── webhooks.ts                # Event handlers
│   ├── shippo/
│   │   ├── client.ts
│   │   ├── rates.ts
│   │   ├── labels.ts
│   │   └── webhooks.ts
│   └── resend/
│       └── client.ts
├── lib/
│   ├── catalog.ts                     # product loaders
│   ├── cart.ts                        # cart server functions
│   ├── inventory.ts                   # reservation + decrement helpers
│   └── email.ts                       # sendEmail wrapper
├── emails/
│   ├── OrderConfirmation.tsx          # react-email components
│   ├── ShippingNotification.tsx
│   ├── DeliveredFollowup.tsx
│   └── RefundConfirmation.tsx
└── routes/
    ├── api.stripe.webhook.ts
    ├── api.shippo.webhook.ts
    ├── order-confirmed.tsx
    └── _admin/
        ├── route.tsx                  # admin layout + role gate
        ├── index.tsx
        ├── products/
        ├── orders/
        ├── inventory.tsx
        ├── craft.tsx
        ├── channels.tsx
        └── settings.tsx
```

---

**Last updated:** 2026-05-15
