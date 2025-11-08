# Backend Market Data Agent

## Responsibilities
- Produce normalized market data events for downstream strategy/risk services.
- Maintain reusable ingestion pipelines (`market_data` crate) and symbol routing rules.
- Expose a gRPC streaming endpoint (`ib.market_data.v1.MarketDataService/StreamEvents`).

## Layout
- `Cargo.toml` – workspace manifest referencing the shared `market_data` crate.
- `services/market_data_service/` – Tokio binary running the ingestion loop and gRPC broadcaster.
- `proto/market_data.proto` – service definition compiled via `tonic-build`.
- `scripts/` – setup/test helpers for Cursor agents and CI.

## Getting Started
1. `bash agents/backend-market-data/scripts/setup.sh`
2. `cargo run -p market_data_service` (defaults to `127.0.0.1:50061`).
3. Stream events with `grpcurl -plaintext localhost:50061 ib.market_data.v1.MarketDataService/StreamEvents`.

## Next Steps
- Replace the mock source with live IB / Nautilus feeds.
- Publish events to a message bus (Redis/NATS) for the gateway to consume.
- Wire configurable symbol subscriptions and throttling.
