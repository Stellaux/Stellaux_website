# Inventory Management Strategy

## Short answer

Yes. The uniform solution is to make **one system the source of truth for inventory** and have every sales channel sync to it.

For this project, the cleanest source of truth is **Supabase**, with:

- this website reading live available stock from Supabase
- Etsy, eBay, and other marketplaces sending orders into Supabase
- Supabase pushing updated stock counts back out to every channel

Do not let each marketplace maintain its own independent inventory count.

## Why the current setup will drift

Right now:

- product data is stored statically in [`src/data/products.ts`](/Users/guchaill/Coding/the-polished-standard/src/data/products.ts:1)
- orders exist in Supabase, but inventory tables do not yet exist in the schema
- checkout is not yet writing inventory changes

That means the storefront can show products, but there is not yet a central stock ledger that all channels can trust.

## Recommended architecture

### 1. Single inventory authority

Create a central inventory model in Supabase. Every channel should map to the same internal SKU.

Core rule:

- `available = on_hand - reserved - safety_buffer`

This prevents overselling during payment, sync delay, or cancellation windows.

### 2. Core tables

Suggested tables:

- `inventory_items`
  - `id`
  - `sku`
  - `product_id`
  - `variant_key`
  - `title`
  - `is_active`

- `inventory_levels`
  - `inventory_item_id`
  - `on_hand`
  - `reserved`
  - `safety_buffer`
  - `available`
  - `updated_at`

- `channel_listings`
  - `inventory_item_id`
  - `channel` (`website`, `etsy`, `ebay`, `facebook`)
  - `channel_listing_id`
  - `channel_sku`
  - `last_synced_available`
  - `sync_status`

- `inventory_movements`
  - `id`
  - `inventory_item_id`
  - `type` (`sale`, `reservation`, `release`, `restock`, `manual_adjustment`, `cancellation`)
  - `quantity`
  - `source` (`website`, `etsy`, `ebay`, `facebook`, `admin`)
  - `reference_id`
  - `created_at`

- `channel_orders`
  - imported external orders for deduping and reconciliation

### 3. Order flow

When any order happens on any channel:

1. Import the order into Supabase.
2. Match each line item to an internal SKU.
3. Create inventory movements.
4. Reduce available stock centrally.
5. Queue sync jobs to update every other channel.

The website should never trust a locally cached stock value if live inventory is available.

### 4. Sync model

Use a small integration worker or edge function for each channel:

- inbound sync: marketplace order webhook or scheduled polling -> Supabase
- outbound sync: Supabase inventory change -> marketplace quantity update

Important behavior:

- make sync jobs idempotent
- log every sync attempt
- retry failures with backoff
- alert when a channel cannot be updated

## Best-practice rules

### SKU discipline

Every sellable unit needs one canonical SKU. This is the most important requirement.

If Etsy listing names, eBay listing names, and website product names differ, that is fine. They still must map to the same internal SKU.

### Reservation window

Reserve stock when checkout starts or when payment intent is created, then release it if checkout expires.

This matters most for low-stock items.

### Safety buffer

Hold back 1-2 units or a small percentage from marketplaces if sync delays are common.

Example:

- actual on hand: `10`
- reserved: `1`
- safety buffer: `1`
- available to channels: `8`

### Reconciliation

Run a scheduled reconciliation job at least every 15-30 minutes:

- compare channel-reported stock vs Supabase stock
- compare imported channel orders vs internal orders
- flag mismatches for review

## What this means for this website

For this codebase, the practical implementation path is:

1. Move sellable inventory out of static-only product data and into Supabase-backed inventory records.
2. Add SKU/variant mapping for each product.
3. Update checkout so successful orders create inventory movements.
4. Add admin or service-layer sync jobs for Etsy/eBay/Facebook/other channels.
5. Update product and cart UI to read live `available` stock before purchase.

## Recommended rollout

### Phase 1

- add inventory schema
- add SKU mapping
- website reads stock from Supabase
- website orders decrement stock

### Phase 2

- import marketplace orders
- push updated counts back to channels
- add retries and reconciliation

### Phase 3

- add reservations
- add safety buffer rules
- add admin dashboard for exceptions and manual adjustments

## Recommendation

If you want the simplest reliable answer: **Supabase should own the inventory number, and every marketplace should sync against it rather than against each other.**

That is the uniform solution for including this website in the same inventory system.
