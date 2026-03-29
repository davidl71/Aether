# Alpaca data flow (operator view)

This document maps **where Alpaca data enters Aether** and how it reaches the TUI. It complements [ALPACA_SOURCE_ARCHITECTURE.md](./ALPACA_SOURCE_ARCHITECTURE.md) (source vs execution boundary) and [MARKET_DATA_INTEGRATION.md](./MARKET_DATA_INTEGRATION.md).

## Credentials

- **Resolution**: `api::credentials` (`CredentialKey` for paper/live key ID and secret). Precedence: env (`APCA_*` and legacy `ALPACA_*`), then keyring (when enabled), then credential files under the user config dir.
- **TUI**: Settings → **Alpaca** — edit rows (paper/live key ID and secret), save to keyring/file, or clear with `d` / Delete. Env vars still override stored values.

## Quotes and ticks (market data)

1. Config lists aggregator sources such as `alpaca_paper` / `alpaca_live` (see market data provider docs).
2. `market_data::AlpacaSource` polls Alpaca REST for subscribed symbols.
3. Backend publishes ticks over **NATS**; the TUI consumes snapshot/stream updates like other quote sources.
4. **End-to-end (quotes)**: Alpaca API → `market_data` → NATS → TUI (charts / watchlist / health).

Alpaca is **not** used to build the SOFR/Treasury **yield curve** in the current stack. That path uses TWS / `tws_yield_curve`, NATS KV curve payloads, and `finance_rates` aggregation — see Yield below.

## Positions and account (read models)

- `AlpacaPositionSource` in `api` fetches account/positions when configured; surfaces appear in backend snapshot/TUI like other broker read models.
- Same credential split (paper vs live) as market data.

## Yield curve tab (separate path)

- **Source**: Curve data is written by the yield-curve writer / TWS daemon, held in **NATS KV**, and merged with benchmarks via HTTP/`finance_rates` — **not** via Alpaca market data APIs.
- **End-to-end (yield)**: TWS or writer → NATS KV / backend → TUI Yield tab (and manual refresh via `api.yield_curve.refresh`).

So “Alpaca → yield curve” is **not** one pipeline. Operators should expect:

- **Alpaca** → quotes/ticks (+ optional positions) → TUI.
- **Yield** → TWS/KV/finance_rates → TUI Yield tab.

## Rust-first integration note

Active Alpaca code lives under `agents/backend/`:

| Concern | Location |
|--------|----------|
| REST quotes | `crates/market_data/src/alpaca.rs` |
| Credential keys + storage | `crates/credential_store` (import as `api::credentials`) |
| Positions | `crates/api/src/alpaca_positions.rs` |
| TUI health / settings | `services/tui_service/src/alpaca_health.rs`, `ui/settings_alpaca.rs` |

Archived Python or other-language Alpaca experiments are not on the hot path; new work should extend the Rust crates above and follow the factory/registry patterns in [MARKET_DATA_PROVIDER_ARCHITECTURE.md](./MARKET_DATA_PROVIDER_ARCHITECTURE.md).
