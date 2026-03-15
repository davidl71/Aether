# Agent E: C++ tws_client.cpp Refactor

**Obsolete:** The C++ native build and `native/` tree have been removed. IBKR/TWS integration now lives in **Rust** (`agents/backend/crates/ib_adapter/`). Use the exploration or refactor agents for Rust code; this prompt is kept for historical context only.

---

## Role (legacy)

Split `native/src/tws_client.cpp` (3,684 lines) into granular modules organized by responsibility domain, while keeping the existing `TWSClient` public API unchanged.

## Task

**Split tws_client.cpp into granular modules** (`T-1772135701785826000`)

The file contains a single `TWSClient::Impl` class (inherits `DefaultEWrapper`) that handles everything: connection, market data, orders, positions, account data, contracts, and 40+ EWrapper callbacks. Split into focused modules.

## Architecture

The `TWSClient::Impl` class uses the PImpl pattern. The Impl class inherits from `DefaultEWrapper` and overrides callbacks. The split should extract groups of related methods into helper classes or free functions that the Impl delegates to, OR split the Impl into partial classes via includes.

### Recommended approach: Extract into delegate classes

```
TWSClient::Impl (tws_client.cpp - reduced to ~500 lines)
  ├── TWSConnectionManager (tws_connection.cpp)
  ├── TWSMarketDataHandler (tws_market_data.cpp)
  ├── TWSOrderHandler (tws_orders.cpp)
  ├── TWSPositionHandler (tws_positions.cpp)
  └── TWSContractHandler (tws_contracts.cpp)
```

## Method-to-Module Mapping

### tws_connection.cpp (~300 lines)

Connection lifecycle and infrastructure:

- `connectAck()` (line 548)
- `connectionClosed()` (line 590)
- `nextValidId()` (line 661)
- `error()` (line 1200)
- `connect()` logic (lines 200-490)
- `disconnect()` logic
- Port detection and reconnection logic
- Mock client setup

### tws_market_data.cpp (~400 lines)

Market data tick callbacks and request handling:

- `tickPrice()` (line 729)
- `tickSize()` (line 810)
- `tickOptionComputation()` (line 844)
- `tickString()` (line 3058)
- `tickEFP()` (line 3060)
- `tickGeneric()` (line 3064)
- `tickSnapshotEnd()` (line 3065)
- `tickNews()` (line 3269)
- `tickReqParams()` (line 3273)
- `tickByTickAllLast()`, `tickByTickBidAsk()`, `tickByTickMidPoint()` (lines 3294-3299)
- `marketDataType()` (if present)
- Mock market data generation

### tws_orders.cpp (~300 lines)

Order placement, status, and execution:

- `orderStatus()` (line 883)
- `openOrder()` (line 926)
- `openOrderEnd()` (line 973)
- `execDetails()` (line 985)
- `execDetailsEnd()` (line 1008)
- `orderBound()` (line 3300)
- `place_order()`, `place_combo_order()` (public API)
- `cancel_order()`, `cancel_all_orders()`
- `get_order()`, `get_active_orders()`

### tws_positions.cpp (~400 lines)

Position tracking, P&L, and account data:

- `position()` (line 1022)
- `positionEnd()` (line 1062)
- `positionMulti()`, `positionMultiEnd()` (lines 3164-3167)
- `updateAccountValue()` (line 1082)
- `updateAccountTime()` (line 1119)
- `accountDownloadEnd()` (line 1123)
- `updatePortfolio()` (line 1137)
- `accountSummary()`, `accountSummaryEnd()` (lines 3154-3156)
- `accountUpdateMulti()`, `accountUpdateMultiEnd()` (lines 3168-3171)
- `pnl()`, `pnlSingle()` (lines 3286-3288)
- `request_positions()`, `get_positions()`, `get_position()`
- `request_account_updates()`, `get_account_info()`

### tws_contracts.cpp (~200 lines)

Contract details and option chain queries:

- `contractDetails()` (line 3081)
- `contractDetailsEnd()` (line 3125)
- `request_contract_details()` (public API)
- `request_option_chain()` (public API)
- Contract conversion helpers

## Previously Extracted Files (already done, do not redo)

These were extracted in a prior session and are already in the build:

- `native/src/types.cpp` + `native/include/types.h` -- struct implementations
- `native/src/connection_utils.cpp` + `native/include/connection_utils.h` -- port checking, mock client
- `native/src/tws_error_codes.cpp` + `native/include/tws_error_codes.h` -- error guidance tables

## Files You Own (exclusive)

- `native/src/tws_client.cpp` (reduce from 3,684 to ~500 lines)
- `native/src/tws_connection.cpp` (new)
- `native/src/tws_market_data.cpp` (new)
- `native/src/tws_orders.cpp` (new)
- `native/src/tws_positions.cpp` (new)
- `native/src/tws_contracts.cpp` (new)
- `native/include/tws_connection.h` (new)
- `native/include/tws_market_data.h` (new)
- `native/include/tws_orders.h` (new)
- `native/include/tws_positions.h` (new)
- `native/include/tws_contracts.h` (new)
- `native/CMakeLists.txt` (add new sources to SOURCES and HEADERS lists)
- `native/tests/CMakeLists.txt` (add new sources to test target_sources)

## Files You Must NOT Touch

- `scripts/` (owned by Agent B)
- `ansible/` (owned by Agent C)
- `Justfile` (owned by Agents A, B, D)
- `proto/` (owned by Agent D)
- `python/` (owned by Agent C)
- `native/src/types.cpp`, `native/src/connection_utils.cpp`, `native/src/tws_error_codes.cpp` (already extracted)
- `native/include/cache_client.h` (owned by Agent C)

## Code Style

- C++20, 2-space indentation, Allman braces
- `snake_case` functions/variables, `PascalCase` types
- Only comment non-obvious trading math
- Include `#pragma once` in all headers

## Completion Criteria

- [ ] `tws_client.cpp` reduced to ~500 lines (Impl class + delegation)
- [ ] 5 new `.cpp` + 5 new `.h` files created
- [ ] `native/CMakeLists.txt` updated with new sources
- [ ] `native/tests/CMakeLists.txt` updated with new sources
- [ ] All `TWSClient` public API methods still work (no signature changes)
- [ ] `ninja -C build` compiles without errors (or at minimum no new errors beyond pre-existing toolchain issues)
- [ ] Exarp task `T-1772135701785826000` marked Done
