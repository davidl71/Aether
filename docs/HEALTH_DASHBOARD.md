# Health Dashboard

Unified health JSON for services and dashboards, driven by NATS `system.health` updates from backends.

## Overview

- **Service:** `python/services/health_dashboard.py` (FastAPI)
- **Port:** `HEALTH_DASHBOARD_PORT` (default **8011** when run with other services; app default 8010)
- **Endpoints:**
  - `GET /api/health` — Aggregated JSON: `backends`, `backends_list`, `all_ok`, `any_error`, `nats_connected`, `generated_at`
  - `GET /api/health/dashboard` — Same payload
  - `GET /api/health/{backend_id}` — Single backend health (404 if not yet reported)

## Behind the shared web server (nginx)

With `config/nginx/backend-services.conf` loaded, or through the Rust backend:

- **`GET http://localhost:8080/api/health-aggregated`** — Proxies to the health dashboard `/api/health` (machine-readable aggregated health for services/dashboards).
- **`GET http://localhost:8080/api/heartbeat`** — Rust-owned compatibility/proxy path to the same aggregated health payload.

## How backends appear

Backends must have **`NATS_URL`** set and publish to **`system.health`** on each health check. The health dashboard subscribes to `system.health` and keeps the latest payload per `backend` id.

| Backend        | Publishes when           |
|----------------|--------------------------|
| IB             | `NATS_URL` set           |
| Alpaca         | `NATS_URL` set           |
| Tastytrade     | `NATS_URL` set           |
| Discount Bank  | `NATS_URL` set           |
| Risk-Free Rate | `NATS_URL` set           |

## Running

- **Standalone:** `./scripts/run_health_dashboard.sh` or `./scripts/service.sh start healthdashboard`
- **Start all services:** `./scripts/start_all_services.sh` (or `./scripts/service.sh start-all`) now includes the health dashboard after NATS and backends.
- **Supervisord:** `[program:health_dashboard]` in `config/supervisord.conf` (port 8011)
- **Systemd:** `config/systemd/user/ib-box-spread-health_dashboard.service`
- **service_manager.sh:** `./scripts/service_manager.sh start health_dashboard` or `start-all` (health_dashboard is in the service list).

If you use a shared config file (`config/config.json` or `IB_BOX_SPREAD_CONFIG`), add under `services` so the TUI uses the dashboard:

```json
"health_dashboard": { "port": 8011 }
```

(See `config/config.example.json` for the full services block.)

## Dev iteration: reload when binary changes

To restart a service (or run a command) whenever the C++ build output changes, use:

- **`./scripts/dev_watch_binary.sh`** — Watches `build/`; on change prints a restart reminder.
- **`./scripts/dev_watch_binary.sh -- './scripts/service.sh restart ib'`** — Runs that command on change.
- **`./scripts/dev_watch_binary.sh --path build/macos-arm64-debug/bin/ib_box_spread -- 'just build && ./scripts/service.sh restart ib'`** — Custom path and command.

Uses **fswatch** (macOS: `brew install fswatch`) or **inotifywait** (Linux: inotify-tools); falls back to polling every 2s if neither is installed.

## Consuming the JSON

Use **`GET /api/health`** (direct on port 8011) or **`GET /api/health-aggregated`** (via nginx on 8080) for:

- **status** — `ok` if all reported backends are ok, else `error` or `degraded`
- **backends** — Map of backend_id → last health payload (with `updated_at`)
- **backends_list** — List of backend ids
- **all_ok** / **any_error** — Booleans
- **nats_connected** — Whether the dashboard is connected to NATS

## TUI and PWA

- **TUI:** The TUI uses the shared API origin at `{api_base_url}/api/health` for unified health.
- **PWA:** When `VITE_API_URL` points at the shared server (e.g. `http://localhost:8080/api`), the PWA fetches `GET /api/health-aggregated` once per interval and maps `backends` to header status badges. Set `VITE_HEALTH_AGGREGATED_URL` to point at the dashboard directly (e.g. `http://localhost:8011/api/health`) when not using nginx.

## References

- `docs/NATS_SETUP.md` — NATS health publishing and bridge setup
- `docs/NATS_TOPICS_REGISTRY.md` — `system.health` payload shape
