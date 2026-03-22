# NautilusTrader Research

## Overview

**NautilusTrader** is a production-grade, Rust-native trading engine with Python API.

- **Website:** https://nautilustrader.io
- **GitHub:** https://github.com/nautechsystems/nautilus_trader
- **Stars:** 21.1k
- **License:** LGPL-3.0
- **Status:** Active (17,857 commits)

## IBKR Integration

| Status | Stable |
|--------|--------|
| Venue ID | `INTERACTIVE_BROKERS` |
| Install | `pip install nautilus_trader[ib]` |

**This is significant:** NautilusTrader has **Interactive Brokers support** - it could replace the broken C++ TWS API client entirely!

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    NautilusTrader                        │
├─────────────────────────────────────────────────────────┤
│  Python Strategy Layer (PyO3 bindings)                 │
│  - Strategy logic, config, orchestration                │
├─────────────────────────────────────────────────────────┤
│  Rust Core (event-driven, nanosecond resolution)       │
│  - Cache, MessageBus, Portfolio, Execution            │
├─────────────────────────────────────────────────────────┤
│  Adapters (modular)                                    │
│  - IBKR, Binance, Bybit, Kraken, etc.                │
└─────────────────────────────────────────────────────────┘
```

## Features

| Feature | Support |
|---------|---------|
| Multi-asset | ✅ Equities, Options, FX, Crypto, Futures |
| Backtesting | ✅ Nanosecond resolution |
| Live Trading | ✅ Same code as backtest |
| Options/Greeks | ✅ Calculate and publish to message bus |
| Box Spreads | Unknown - needs investigation |
| Combo Orders | Needs investigation |

## Integration Comparison

| Integration | Status | Our Current |
|------------|--------|-------------|
| Interactive Brokers | ✅ Stable | ❌ Broken (C++) |
| Binance | ✅ Stable | ❌ Not implemented |
| Kraken | ✅ Stable | ❌ Not implemented |

## Can It Replace Our Stack?

### Replace Completely?

| Component | Can Nautilus Replace? | Notes |
|-----------|----------------------|-------|
| C++ TWS Client | ✅ **Yes** | IBKR adapter is stable |
| Box Spread Strategy | Partial | Needs option combo order support investigation |
| Greeks Calculator | ✅ Yes | Built-in |
| Risk Checks | Partial | Has portfolio/risk concepts |
| Order Management | ✅ Yes | Full order lifecycle |
| Market Data | ✅ Yes | Via IBKR adapter |

### Advantages

1. **IBKR Integration** - Already works, stable
2. **Production-grade** - 21k stars, active dev
3. **Research-to-live parity** - Same code for backtest/live
4. **Multi-venue** - Easy to add other brokers
5. **Rust-native** - Memory safe, fast

### Concerns

1. **Box spreads** - Need to verify combo order support
2. **Custom margin** - IBKR-specific margin may need custom logic
3. **Learning curve** - New framework to learn
4. **Vendor lock-in** - Relying on third-party for IBKR connection

## Recommendation

**Adopt NautilusTrader** - Replace broken C++ TWS API with NautilusTrader's IBKR adapter.

### Migration Path

1. **Phase 1**: Deploy NautilusTrader with IBKR for live data/orders
2. **Phase 2**: Migrate box spread strategy to Python (Nautilus strategy API)
3. **Phase 3**: Deprecate C++ trading engine entirely

### Risks

- Box spread combo order support needs verification
- May need custom margin calculations
- Learning curve for team

## References

- IBKR Integration Guide: `docs/integrations/ib.md` in repo
- PyPI: `nautilus_trader`
- Installation: `pip install nautilus_trader[ib]`
