# NautilusTrader IB Agent

Replaces the C++ `TWSClient`/`TWSAdapter` as the primary IBKR connection layer.
Uses NautilusTrader's official Interactive Brokers adapter to receive market data
and execute box spread orders, then republishes events to NATS using the project's
existing `NatsEnvelope` + protobuf schema.

## Architecture

```
NT Python agent (this service)
  └── IBExecClient (clientId=10) ─┐
  └── IBDataClient (clientId=11) ─┴──→ TWS port 7497 (paper)
  └── BoxSpreadStrategy
        └── on_quote_tick → chain cache → _find_box_spreads()
        └── on_order_filled → publish to NATS
  └── NatsBridge
        └── market-data.tick.{symbol}  → MarketDataEvent
        └── strategy.decision.{symbol} → StrategyDecision
        └── orders.fill.{order_id}     → BoxSpreadExecution
        └── positions.update.{symbol}  → Position
```

The Rust backend (agents/backend/) and TUI receive events from NATS unchanged.

## Quick Start

```bash
# 1. Install deps (from repo root)
just nautilus-sync

# 2. Generate protobuf stubs (requires grpcio-tools, installed by nautilus-sync)
just proto-gen-nautilus

# 3. Edit config (set your IB paper account ID)
$EDITOR agents/nautilus/config/default.toml

# 4. Start paper trading
just nautilus-paper

# 5. Verify NATS events (in another terminal)
nats sub "market-data.>"
```

## C++ Engine in Nautilus Mode

When the NT agent owns the IB connection, rebuild the C++ engine with:

```bash
just build-nautilus-mode
```

This sets `NAUTILUS_BROKER_ENABLED` which suppresses the C++ `TWSClient` when
the `"ib"` broker is selected in config, preventing `clientId` conflicts.

## Client ID Requirements

NT internally uses **two** IBKR client IDs:

| Connection | clientId | Default |
|-----------|----------|---------|
| Exec client | `ib.client_id` | 10 |
| Data client | `ib.client_id + 1` | 11 |

These **must not** conflict with the C++ TWSClient (default clientId = 1).
When `ENABLE_NAUTILUS_BROKER=ON`, the C++ TWSClient is suppressed, removing the conflict.

## Configuration

See `config/default.toml` for all options. Key settings:

```toml
[ib]
port = 7497        # 7497=TWS paper, 4002=Gateway paper, 7496=TWS live
client_id = 10

[strategy]
symbols = ["SPX", "XSP", "NDX"]
min_arbitrage_profit = 0.10   # USD
min_roi_percent = 0.5
```

## Tests

```bash
just test-nautilus
```

Tests run without a live IB connection using mocked NT types.
Protobuf stubs must be generated first (`just proto-gen-nautilus`).
