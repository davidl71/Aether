# Health Aggregation

Unified health JSON now lives on the Rust backend and is driven directly by NATS `system.health`.

## Active endpoints

- `GET /health` — basic Rust backend health
- `GET /api/health-aggregated` — aggregated backend health from Rust
- `GET /api/heartbeat` — compatibility alias for the same aggregated payload
- `GET /api/heartbeat/dashboard` — compatibility alias for the same aggregated payload
- `GET /api/heartbeat/{backend}` — single backend health when present in the Rust health map
- `GET /gateway/health` — lightweight gateway-compat status endpoint on Rust

## Current ownership

- Backends publish `BackendHealth` or `NatsEnvelope(BackendHealth)` on `system.health`
- Rust subscribes to `system.health` and keeps the latest payload per backend
- There is no separate Python `health_dashboard` daemon in the active runtime

## Notes

- `GET /api/health-aggregated` is the preferred machine-readable health route for clients and scripts.
- Older `/api/heartbeat/*` paths remain as compatibility aliases.
- Historical references to the old Python health dashboard should be treated as archive material only.
