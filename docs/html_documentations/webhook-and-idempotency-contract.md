# Webhook And Idempotency Contract

## Purpose

All external event sources must follow one ingestion contract so Stripe, Shippo, eBay, and Etsy can be handled consistently.

## Webhook Lifecycle

1. receive raw request body and provider signature headers
2. identify provider source
3. verify signature against provider secret or public key contract
4. derive external event id and idempotency key
5. persist technical receipt record
6. deduplicate against prior events
7. dispatch business processing
8. mark processed or failed
9. emit audit events when business state changes

## Three Distinct Records

### Webhook event log

Technical receipt record for replay safety and provider reconciliation.

Stores:

- source
- external event id
- event type
- raw payload reference or payload snapshot strategy
- status
- processing timestamps
- failure reason

### Idempotency record

Prevents duplicate side effects.

Stores:

- normalized idempotency key
- scope
- source
- payload fingerprint

### Audit log

Business-facing event trail.

Stores:

- action name
- actor
- subject
- business metadata

Audit records must not be treated as webhook deduplication records, and webhook logs must not be treated as business audit history.

## Source Naming Contract

Normalized provider names:

- `stripe`
- `shippo`
- `ebay`
- `etsy`

## Status Contract

Minimum lifecycle:

- `received`
- `processed`
- `failed`

## Idempotency Scope Contract

Common idempotency scopes:

- `http_request`
- `webhook_event`
- `background_job`

Later phases may add provider-specific conventions, but they must map back to one of those operational scopes.
