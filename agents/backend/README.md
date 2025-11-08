# Backend Agent

## Responsibilities
- Ingest market data and normalise events for downstream consumers.
- Execute Nautilus Trader strategies through a Rust/Python bridge.
- Perform pre-trade risk checks and expose REST/gRPC surfaces for clients.

## Layout
- `Cargo.toml`: Rust workspace aggregating core crates.
- `crates/`: library crates for market data, strategy bridge, risk, and API layers.
- `services/backend_service/`: Tokio binary wiring crates together.
- `python/`: Nautilus Trader strategy scaffold packaged for reuse.
- `proto/`: gRPC service definitions.
- `config/`: runtime configuration templates (`default.toml`).
- `scripts/`: setup and CI entrypoints.

## Getting Started
1. Run `bash agents/backend/scripts/setup.sh` to create the virtualenv, install Python deps, and fetch Rust crates.
2. Start the live service with `cargo run -p backend_service` from `agents/backend`.
3. Hit the REST surface via `curl http://127.0.0.1:8080/api/v1/snapshot`.
4. Stream strategy decisions with a gRPC client against `ib.backend.v1.StrategyService/StreamDecisions` on port `50051`.
5. Execute checks via `bash agents/backend/scripts/run-tests.sh`.

## Current Behaviour
- Periodic mock market data updates drive the shared snapshot returned to TUI/mobile/web clients.
- Strategy signals flow through a mock Nautilus loop, risk checks vet each decision, and both REST/gRPC surfaces stream the approved trades plus risk status.
- REST now exposes `POST /api/v1/strategy/{start,stop}` to toggle the mock engine, updating risk status and alert stream in lockstep.
- Alerts, positions, historic fills, and orders are seeded with example data for UI prototyping.

## Next Steps
- Swap mock data with the real ingestion pipeline and Nautilus-driven strategy execution.
- Flesh out POST command endpoints (`/strategy/start`, `/combos/*`) to match `agents/shared/API_CONTRACT.md`.
- Expand risk checks beyond scaffolding and persist state to QuestDB/ Livevol feeds.
