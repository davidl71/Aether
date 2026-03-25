# Health Aggregation

Unified health JSON now lives on the Rust backend and is driven directly by NATS `system.health`.

## Active endpoints

- `GET /health` — basic Rust backend health plus transport status
- `GET /api/health-aggregated` — aggregated backend health from Rust
- `GET /api/heartbeat` — compatibility alias for the same aggregated payload
- `GET /api/heartbeat/dashboard` — compatibility alias for the same aggregated payload
- `GET /api/heartbeat/{backend}` — single backend health when present in the Rust health map
- `GET /gateway/health` — lightweight gateway-compat status endpoint on Rust

## Current ownership

- Active Rust services publish `BackendHealth` on `system.health`
- `nats_adapter` owns the shared heartbeat publisher helper and the transport health DTO
- `api` owns the health DTO / aggregation model
- `backend_service` subscribes to `system.health` and exposes the aggregated REST health routes
- `tui_service` subscribes to the same stream and renders the component map in Settings
- There is no separate Python `health_dashboard` daemon in the active runtime

## Active component publishers

The active runtime should normally expose at least these service ids on `system.health`:

- `backend_service`
- `tui_service`
- `tws_yield_curve_daemon`

Each service heartbeat includes `updated_at`; Rust service publishers also attach static metadata
such as `pid` and service identity in `extra`.
The aggregate response also carries a first-class `transport` object for the NATS subscription
itself. That transport entry is marked stale/degraded when the last observed health message ages
past `HEALTH_STALE_AFTER_SECS` (default 45s).

## Notes

- `GET /api/health-aggregated` is the preferred machine-readable health route for clients and scripts.
- Older `/api/heartbeat/*` paths are tolerated legacy aliases, not the preferred surface for new work.
- Historical references to the old Python health dashboard should be treated as archive material only.
- `system.health` should represent long-lived service/process liveness and coarse degraded/error state.
  It should not be overloaded with provider selection, mock/demo mode, or snapshot-derived metrics that
  already belong to the snapshot/read-model path.
- NATS transport health belongs in the aggregate health DTO, not in the TUI or snapshot read model.
