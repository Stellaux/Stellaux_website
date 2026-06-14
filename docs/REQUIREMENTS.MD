# Stellaux — Requirements

> **Status:** Canonical source of truth for product scope, actors, and use-case flows.
> When this document and any other doc disagree, **this document wins** and the other
> doc should be corrected. See [§9 Known Inconsistencies](#9-known-inconsistencies) for
> the current reconciliation backlog.
>
> **Last updated:** 2026-06-12

---

## 1. Overview

Stellaux is a jewelry-retailing e-commerce platform targeting the low-to-mid price
segment. The current collection centers on gold and silver elements for chains,
bracelets, rings, and other pieces. The differentiating business model is **heavy
customization** — customers assemble pieces from compatible components (a base plus
accessories/gemstones) through a dedicated "craft" builder.

Stellaux is also a **multi-channel** seller: the first-party website is one channel
alongside Etsy and eBay. Supabase Postgres is the single source of truth for catalog,
inventory, and customers; every channel reads stock from and writes orders back to it.

### Tech context

| Concern | Choice |
|---|---|
| Backend | Rust · Axum · SeaORM (`stellaux_server/`) |
| Data / Auth / Storage | Supabase Postgres + Supabase Auth |
| Payments | Stripe (Checkout Sessions + webhooks) |
| Shipping | Shippo (rate quotes, labels, tracking) |
| Transactional email | Resend |
| Storefront | TanStack Start (separate; consumes the backend REST API) |
| Internal dashboard | Separate repo; consumes the same backend contracts |

API conventions (full detail in [docs/API_Guidelines.md](API_Guidelines.md)):
REST-first, URL-versioned under `/api/v1/...`, documented code-first via utoipa/OpenAPI,
Swagger UI served at `/docs`.

---

## 2. Actors & Roles

Authorization roles are resolved from internal Postgres data, not from the identity
provider. Role lookup defaults to **least privilege**; a missing or malformed role row
must never escalate. (Source: [auth-contract.md](html_documentations/auth-contract.md).)

| Role | Who | Capabilities |
|---|---|---|
| `guest` | Unauthenticated visitor | Browse catalog, use the craft builder, hold an anonymous cart, check out |
| `customer` | Authenticated shopper | Everything a guest can do, plus saved profile, addresses, and order history |
| `support` | Support staff | Read customer/order data to handle contact tickets |
| `staff` | Operations staff | Fulfillment and catalog operations (subset of admin) |
| `admin` | Privileged operator | All privileged actions: order edits, refunds, catalog/inventory CRUD, role elevation, audit log |

> **Note:** Earlier docs used the term "employee"; the canonical role vocabulary is the
> four roles above (`customer` / `support` / `staff` / `admin`) plus the unauthenticated
> `guest`.

### Identity model

- Human users authenticate through **Supabase**.
- Route middleware verifies the caller and stamps a normalized claim set into the request.
- The stable caller contract downstream domains depend on is: `user_id`, `role`,
  `identity provider`, and intended `audience` when relevant — never raw JWT internals.
- Server-issued JWTs are allowed only for service-owned/transitional flows, not as the
  primary path for human admin access.

---

## 3. Website Pages

The first-party storefront has five required pages:

| Page | Purpose | State |
|---|---|---|
| Home | Hero carousel of trending items (slow-scroll transitions) + a secondary carousel of hot items | Design set apart from the hero |
| Store | Product browsing, filtering, product detail with variant/size selection | Mostly set; input-field refinements pending |
| Story | Brand narrative | — |
| Craft | Custom-assembly builder (base + compatible accessories) | Requires heavy restructuring |
| Contact | Support / feedback entry point that opens a ticket | — |

> The storefront route uses `shop` (e.g. `shop.index`, `shop.$slug`) for what this
> document calls the **Store** page; treat "Store" and "shop" as the same surface.

---

## 4. Modules (Bounded Contexts)

Server responsibilities, reconciled with the implemented `stellaux_server/src/domains/*`
contexts and the cross-cutting `src/common/` platform layer.

### Cross-cutting platform — `src/common/`

Owns config, bootstrap/runtime wiring, shared auth middleware, JWT/JWKS primitives,
storage abstraction, error translation, and audit + idempotency primitives. Owns **no**
product, order, shipment, or customer business rules.

### Domains

| Module | Owns | Key endpoints (implemented or contracted) |
|---|---|---|
| **auth** | Login, signup, forgot/reset password; session contract; role-aware identity | `POST /auth/login`, `/auth/signup`, `/auth/forgot-password`, `/auth/reset-password` |
| **account** | Customer-facing views over profile, addresses, and order history | `GET/PATCH /me`, `POST /me/password`, `GET /orders`, `GET /orders/{id}`, `GET/POST /addresses`, `PATCH/DELETE /addresses/{id}` |
| **catalog** | Products, variants, collections, categories, product media | `GET /products`, `/products/{handle}`, `/collections`, `/categories` |
| **inventory** | Stock levels, reservation rules, adjustments, availability state | Read surfaces via catalog ("in stock"); writes via admin |
| **craft** | Custom-assembly builder: bases, accessories, compatibility rules, assembly grouping | `GET /bases`, `/accessories`, `/compatibility/{base_handle}` |
| **cart** | Cart + line items; anonymous-token sessions; merge-on-login | `GET /cart`, `DELETE /cart`, `POST /cart/items`, `PATCH /cart/items/{id}`, `DELETE /cart/items/{id}` |
| **checkout** | Shipping-rate quoting, checkout-session creation, inventory reservation, Stripe handoff | `POST /checkout/shipping-rates`, `POST /checkout/session` |
| **orders** | Order normalization, line items, status transitions, totals, source/channel semantics | Created via webhook; read via account/admin |
| **admin** | Privileged HTTP workflows across contexts (orders, products, inventory, fulfillment), audit logging, role elevation — orchestrates through domain services, does not own the rules | `/admin/orders*`, `/admin/products*`, `/admin/inventory*`, `/admin/compatibility*`, `/admin/channel-listings` |
| **webhooks** | External event ingestion (Stripe, Shippo; future channel sync); idempotent processing | `POST /webhooks/stripe`, `POST /webhooks/shippo` |
| **support** *(planned)* | Contact/feedback tickets from the site; opens a ticket for the support team | Contracted, not yet implemented |
| **sync** *(planned)* | Multi-channel catalog/inventory reconciliation with Etsy/eBay, initiated from the internal dashboard | Manual in v1; automated push in v2 (see [§7](#7-multi-channel)) |

> **Layering target** (`api → application → domain → infra`; source:
> [bounded-context-map.md](html_documentations/bounded-context-map.md)): `api/` may depend
> on `application/`, `dto/`, and `common`; `application/` on `domain/`; `infra/` implements
> `domain` ports. Today only `auth` and `webhooks` realize the full layering; the other
> domains currently expose `api/` + `dto/` only and will grow the inner layers as business
> logic lands. Marketplace adapters and the storefront must target these same contexts
> rather than forking parallel models.

---

## 5. Cross-Cutting Rules

These hold across every use case below.

- **Inventory authority.** `available = on_hand − reserved − safety_buffer`. Supabase owns
  the number; the website must read live `available` before purchase and must not trust a
  locally cached value when live inventory is reachable.
- **SKU discipline.** Every sellable unit has exactly one canonical internal SKU. Etsy/eBay
  listing names may differ but must map to the same SKU.
- **Reservation window.** Stock is reserved when a checkout session is created and released
  if the session expires or payment fails.
- **Ownership guard.** Any handler loading a user-owned resource by id must either scope the
  query by `user_id` or call an ownership guard (`ensure_owns(owner_id)`), regardless of
  whether the caller is a Supabase user or a service JWT.
- **Webhook idempotency.** Every webhook is keyed on `(source, external_id)` in
  `webhook_events`; handlers `INSERT … ON CONFLICT DO NOTHING` first, process, then stamp
  `processed_at`. Replays are safe.
- **PII.** Never log full shipping addresses or card data; Stripe `last4` + brand is enough
  for display.

---

## 6. Core Use Cases & Flows

Each flow lists actor, preconditions, the main path, and the notable alternate/error paths.

### UC-1 · Browse catalog

- **Actor:** guest or customer
- **Precondition:** none
- **Main flow:**
  1. Visitor opens the Store page → `GET /api/v1/products` (optionally filtered by
     collection/category/material).
  2. Backend returns active products with variants and live `available` per variant.
  3. Visitor opens a product → `GET /api/v1/products/{handle}`; the size selector is
     rendered from `product_variants`.
- **Alternate:** out-of-stock variants render an "out of stock" state and cannot be added.
- **Postcondition:** none (read-only).

### UC-2 · Build a custom assembly (Craft)

- **Actor:** guest or customer
- **Precondition:** craft-role products and compatibility rules exist
- **Main flow:**
  1. Visitor opens Craft → `GET /api/v1/bases` and `GET /api/v1/accessories`
     (products filtered by `craft_role`).
  2. Visitor picks a base → `GET /api/v1/compatibility/{base_handle}` returns the
     accessories compatible with that base.
  3. Visitor selects compatible accessories; UI validates each against the compatibility set.
  4. "Add assembly to bag" mints a client-side `assembly_id` and calls **UC-3 add-item**
     once per component, sharing that `assembly_id` and stamping `assembly_role`
     (`base` / `accessory`).
- **Alternate:** an incompatible accessory is rejected client-side before add-to-cart.
- **Postcondition:** cart contains a grouped assembly displayed as one combined line.

### UC-3 · Cart management

- **Actor:** guest (anonymous token) or customer (`user_id`)
- **Precondition:** none; a cart is created lazily on first action
- **Main flow:**
  - Read cart → `GET /api/v1/cart` (resolved by `user_id` if authed, else anonymous-token
    cookie).
  - Add line → `POST /api/v1/cart/items` `{ variant_id, qty, assembly_id?, assembly_role? }`;
    backend validates `available > 0` and **snapshots `unit_price_cents`** at add time.
  - Update qty → `PATCH /api/v1/cart/items/{id}`.
  - Remove line → `DELETE /api/v1/cart/items/{id}` (releases any reservation tied to the line).
  - Clear → `DELETE /api/v1/cart` (releases all reservations).
- **Sessions:** the anonymous token is a signed (HMAC) `HttpOnly`, `SameSite=Lax`,
  30-day cookie set on first cart action. LocalStorage is an optimistic-UI fallback only;
  the server is source of truth.
- **Merge on login:** on auth-state change, the anonymous cart is merged into the user's
  cart exactly once.
- **Postcondition:** persisted cart survives page refresh and login.

### UC-4 · Checkout & payment

- **Actor:** guest or customer with a non-empty cart
- **Precondition:** every line's variant has `available ≥ qty`
- **Main flow:**
  1. Customer enters/selects a shipping address.
  2. `POST /api/v1/checkout/shipping-rates` `{ cart_id, address }` → backend computes total
     weight/dimensions from line items, calls Shippo, returns 2–3 service levels with
     cents-precise pricing.
  3. Customer selects a shipping method.
  4. `POST /api/v1/checkout/session` `{ cart_id, address_id, shippo_rate_id }`, in one DB
     transaction:
     - **Reserves inventory** (`reserved += qty` per line) using a per-`variant_id` lock to
       serialize concurrent reservations.
     - **Creates the order row as `pending`** with snapshotted line items, shipping method,
       and totals.
     - Creates a Stripe Checkout Session (line items from price snapshots + a shipping line;
       `metadata { cart_id, order_id, address_id, shippo_rate_id }`), stores
       `stripe_checkout_session_id` on the order, returns the Stripe-hosted URL.
  5. Customer is redirected to Stripe Checkout and pays.
  6. On success Stripe redirects to the confirmation page, which polls/subscribes until the
     order flips to `paid` (the **webhook**, UC-5, is the authority for that transition).
- **Alternate / error:**
  - `checkout.session.expired` (default 24h) or `payment_intent.payment_failed` → release
    reservations, mark the order `cancelled`, mark cart `abandoned`.
  - A variant goes out of stock between cart and reservation → reservation fails, no order is
    created, and the customer is asked to adjust the cart.
- **Postcondition:** a `pending` order exists with inventory reserved, awaiting payment confirmation.

> **Order-creation rule:** the order row **is created at checkout in a `pending` state**,
> with inventory reserved. The **Stripe webhook** on `checkout.session.completed`
> (UC-5) is the authority that transitions it `pending → paid` and decrements on-hand stock.
> The webhook never creates orders from scratch — it locates the pending order by
> `stripe_checkout_session_id`. See [§9](#9-known-inconsistencies) item 8.

### UC-5 · Order fulfillment (webhook-driven)

- **Actor:** Stripe and Shippo (system actors), then `staff`/`admin`
- **Precondition:** a `pending` order + checkout session exist (UC-4)
- **Main flow (Stripe `checkout.session.completed`):**
  1. Verify signature over the **raw** body; `INSERT webhook_events … ON CONFLICT DO NOTHING`
     (duplicate → return 200 immediately).
  2. Locate the pending order by `stripe_checkout_session_id`; transition it `pending → paid`;
     convert reserved stock to a sale (decrement `inventory_levels.on_hand` and `reserved`);
     mark the cart `converted`. Idempotent on the order's already-`paid` state.
  3. Purchase a Shippo label (async, retried with backoff): store
     `shippo_transaction_id`, `tracking_number`, `tracking_carrier`, `tracking_url`; upload
     the label PDF to storage.
  4. Queue the `OrderConfirmation` email; after label purchase, queue `ShippingNotification`.
  5. Stamp `webhook_events.processed_at`.
- **Tracking (Shippo `track_updated`):** map carrier status →
  `order.status` (`TRANSIT→shipped`, `DELIVERED→delivered`; `RETURNED`/`FAILURE` → admin
  alert). On delivery, queue a `DeliveredFollowup` (+3 days).
- **Error paths:** label purchase fails after 3 retries → mark `fulfillment_status`
  unfulfilled + email admin (manual label fallback in admin UI). Any handler error →
  set `webhook_events.error` and return 500 so the provider retries.
- **Postcondition:** the order is `paid` with tracking; inventory decremented exactly once.

### UC-6 · Account & order history

- **Actor:** customer
- **Precondition:** authenticated
- **Main flow:**
  - View/update profile → `GET/PATCH /api/v1/me`; change password → `POST /api/v1/me/password`.
  - List/get orders → `GET /api/v1/orders`, `GET /api/v1/orders/{order_id}`
    (scoped to the caller via the ownership guard).
  - Manage addresses → `GET/POST /api/v1/addresses`, `PATCH/DELETE /api/v1/addresses/{id}`.
- **Postcondition:** profile/address changes persisted; reads return only the caller's data.

### UC-7 · Authentication

- **Actor:** guest
- **Main flow:**
  - Sign up → `POST /api/v1/auth/signup`; log in → `POST /api/v1/auth/login` (Supabase
    identity; role resolved from internal storage, defaulting to least privilege).
  - Forgot password → `POST /api/v1/auth/forgot-password`; reset → `POST /api/v1/auth/reset-password`.
- **Postcondition:** an authenticated session with a normalized caller claim set; on login,
  the anonymous cart merges (UC-3).

### UC-8 · Admin — order operations

- **Actor:** `admin` (some reads available to `staff`/`support`)
- **Precondition:** authenticated with sufficient role; gating is server-side, never trusting
  client role state
- **Main flow:**
  - List/filter orders → `GET /api/v1/admin/orders` (status, source, date, carrier);
    detail → `GET /api/v1/admin/orders/{order_id}`.
  - Refund → `POST …/refund` (confirmation-gated; logs `actor_user_id`; restocks if
    pre-fulfillment; queues `RefundConfirmation`).
  - Cancel → `POST …/cancel`; reprint label → `POST …/labels`.
- **Postcondition:** order state changes are audit-logged.

### UC-9 · Admin — catalog & inventory operations

- **Actor:** `admin` / `staff`
- **Main flow:**
  - Products CRUD → `GET/POST /api/v1/admin/products`,
    `GET/PATCH/DELETE …/products/{id}` (incl. variants, images, `craft_role`).
  - Inventory → `GET /api/v1/admin/inventory`, `POST …/inventory` to adjust on-hand with a
    reason-coded `inventory_adjustment` (`restock`/`shrinkage`/`return`/`manual`/`channel_sync`).
  - Craft compatibility → `GET/POST …/compatibility`, `DELETE …/compatibility/{…}`.
  - Channel listings → `GET …/channel-listings`.
- **Postcondition:** catalog/inventory updated; adjustments recorded for audit and channel sync.

### UC-10 · Support / contact ticket *(planned)*

- **Actor:** guest or customer
- **Main flow:** visitor submits feedback/contact from the Contact page → a new open support
  ticket is created for the support team to handle.
- **Status:** contracted in this document; no backend domain implemented yet (see
  [§9](#9-known-inconsistencies) item 3).

### UC-11 · Multi-channel sync *(planned)*

- **Actor:** internal dashboard (separate repo) / scheduled worker
- **Main flow (v1, manual):** an Etsy/eBay order is recorded with `source` and
  `external_order_id`; inventory is adjusted across channels by an operator.
- **Main flow (v2, automated push):** a scheduled job detects inventory deltas and pushes
  them to Etsy/eBay; failures are logged in `webhook_events` for replay.
- **Status:** see [§7](#7-multi-channel).

---

## 7. Multi-Channel

Supabase is the inventory/product source of truth; Etsy and eBay are additional sales
channels that must reflect our inventory.

- **v1 (manual, ships at launch):** identical SKUs across channels; operators record
  marketplace orders with `source` + `external_order_id` and adjust other channels manually.
- **v2 (one-way push):** scheduled worker pushes inventory deltas to Etsy/eBay; failures
  logged for replay.
- **v3 (bidirectional):** ingest marketplace order webhooks, auto-insert orders, decrement
  inventory on every channel order.

The schema (`channel_listings`, `inventory_adjustments`, `orders.source` /
`external_order_id`) already supports the bidirectional case; only connector code is
deferred. Detailed plan: [backend_integration.md §6](backend_integration.md).

---

## 8. Out of Scope (v1)

Discount/promo engine, gift cards, subscriptions, multi-currency display, B2B/wholesale
tiers, abandoned-cart recovery emails, product reviews/UGC, self-service RMA portal, mobile
admin app, inventory forecasting, Etsy/eBay auto-sync (v2/v3), and custom/embedded Stripe
Checkout. Full list: [backend_integration.md §12](backend_integration.md).

---

## 9. Known Inconsistencies

Reconciliation backlog found while unifying the docs. Each item names the canonical
resolution this document adopts; satellite docs should be corrected to match.

1. **Module list drift.** `REQUIREMENTS` (old) listed account/admin/products/catalog/cart/
   order/chat/sync; `ARCHITECTURE.md` listed only account/admin/cart (truncated). The
   implemented domains are auth, account, catalog, cart, checkout, craft, admin, webhooks
   (plus empty `recommendation/` and `user/` dirs). **Canonical:** [§4](#4-modules-bounded-contexts).
2. **`products` vs `catalog`.** Old requirements listed both as separate modules. **Canonical:**
   a single **catalog** context owns products + variants + collections + categories;
   **inventory** is its own context. The old line "catalog: inventory for tracking available
   orders" conflated the two and is replaced.
3. **`chat`/support module missing in code.** Requirements promise a contact/feedback ticket
   flow, but there is no `support`/`chat` domain. **Canonical:** kept as **planned** (UC-10).
4. **`craft` missing from old module lists.** The craft builder is central to the business
   model and is implemented, yet absent from the old `REQUIREMENTS`/`ARCHITECTURE` module
   lists. **Canonical:** craft is a first-class module ([§4](#4-modules-bounded-contexts), UC-2).
5. **`auth` folded into `account`.** Old docs treated auth as part of account; code and the
   auth contract treat **auth** as its own context. **Canonical:** separate (UC-7).
6. **Brand name.** Three names appear: **Stellaux** (requirements, architecture, server code),
   **Maison Aure** (`backend_integration.md` → `orders@themaisonaure.com`), and
   **the-polished-standard** (repo dir). **Canonical:** the product/brand is **Stellaux**;
   `backend_integration.md` should update the placeholder email. *Confirm with owner if
   "Maison Aure" is an intentional public-facing brand.*
7. **Role vocabulary.** Old requirements said "admin, employee user"; the auth contract
   defines `customer`/`support`/`staff`/`admin`. **Canonical:** the four-role set + `guest`
   ([§2](#2-actors--roles)); "employee" is retired.
8. **Order lifecycle.** Old requirements implied an order is created **pending** at checkout
   and flipped to **paid** after Stripe validation; `backend_integration.md` instead created
   the order **only** in the `checkout.session.completed` webhook. **Canonical (resolved):**
   the order row **is created `pending` at checkout** with inventory reserved (UC-4); the
   webhook transitions it `pending → paid` and decrements on-hand stock (UC-5). The webhook
   never creates an order from scratch — it locates the pending order by
   `stripe_checkout_session_id`. `backend_integration.md §5.3/§5.4` must be updated to the
   pending-at-checkout model.
9. **Inventory schema drift.** `inventory_management.md` proposes `inventory_items` +
   `inventory_levels` (with `safety_buffer`, `available`) + `inventory_movements` +
   `channel_orders`; `backend_integration.md` proposes `inventory_levels` keyed by
   `variant_id` + `inventory_adjustments` + `channel_listings`. **Canonical (resolved):** the
   **`inventory_management.md` shape** — `inventory_items` / `inventory_levels`
   (`safety_buffer`, computed `available`) / `inventory_movements` / `channel_orders` — with
   `available = on_hand − reserved − safety_buffer` ([§5](#5-cross-cutting-rules)).
   `backend_integration.md §4.1` must be reconciled to this shape (rename `inventory_adjustments`
   → `inventory_movements`, add `inventory_items` + `safety_buffer`/`available`).
10. **`docs/server.md` missing.** Both old `REQUIREMENTS` and `ARCHITECTURE` reference
    `docs/server.md`, which does not exist. **Canonical:** server detail lives in
    [API_Guidelines.md](API_Guidelines.md), [ARCHITECTURE.md](ARCHITECTURE.md), and the
    `html_documentations/` set; the dangling `server.md` reference is removed.
11. **Documented layering vs reality.** `bounded-context-map.md` describes a four-layer
    `api/application/domain/infra` structure, but only `auth` and `webhooks` implement it
    today. **Canonical:** the layering is the **target**; [§4](#4-modules-bounded-contexts)
    notes the current partial state so the doc isn't read as describing shipped code.
12. **Duplicated/garbled prose.** `CLIENT.md` repeats the pages list twice; the old
    `REQUIREMENTS` had a broken cart line ("adding p / products to cart") and several typos
    ("jewlery", "privledged", "referesh"). Cleaned here; the satellite files should be
    de-duplicated.

---

## 10. Reference Documents

- [API_Guidelines.md](API_Guidelines.md) — REST/OpenAPI conventions, versioning, utoipa
- [ARCHITECTURE.md](ARCHITECTURE.md) — high-level architecture *(overlaps this doc; trim to architecture-only)*
- [CLIENT.md](CLIENT.md) — storefront/page specifications
- [backend_integration.md](backend_integration.md) — full backend build plan (Stripe/Shippo/Resend, schema, phasing)
- [inventory_management.md](inventory_management.md) — inventory authority strategy
- [html_documentations/](html_documentations/) — bounded-context map, auth contract, API contract surface, ownership matrices, observability
