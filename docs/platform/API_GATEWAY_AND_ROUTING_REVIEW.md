# API Gateway and Web Routing Review

**Last updated**: 2026-03-11
**Purpose**: capture the post-simplification scope of `api-gateway` and define the default web routing model.

## api-gateway scope

### Keep

- aggregated gateway health
- `LIVE_STATE` KV read endpoints
- operational routing for integration backends that are still legitimately separate services
- compatibility entrypoint for the TUI when one gateway URL is preferable to multiple backend URLs

### Do not regrow

- removed Python analytics/calculations proxy paths
- generic pass-through proxying for frontend read models already owned by Rust
- duplicate API surfaces that add no ownership or operational value

## Current decision

`api-gateway` should stay **operationally focused**, not become the primary place where frontend business APIs live.

- **Rust** owns frontend read models.
- **Go gateway** owns operational aggregation and selected routing convenience.
- **Python** remains behind explicit service boundaries only where Python-specific logic still exists.

## Default web routing model

### Preferred default

- browser -> shared origin (nginx / common base URL)
- shared origin -> Rust API, Go gateway, and selected Python services by path

### Dev override mode

- direct `VITE_*_PORT` values remain supported for local debugging and service-by-service development
- direct per-port browser wiring is **not** the recommended default topology

## Practical default path model

- snapshot and frontend read models: Rust origin
- unified health: shared-origin `/api/health-aggregated` when available
- specialist services such as risk-free-rate: path-routed behind the shared origin when deployed together

## Follow-up implementation direction

1. Keep `VITE_API_URL` as the main default knob.
2. Keep per-service ports as optional overrides.
3. Prefer shared-origin examples in docs, env examples, and setup guidance.
