## Table `users`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `email` | `text` |  Unique |
| `password_hash` | `text` |  |
| `full_name` | `text` |  Nullable |
| `created_at` | `timestamptz` |  |
| `last_login_at` | `timestamptz` |  Nullable |

## Table `email_tokens`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `email` | `text` |  |
| `token` | `text` |  Unique |
| `expires_at` | `timestamp` |  |
| `used_at` | `timestamp` |  Nullable |
| `created_at` | `timestamp` |  Nullable |

## Table `session`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `anonymous_token` | `text` |  Unique |
| `created_at` | `timestamptz` |  |
| `expires_at` | `timestamptz` |  Nullable |
| `last_seen_at` | `timestamptz` |  |

## Table `guest_profiles`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `email` | `text` |  |
| `session_id` | `uuid` |  Unique |
| `device_fingerprint` | `text` |  Nullable |
| `created_at` | `timestamptz` |  |
| `converted_to_user_id` | `uuid` |  Nullable |

## Table `categories`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `slug` | `text` |  Unique |
| `name` | `text` |  |
| `size_unit` | `text` |  |
| `sort_order` | `int4` |  |
| `created_at` | `timestamptz` |  |
| `updated_at` | `timestamptz` |  |

## Table `category_size_options`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `category_id` | `uuid` |  |
| `size_value` | `numeric` |  |
| `label` | `text` |  Nullable |
| `sort_order` | `int4` |  |

## Table `collections`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `slug` | `text` |  Unique |
| `name` | `text` |  |
| `description` | `text` |  Nullable |
| `created_at` | `timestamptz` |  |
| `updated_at` | `timestamptz` |  |

## Table `products`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `handle` | `text` |  Unique |
| `name` | `text` |  |
| `description` | `text` |  Nullable |
| `category_id` | `uuid` |  |
| `default_material` | `text` |  Nullable |
| `tags` | `_text` |  |
| `status` | `text` |  |
| `created_at` | `timestamptz` |  |
| `updated_at` | `timestamptz` |  |

## Table `product_collections`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `product_id` | `uuid` | Primary |
| `collection_id` | `uuid` | Primary |

## Table `product_variants`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `product_id` | `uuid` |  |
| `material` | `text` |  |
| `design` | `text` |  Nullable |
| `type_label` | `text` |  |
| `size_value` | `numeric` |  Nullable |
| `sku` | `text` |  Unique |
| `barcode` | `text` |  Nullable |
| `price_cents` | `int8` |  |
| `cost_cents` | `int8` |  Nullable |
| `weight_grams` | `int4` |  Nullable |
| `dimensions` | `text` |  Nullable |
| `status` | `text` |  |
| `created_at` | `timestamptz` |  |
| `updated_at` | `timestamptz` |  |

## Table `product_media`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `product_id` | `uuid` |  |
| `variant_id` | `uuid` |  Nullable |
| `storage_key` | `text` |  |
| `kind` | `text` |  |
| `alt_text` | `text` |  Nullable |
| `position` | `int4` |  |
| `created_at` | `timestamptz` |  |

## Table `orders`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `user_id` | `uuid` |  Nullable |
| `guest_profile_id` | `uuid` |  Nullable |
| `order_number` | `text` |  Unique |
| `status` | `text` |  |
| `currency` | `text` |  |
| `subtotal_cents` | `int8` |  |
| `tax_cents` | `int8` |  |
| `shipping_cents` | `int8` |  |
| `total_cents` | `int8` |  |
| `total` | `int4` |  |
| `created_at` | `timestamptz` |  |
| `updated_at` | `timestamptz` |  |
| `shipping_address` | `jsonb` |  |
| `billing_address` | `jsonb` |  |
| `placed_at` | `timestamptz` |  Nullable |
| `shipped_at` | `timestamptz` |  Nullable |
| `paid_at` | `timestamptz` |  Nullable |

## Table `order_items`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `order_id` | `uuid` |  |
| `product_id` | `uuid` |  |
| `variant_id` | `uuid` |  Nullable |
| `sku` | `text` |  |
| `name` | `text` |  |
| `quantity` | `int4` |  |
| `unit_price_cents` | `int8` |  |
| `total_cents` | `int8` |  |

## Table `channel_listings`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `uuid` | Primary |
| `channel` | `text` |  |
| `external_listing_id` | `text` |  |
| `external_variant_id` | `text` |  Nullable |
| `external_sku` | `text` |  Nullable |
| `product_id` | `uuid` |  |
| `variant_id` | `uuid` |  Nullable |
| `raw` | `jsonb` |  Nullable |
| `status` | `text` |  |
| `last_synced_at` | `timestamptz` |  Nullable |
| `created_at` | `timestamptz` |  |
| `updated_at` | `timestamptz` |  |

## Table `user_roles`

Authorization role per Supabase user id (sub). Absence of a row ⇒ customer. Auth_Contract §4.1.

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `user_id` | `uuid` | Primary |
| `role` | `text` |  |
| `created_at` | `timestamptz` |  |
| `updated_at` | `timestamptz` |  |

## Table `kernel_isolation_probe`

### Columns

| Name | Type | Constraints |
|------|------|-------------|
| `id` | `int4` |  Nullable |

