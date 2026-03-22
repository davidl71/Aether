# TWS API Learnings Summary

**Archived**: 2026-03-19
**Source**: `docs/research/learnings/` (15 docs)
**Era**: IBKR TWS API research phase; much of this informed the Rust `ib_adapter`

> Key patterns and best practices extracted from the TWS API research phase (2024-2025). These are preserved for the Rust `ib_adapter` team to reference when implementing IBKR connectivity.

---

## Key Patterns for ib_adapter (Rust)

### 1. Connection Flow (Critical)
Source: `TWS_API_BEST_PRACTICES.md`, `TWS_API_IMPLEMENTATION_COMPARISON.md`

The **EReader thread must start BEFORE waiting for `nextValidId`** — otherwise the connection hangs indefinitely.

```
正确的顺序:
1. eConnect()
2. 立即启动 EReader thread  ← 关键!
3. 等待 nextValidId 回调
4. 通过 EReader 处理消息
```

**Rust implication**: The `ib_adapter` must spawn the reader task before any await on connection confirmation.

### 2. Async Connection Mode
Source: `TWS_API_BEST_PRACTICES.md`

TWS API supports `asyncEConnect(true)` — non-blocking handshake. Benefits: more responsive errors, better for async applications. The C++ implementation used this; the Rust adapter should mirror this pattern.

### 3. Rate Limiting (HIGH PRIORITY — not yet implemented)
Source: `YATWS_LEARNINGS.md`

yatws (production Rust TWS lib, millions $ traded) implements:
- Max 50 msg/sec (IBKR limit is ~60)
- Max 50 simultaneous historical data requests
- Max 100 market data lines
- Stale request cleanup (removes requests > 5 min old)

**Status**: Not implemented in current `ib_adapter`. Critical for IBKR compliance.

### 4. EClient/EWrapper Architecture
Source: `ECLIENT_EWRAPPER_ARCHITECTURE.md`

IBKR TWS API uses a request-response pattern:
- **EClient** (send requests → TWS): `reqMktData()`, `placeOrder()`, `reqPositions()`
- **EWrapper** (receive callbacks ← TWS): `tickPrice()`, `orderStatus()`, `position()`

This dual-interface pattern is exactly what the Rust `ib_adapter` implements.

### 5. Error Code Awareness
Source: `TWS_API_TROUBLESHOOTING_LEARNINGS.md`, `TWS_API_BEST_PRACTICES.md`

Important IBKR error codes:
- **502**: Connection rejected (wrong port, credentials, not connected)
- **1100**: Connection lost
- **2104/2106**: Market data farm connected/disconnected
- **2110**: Market data farm connection warning

### 6. Type Safety (Strongly Typed Enums)
Source: `YATWS_LEARNINGS.md`

yatws uses enums instead of strings — compile-time safety for:
- `OrderType`: LMT, MKT, STO, etc.
- `TimeInForce`: DAY, GTC, IOC, etc.
- `SecType`: STK, OPT, FUT, etc.
- `OrderAction`: BUY, SELL

**Rust equivalent**: Use typed enums in `ib_adapter`, not raw strings.

### 7. Session Recording/Replay
Source: `YATWS_LEARNINGS.md`

yatws records all TWS interactions to SQLite for:
- Testing without live TWS
- Reproducing issues
- Backtesting with real API interactions

**Status**: Not implemented. Useful for `ib_adapter` testing.

### 8. Options Strategy Builder Pattern
Source: `YATWS_LEARNINGS.md`

yatws provides a builder for multi-leg strategies:
```rust
let (contract, order) = builder
    .bull_call_spread(expiry, strike1, strike2)
    .with_limit_price(3.50)
    .build();
```
**Box spread equivalent**: Builder pattern for 4-leg combo orders. Not yet implemented in `ib_adapter`.

### 9. Book-keeping (Auto Portfolio Tracking)
Source: `YATWS_LEARNINGS.md`

yatws automatically maintains:
- Portfolio with real-time P&L
- Order book
- Position tracking

