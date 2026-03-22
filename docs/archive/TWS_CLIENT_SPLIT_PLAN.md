# TWS Client Module Split Plan

**Status:** In Progress  
**Related:** `native/src/tws_client.cpp`, `native/include/tws_client.h`

## Current Structure

`tws_client.cpp` already delegates to focused handlers:

- `tws_connection.cpp` - connection lifecycle
- `tws_market_data.cpp` - market data
- `tws_orders.cpp` - order management
- `tws_positions.cpp` - position tracking
- `tws_contracts.cpp` - contract handling

The `TWSClient::Impl` class (in tws_client.cpp) holds ~800 lines including:

- NATS client setup and publishing
- EWrapper overrides that delegate to handlers
- Conversion helpers

## Completed

1. **tws_conversions.cpp** - Extracted `convert_to_tws_contract` and `calculate_dte` into `native/src/tws_conversions.cpp` and `native/include/tws_conversions.h`.

## Proposed Further Split (deferred)

1. **tws_nats_bridge.cpp** - Extract NATS publishing logic from Impl into a dedicated bridge class.
2. **tws_impl.cpp** - Move Impl class body to separate TU; keep tws_client.cpp as thin facade.

## Dependency Order

```
tws_conversions -> tws_impl -> tws_client (public API)
tws_nats_bridge -> tws_impl
```

## Completion Criteria

- [x] Conversions in tws_conversions.cpp
- [x] CMakeLists updated
- [ ] NATS logic in tws_nats_bridge.cpp (deferred)
- [ ] Impl in tws_impl.cpp (deferred)
- [ ] Tests pass (requires toolchain fix: NLopt CXX check)
