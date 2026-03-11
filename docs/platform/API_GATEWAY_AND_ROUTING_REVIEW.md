# API Gateway Retirement Review

**Last updated**: 2026-03-11
**Purpose**: capture the retirement state of `api-gateway` and the resulting default routing model.

## Decision

`api-gateway` is retired.

- **Rust** owns frontend read models.
- **Rust** owns `LIVE_STATE` read/watch endpoints for clients.
- **Rust** owns the client-facing heartbeat proxy path, `/api/health-aggregated`, and `/gateway/health`.
- **Python** health dashboard remains as the current internal `system.health` aggregation service.
- **Python** remains behind explicit service boundaries only where Python-specific logic still exists.

## Default frontend routing model

### Preferred default

- client -> shared Rust origin for application APIs
- Rust -> health dashboard only for operational health aggregation when needed

### Dev override mode

- direct `VITE_*_PORT` values remain supported for local debugging and service-by-service development
- direct per-port browser wiring is **not** the recommended default topology

## Practical default path model

- snapshot and frontend read models: Rust origin
- unified health: Rust/shared path by default
- heartbeat aggregation: Rust-exposed path backed by the existing health dashboard service

## Follow-up implementation direction

1. Keep Rust as the default client-facing origin.
2. Do not reintroduce a separate Go gateway process.
3. Decide later whether the Python health dashboard should also move into Rust or remain a separate internal service behind Rust-owned routes.
