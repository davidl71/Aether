# Publish/subscribe messaging investigation and implementation

## Current state

| Component | Role | Details |
|-----------|------|---------|
| **NATS server** | Core/JetStream | `config/nats-server.conf`; started via `scripts/service.sh` or supervisord. Default `nats://localhost:4222`. |
| **Publishers** | Rust backend, C++ (when `ENABLE_NATS`), Python strategy | Market data (`market-data.tick.{symbol}`), strategy signals/decisions (`strategy.signal.*`, `strategy.decision.*`). |
| **Subscribers** | PWA (nats.ws), Go nats-questdb-bridge, Health Dashboard | Market data → QuestDB; strategy topics for UI; Health Dashboard subscribes to `system.health` and serves unified `/api/health` JSON. |
| **Python NATS** | `python/integration/nats_client.py` | `nats-py` (async); connect, subscribe, publish strategy signals/decisions. No backend snapshot/health publish yet. |

**Gap:** Backend services (IB, Alpaca, TradeStation, etc.) do **not** publish snapshots or health to NATS. TUI and PWA poll REST. Adding publish gives a path to subscribe instead of poll and to build an aggregator.

---

## Topic alignment (registry)

See `docs/NATS_TOPICS_REGISTRY.md`. Added for this implementation:

- **`snapshot.{backend_id}`** — Backend snapshot payload (e.g. `snapshot.ib`, `snapshot.alpaca`). Publisher: backend service. Subscribers: TUI/PWA provider, snapshot aggregator. Payload: JSON snapshot (same shape as REST `/api/snapshot`).
- **`system.health`** — Health status from any backend. Payload: `{"backend": "ib", "status": "ok", "ts": "...", ...}`. One topic so subscribers get all backends; filter by `backend` in payload.

---

## Implementation plan (started)

### 1. NATS client: publish snapshot and health

- **File:** `python/integration/nats_client.py`
- **Add:** Module-level lazy NATS connection; `publish_snapshot(backend_id: str, payload: dict)` and `publish_health(backend_id: str, status: dict)` (async). Subject `snapshot.{backend_id}` and `system.health`; payload JSON. No-op if NATS not available or connect fails.
- **Usage:** Backends call `asyncio.create_task(nats_client.publish_snapshot("ib", payload))` after building snapshot so publish is fire-and-forget and does not block the request.

### 2. IB service: publish on snapshot and health

- **File:** `python/integration/ib_service.py`
- **Enable when:** `NATS_URL` is set (IB service then publishes snapshot and health).
- **Snapshot:** After building and caching payload (and writing file if configured), call `asyncio.create_task(publish_snapshot("ib", payload))`.
- **Health:** After building health response, call `asyncio.create_task(publish_health("ib", health_dict))`.

### 3. Snapshot subscriber (proof-of-concept)

- **File:** `python/integration/nats_snapshot_subscriber.py`
- **Behavior:** Subscribe to `snapshot.>` and `system.health`; keep last snapshot per backend and last health per backend in memory; optional `--serve-port PORT` to expose last snapshot via HTTP (requires aiohttp).
- **Run:** `uv run python -m python.integration.nats_snapshot_subscriber [--nats-url nats://localhost:4222] [--serve-port 9010]`

### 4. Health Dashboard (unified health JSON)

- **File:** `python/services/health_dashboard.py`
- **Behavior:** Subscribes to `system.health`; keeps last health per backend in memory; serves `GET /api/health` (and `/api/health/dashboard`, `/api/health/{backend_id}`) for services/dashboards.
- **Deploy:** Port 8011; nginx location `/api/health-aggregated` proxies to it when using shared web server. See `docs/HEALTH_DASHBOARD.md`.

### 5. Optional next steps

- **Alpaca / TradeStation / Tastytrade:** Same pattern: after building snapshot, `publish_snapshot("alpaca", payload)` if NATS enabled.
- **TUI NATS provider:** ✅ Implemented. `python/tui/providers.py` **NatsProvider** subscribes to `snapshot.{backend}` and `system.health`; updates UI on each message (no polling). In Setup choose "NATS (subscribe)" and set `nats_url` / `nats_snapshot_backend` in config (defaults: `nats://localhost:4222`, `ib`). Requires `nats-py`.
- **Aggregator service:** Subscribes to `snapshot.ib`, `snapshot.alpaca`, …; merges; publishes `snapshot.aggregated` or serves single REST `/api/snapshot`.

---

## Environment

| Variable | Default | Purpose |
|----------|---------|---------|
| `NATS_URL` | `nats://localhost:4222` | NATS server URL. When set, IB service publishes snapshot and health after each build/check. |
| `NATS_PUBLISH_SNAPSHOT` | (unset) | Optional: set to `0` or `false` to disable snapshot publish even when NATS_URL is set. |
| `NATS_PUBLISH_HEALTH` | (unset) | Optional: set to `0` or `false` to disable health publish even when NATS_URL is set. |

Snapshot and health publish are enabled whenever `NATS_URL` is set (no extra env required).

---

## References

- `docs/NATS_USE_OPPORTUNITIES.md` — Opportunities (TUI snapshot, health, aggregator).
- `docs/NATS_TOPICS_REGISTRY.md` — Topic names and schemas.
- `docs/NATS_INTEGRATION_PYTHON.md` — Python client usage.
- `docs/WEB_SERVICES_CALL_MAP_AND_OPPORTUNITIES.md` — §3.3 shared data source (NATS).
