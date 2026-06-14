# Storage Decision

## Decision

Use Supabase Storage as the selected product-image storage system.

Supabase Postgres remains the operational source of truth for catalog, inventory, orders, customers, webhook state, and audit state. Supabase Storage is the binary store for product images, with image metadata and references still recorded in Postgres.

## Why This Is The Current Recommendation

### Best fit for internal tooling

The internal dashboard and operations flows will benefit from keeping product-image metadata and delivery on the same platform family as the relational data:

- product records live in Supabase Postgres
- product image files live in Supabase Storage
- internal tools can reason about image references, upload state, and permissions without introducing a second external storage platform decision right now

### Simpler operational boundary for Phase 0 and Phase 1

The current roadmap is backend-first and operations-first. Using Supabase for both relational data and product-image storage simplifies:

- environment setup
- admin/internal tooling integration
- image URL and bucket policy management
- future dashboard workflows for product maintenance

### Good enough coupling for this asset class

Product images are a strong candidate for tighter platform coupling because:

- they are long-lived catalog assets
- they are read frequently by internal tooling and storefront surfaces
- they benefit from consistent access patterns more than maximum backend portability

## Scope Of This Decision

This decision is finalized for:

- product images
- product-related display assets closely tied to catalog management

This document does not force the same choice for every binary artifact forever. Shipping labels, temporary exports, or other operational files may still justify a different storage posture later if their access pattern or privacy model differs.

## Contract Rule

Regardless of provider choice, the application contract should stay the same:

- store only object keys and metadata in Postgres
- keep provider details behind the storage abstraction
- generate public URLs through backend configuration rather than hard-coding vendor paths
- treat storage as a binary asset system, not the owner of catalog or order semantics

## Data Flow Contract

For product images, the intended data flow is:

1. internal tooling or admin API uploads an image
2. the API stores the binary in Supabase Storage
3. the API stores the bucket/object key, ordering, alt text, and related metadata in Postgres
4. storefront and internal tools read product/image metadata from Postgres
5. clients resolve the final asset URL through the storage configuration contract

This keeps relational truth in Postgres while letting Supabase Storage own the file bytes themselves.
