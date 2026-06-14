# API Contract Surface

## Goal

Keep website and dashboard teams aligned on stable backend seams without letting frontend implementation order redefine backend ownership.

## Stable Surface For Early Parallel Work

### Public-facing

- catalog browsing contract
- auth session contract
- future cart and checkout entrypoints

### Operational/internal

- order lookup contract
- shipment status contract
- customer-service lookup contract
- webhook health and replay contract

## Backend-First Rule

Frontend teams may depend on:

- route shapes
- DTO contracts
- error envelope conventions
- authorization expectations

Frontend teams may not assume:

- ownership of database shape
- provider-specific side effects
- alternate business rules per surface

The first-party website and internal dashboard must consume the same backend business contracts that marketplace operations use.
