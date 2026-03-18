# Service Manager

Two entrypoints manage services:

- **`scripts/service.sh`** – Active set: `nats`, `rust`, `memcached`. Used by `just services-start`, `just services-stop`, `just svc list`, etc.
- **`scripts/service_manager.sh`** – Minimal active stack: `rust_backend`, `nats`. Used by `just services-active-start`, `just services-active-stop`.

The React/Vite web app is retired as an active runtime surface and is not managed here.

## Requirements for `just services-start`

Optional services are **skipped** if not installed (no failure):

| Service   | Required? | How to install / enable |
|-----------|-----------|---------------------------|
| **rust**  | Yes       | Built-in (Rust backend) |
| **nats**  | Optional  | `brew install nats-server` — skipped if not on PATH |
| **memcached** | Optional | `brew install memcached` — skipped if not on PATH |

For the **minimal active stack** (Rust backend + NATS only), use `just services-active-start` and install NATS: `brew install nats-server`.

## Justfile targets

| Target | Script | What runs |
|--------|--------|-----------|
| `just services-start` | start_all_services.sh → service.sh | nats, memcached, rust (optional ones skipped if not installed). Rust starts async (short wait); use `status-all` to confirm. Sets `MARKET_DATA_PROVIDER=tws`; have TWS/IB Gateway on 7497. |
| `just services-stop` | stop_all_services.sh → service.sh | Stop all in reverse order |
| `just services-active-start` | service_manager.sh | rust_backend + nats only |
| `just services-active-stop` | service_manager.sh | Stop rust_backend + nats |
| `just svc list` | service.sh list | List services and ports |
| `just svc start rust` | service.sh | Start Rust backend (via scripts/start_rust_backend.sh) |

## Usage (service_manager.sh – minimal stack)

```bash
./scripts/service_manager.sh start <service>
./scripts/service_manager.sh stop <service>
./scripts/service_manager.sh restart <service>
./scripts/service_manager.sh status [service]
./scripts/service_manager.sh start-all
./scripts/service_manager.sh stop-all
./scripts/service_manager.sh list
```

Examples:

```bash
./scripts/service_manager.sh start rust_backend
./scripts/service_manager.sh status
./scripts/service_manager.sh stop-all
```

## Usage (service.sh – full set)

```bash
./scripts/service.sh list
./scripts/service.sh start-all    # uses TWS market data (MARKET_DATA_PROVIDER=tws); ensure TWS/IB Gateway on 7497
./scripts/service.sh stop-all
./scripts/service.sh start nats
./scripts/service.sh start rust   # uses config default (mock) unless MARKET_DATA_PROVIDER is set
./scripts/service.sh status rust
```

Start-all sets `MARKET_DATA_PROVIDER=tws` so the Rust backend uses real TWS/IB Gateway data. Override with e.g. `MARKET_DATA_PROVIDER=mock ./scripts/service.sh start-all` to use mock data.

## Active Services

| Service | Port | Purpose |
|---------|------|---------|
| `rust_backend` | 8080 | Shared Rust API/backend |
| `nats` | 4222 | NATS broker |

Ports come from `config/config.json` when present and fall back to the defaults above.

**Yield curve (TUI Yield tab):** NATS must run with **JetStream** so the backend can use the KV bucket (`LIVE_STATE`) for keys like `yield_curve.SPX`. The startup scripts ensure this: `service_manager.sh` and `service.sh` (when no config) start `nats-server -js`; `start_nats.sh` uses `-js` when no config; `config/nats-server.conf` already enables JetStream. Start NATS before the backend so the yield curve writer can write and the API can read from KV.

## Logs

Each service writes to:

```text
logs/<service>_service.log
```

## Why does the Rust backend take long to start?

Startup can feel slow for three reasons:

1. **Release compile on start**  
   The manager starts the backend with `cargo run --release -p backend_service`. If the binary is not already built (first run, after `cargo clean`, or after dependency changes), Cargo does a **full release build** first. That often takes **1–5 minutes** depending on the machine. The script then waits up to **90 seconds** for the health URL to respond, so you see “waiting for port…” until the process is up.

2. **NATS connection**  
   The backend connects to NATS at startup. If NATS is not running or slow to respond, the client may block for several seconds before failing and continuing without NATS.

3. **Startup sequence**  
   Before the REST server binds, the backend loads config, connects to NATS, seeds the snapshot, and spawns market data, strategy, health, and snapshot publisher tasks. The HTTP server is started only after that; then the script’s health check can succeed.

**Faster starts:** Build once, then run the binary so “start” doesn’t compile:

```bash
cd agents/backend && cargo build --release -p backend_service
REST_SNAPSHOT_PORT=9090 ./target/release/backend_service
```

Or use the manager after a first run (binary already built); subsequent starts skip compilation and come up in a few seconds (NATS + in-process setup).

## Notes

- Use this manager for active TUI/CLI-era services only.
- Interactive Brokers public routes are now served by the Rust backend; there is no separate `ib` runtime service here.
- Israeli bank scrapers are retired from the active runtime surface for now and are not managed here.
- Risk-free-rate/benchmark routes are now Rust-owned; the remaining Python implementation is internal and is not managed as a standalone public service here.
- Discount Bank public routes are now Rust-owned and are not managed as a standalone Python service here.
- Historical browser/PWA scripts remain under `web/` as archive/reference material.
- Old one-off wrappers for retired daemons have been removed; keep service management centralized here for active runtime services only.
