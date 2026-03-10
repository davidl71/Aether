# Backend Agent

## Responsibilities
- Ingest market data and normalise events for downstream consumers.
- Perform pre-trade risk checks and expose REST (and NATS) for clients.
- Preserve an optional deprecated Nautilus scaffold for future experiments, not active production use.

## Layout
- `Cargo.toml`: Rust workspace aggregating core crates.
- `crates/`: library crates for market data, strategy bridge, risk, and API layers.
- `services/backend_service/`: Tokio binary wiring crates together.
- `python/`: deprecated Nautilus strategy scaffold kept only for future experiments.
- `config/`: runtime configuration templates (`default.toml`).
- `scripts/`: setup and CI entrypoints.

## Getting Started

### Python Environment (Required for PyO3)

The backend uses PyO3 0.24.x. For local Rust checks, point `PYO3_PYTHON` at the interpreter you want the bridge to use, or source the helper script below.

You can either:

- **Option A (recommended for full backend/Python bridge):** Use a Python 3.12 virtual environment:
  ```bash
  cd agents/backend
  source scripts/activate_python_env.sh
  ```
  See [Python Environment Setup](../../docs/PYTHON_ENVIRONMENT_SETUP.md) for details.

- **Option B (Rust-only builds):** The workspace sets `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` in `.cargo/config.toml`, and `scripts/run-tests.sh` auto-exports `PYO3_PYTHON` from `python3` when unset. Use a dedicated venv when you need deterministic backend/Python bridge behavior.

### Backend Setup

1. Run `bash agents/backend/scripts/setup.sh` to create the virtualenv, install Python deps, and fetch Rust crates.
2. Start the live service with `cargo run -p backend_service` from `agents/backend`.
3. Hit the REST surface via `curl http://127.0.0.1:8080/api/v1/snapshot`.
4. Execute checks via `bash agents/backend/scripts/run-tests.sh`.

### Nautilus Trader Wheel
Nautilus Trader is no longer part of the active backend path. The remaining backend Python package is a deprecated scaffold only. Do not treat it as a supported execution mode without explicit reactivation work.

## Current Behaviour
- Periodic mock market data updates drive the shared snapshot returned to TUI/mobile/web clients.
- Strategy signals flow through a mock internal loop, risk checks vet each decision, and the REST surface streams the approved trades plus risk status.
- REST now exposes `POST /api/v1/strategy/{start,stop}` to toggle the mock engine, updating risk status and alert stream in lockstep.
- Alerts, positions, historic fills, and orders are seeded with example data for UI prototyping.
- Polygon.io integration is available via the market data configuration; set `market_data.provider = "polygon"` and provide an API key (see below).

## Next Steps
- Swap mock data with the real ingestion pipeline.
- Flesh out POST command endpoints (`/strategy/start`, `/combos/*`) to match `agents/shared/API_CONTRACT.md`.
- Expand risk checks beyond scaffolding and persist state to QuestDB/ Livevol feeds.

## Configuring Polygon.io Market Data
Update `agents/backend/config/default.toml` (or the file pointed to by `BACKEND_CONFIG`) to enable Polygon:

```toml
[market_data]
provider = "polygon"
symbols = ["SPY", "QQQ"]
poll_interval_ms = 500

[market_data.polygon]
api_key_env = "POLYGON_API_KEY"
# Alternatively:
# api_key = "your-api-key"
```

Make sure the referenced environment variable is exported before running the backend:

```sh
export POLYGON_API_KEY="pk_your_polygon_token"
cargo run -p backend_service
```
