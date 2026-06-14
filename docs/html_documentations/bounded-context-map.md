# Bounded Context Map

## Cross-Cutting Platform

### `src/common/`

Owns:

- config
- bootstrap/runtime wiring
- shared auth middleware
- shared JWT/JWKS primitives
- storage abstraction
- error translation
- audit and idempotency primitives

Does not own product, order, shipment, or customer business rules.

## Active And Planned Contexts

### `auth`

Owns user-facing auth flows, auth contracts, and role-aware session semantics.

### `webhooks`

Owns external event ingestion contracts and technical event processing boundaries.

### `catalog`

Owns products, variants, collections, categories, and product media semantics.

### `inventory`

Owns stock levels, reservation rules, adjustments, and availability state.

### `orders`

Owns order normalization, line items, status transitions, totals, and source/channel semantics.

### `shipment` / `fulfillment`

Owns labels, tracking, delivery state, and carrier-facing shipment lifecycle.

### `account`

Owns customer-facing views over profiles, orders, and addresses once customer services are implemented.

### `admin`

Owns HTTP/admin workflows across contexts but should orchestrate through domain/application services rather than owning the business rules itself.

## Integration Direction

- `api/` may depend on `application/`, `dto/`, and `common`
- `application/` may depend on `domain/`
- `infra/` may implement `domain` ports
- `domain/` defines business-facing contracts and models

Marketplace adapters and the eventual website must target the same bounded contexts instead of creating separate parallel models.