**Status**: Partially implemented in `ib_adapter` + `ledger` crate.

### 10. Result Pattern for Error Handling
Source: `YATWS_LEARNINGS.md`

Custom error type with:
- `IBKRError` enum (Timeout, ConnectionLost, ApiError, etc.)
- Pattern matching for recovery actions

**Rust equivalent**: `thiserror` enum for `IbError`.

---

## Rust ib_adapter Gaps vs yatws Best Practices

| Feature | yatws | `ib_adapter` | Priority |
|---------|-------|--------------|----------|
| Rate limiting (msg/sec, hist data, mkt data lines) | ✅ | ❌ | **HIGH** |
| Session recording/replay (SQLite) | ✅ | ❌ | LOW |
| Options strategy builder (box spread 4-leg) | ✅ | ❌ | HIGH |
| Strong type enums | ✅ | ⚠️ Partial | MEDIUM |
| Book-keeping (auto P&L) | ✅ | ⚠️ Partial | MEDIUM |
| Result/Error enum | ✅ | ⚠️ Basic | MEDIUM |
| Async mode (`asyncEConnect`) | ✅ | ⚠️ Should verify | MEDIUM |

---

## Learnings Docs (Full List)

| File | Value | Notes |
|------|-------|-------|
| `TWS_API_BEST_PRACTICES.md` | **HIGH** | Connection flow, EReader thread, error codes |
| `YATWS_LEARNINGS.md` | **HIGH** | Rate limiting, strategy builder, type safety |
| `ECLIENT_EWRAPPER_ARCHITECTURE.md` | **HIGH** | Core TWS API architecture |
| `TWS_API_IMPLEMENTATION_COMPARISON.md` | **HIGH** | Comparison with best practices |
| `TWS_API_MARKET_DATA_LEARNINGS.md` | **MEDIUM** | Market data subscription patterns |
| `TWS_API_TROUBLESHOOTING_LEARNINGS.md` | **MEDIUM** | Error codes, common issues |
| `TWS_API_CODE_EXAMPLES_LEARNINGS.md` | **MEDIUM** | Code patterns from examples |
| `NAUTILUS_LEARNINGS.md` | **MEDIUM** | NautilusTrader architecture (Rust, prod) |
| `IB_ASYNC_LEARNINGS.md` | **LOW** | Python async patterns (informational only) |
| `TRADE_FRAME_LEARNINGS.md` | **LOW** | Trade-Frame patterns |
| `TRADE_FRAME_TWS_PATTERNS.md` | **LOW** | Trade-Frame TWS integration |
| `MULTITHREADED_TRADING_LEARNINGS.md` | **LOW** | C++ threading patterns (historical) |
| `IB_API_QUICK_REFERENCE_LEARNINGS.md` | **LOW** | Quick reference learnings |
| `IBKRBOX_LEARNINGS.md` | **LOW** | ibkrbox project (informational) |
| `IBC_LEARNINGS.md` | **LOW** | IBC automation (informational) |
| `ICLI_LEARNINGS.md` | **LOW** | icli project (informational) |
| `LEAN_LEARNINGS.md` | **LOW** | LEAN CLI (historical) |
| `YATWS_LEARNINGS.md` | **HIGH** | Already listed above |
| `TWS_API_DOCKER_LEARNINGS.md` | **LOW** | Docker patterns (may be stale) |

---

## Relevance to Current ib_adapter

**Must read before implementing**:
1. `TWS_API_BEST_PRACTICES.md` — connection flow (critical)
2. `YATWS_LEARNINGS.md` — rate limiting, strategy builder
3. `ECLIENT_EWRAPPER_ARCHITECTURE.md` — callback model

**Rust-specific notes from yatws comparison**:
- Rust async (`tokio`) maps well to TWS async patterns
- Use `Channel` for EWrapper callbacks → Rust channels
- `Arc<Mutex<>>` for shared state (connection, orders)
- Consider `SQLite` for session recording (matching engine already uses it)
