# Archived: Integration Patterns Reference

**Archived:** 2026-03-22
**Reason:** These integration plans were from the C++/LEAN era. Patterns have been preserved as tasks.

---

## Broker Adapter Patterns (from ALPACA_INTEGRATION_PLAN_V2.md)

### Capability Flags
Each broker adapter should expose runtime capability discovery:
- `supports_box_spreads()` → bool
- `supports_options()` → bool
- `supports_combo_orders()` → bool

### Per-Broker Config
```cpp
struct AlpacaConfig {
    std::string api_key;
    std::string secret_key;
    bool paper_trading;
    std::string base_url;
    std::string data_url;
};
```

### Multi-Leg Order Composition
Box spread as 4-leg MLeg order with `position_intent`:
- `buy_to_open` / `sell_to_open`
- `buy_to_close` / `sell_to_close`

### Symbol Format (OCC)
`[Root][Expiry YYMMDD][C/P][Strike*1000 8-digit]`

---

## Position Retrieval Patterns (from IBKR_POSITION_RETRIEVAL.md)

### Async + Sync Dual API
```cpp
// Async with callback
void request_positions(PositionCallback callback);

// Sync fallback with timeout
Positions request_positions_sync(int timeout_ms);
```

### Thread-Safe Caching
- Mutex-protected position storage
- Return cached data on timeout
- Real-time push via `updatePortfolio()`

### Rate Limiting
Built-in rate limiter to avoid TWS API throttling.

### Connection Config
```cpp
struct ConnectionConfig {
    std::string host;
    int port;           // 7497=paper, 7496=live
    int client_id;
    int connection_timeout_ms;
    bool auto_reconnect;
};
```

---

## Phase-Based Migration (from CPPTRADER_INTEGRATION_POINTS.md)

1. **Foundation**: Core interface without dependencies
2. **Integration**: Wire to real implementation
3. **Validation**: Tests and performance benchmarks
4. **Enhancement**: Advanced features
5. **Alternate**: Python/other language bindings

### Performance Targets
- Tick→callback latency: <100μs
- Throughput: >100K ticks/second
- Depth levels: 100+

---

## Tasks Created From These Patterns

| Task | Description |
|------|-------------|
| T-1774187448891119000 | Enhance broker_engine with capability flags and sync fallback |
| T-1774187462319209000 | Document phase-based migration pattern |
| T-1774187465813794000 | Validate broker adapter config patterns |

---

## Original Files Archived

- `docs/archive/ALPACA_INTEGRATION_PLAN_V2.md`
- `docs/archive/CPPTRADER_INTEGRATION_POINTS.md`
- `docs/archive/IBKR_POSITION_RETRIEVAL.md`
