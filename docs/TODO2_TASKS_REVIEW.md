# Todo2 Tasks Review

**Date**: 2025-12-24
**Project**: ib_box_spread_full_universal

## Current Task Status

### Task Summary
- **Total Tasks**: 206 tasks
- **Done**: 140 tasks (68.0%)
- **Todo**: 63 tasks (30.6%)
- **In Progress**: 1 task (0.5%)
- **Other**: 2 tasks (mixed case status)

### Priority Breakdown
- **High**: 160 tasks (77.7%)
- **Medium**: 41 tasks (19.9%)
- **Critical**: 1 task (0.5%)
- **Low**: 4 tasks (1.9%)

### Active Tasks
- **In Progress**: 1 task
  - **T-209**: Diagnose MCP server communication performance issues

## Code TODO/FIXME Comments

Found **2,044 TODO/FIXME comments** in codebase (excluding third-party code).

### Key Areas with TODOs

**C++ Source Files (`native/src/`)**:

1. **`tui_provider.cpp`** - Multiple implementation TODOs:
   - Implement HTTP client using curl
   - Fetch account summary and positions
   - Implement session validation
   - Implement account listing
   - Fetch market data and options data
   - Implement OAuth 2.0 token request
   - Implement WebSocket connection using websocketpp
   - Attempt WebSocket reconnection

2. **`tws_client.cpp`** - TWS API implementation TODOs:
   - Implement option chain request using reqSecDefOptParams
   - Implement proper market hours check
   - Request and return actual TWS server time
   - Implement proper DTE calculation

3. **`ib_box_spread.cpp`** - Health/metrics TODOs:
   - Expose rate limiter metrics from TWSClient
   - Track last error
   - Track error count

4. **`risk_calculator.cpp`** - Risk calculation TODOs:
   - Get Greeks from position
   - Calculate actual correlation

5. **`box_spread_strategy.cpp`** - Strategy TODOs:
   - Implement full evaluation logic

6. **`tui_app.cpp`** - UI TODOs:
   - Implement row selection
   - Implement page scrolling
   - Implement jump to top/bottom
   - Implement detail view
   - Implement Nautilus provider

7. **`wasm_bindings.cpp`** - WASM TODOs:
   - Calculate from risk calculator
   - Calculate Greeks
   - Create strategy instance (needs refactoring)
   - Use risk calculator (needs refactoring)

**Header Files (`native/include/`)**:

1. **`box_spread_strategy.h`**:
   - TODO: Refactor to calculate_implied_interest_rate() for synthetic financing focus

2. **`tui_provider.h`**:
   - TODO: Implement actual WebSocket connection using websocketpp or similar library

**Python Files**:

1. **`strategy_runner.py`**:
   - TODO: Implement full opportunity evaluation
   - TODO: Implement full completion check
   - TODO: Convert spread to InstrumentIds and prices

## Recommendations

### Immediate Actions

1. **Review T-209**:
   - Task is "In Progress" - should be completed or moved to Review
   - Related to MCP server performance issues we just fixed

2. **Convert High-Priority Code TODOs to Todo2 Tasks**:
   - TWS API implementation TODOs (tws_client.cpp)
   - Risk calculation TODOs (risk_calculator.cpp)
   - Strategy evaluation TODOs (box_spread_strategy.cpp)

3. **Task Discovery Issue**:
   - The exarp task_discovery tool analyzed project-management-automation instead of ib_box_spread_full_universal
   - Need to ensure PROJECT_ROOT is correctly set when running discovery

### High-Priority Code TODOs to Convert

**Critical for Trading Functionality**:
1. Option chain request implementation (tws_client.cpp)
2. Market hours check (tws_client.cpp)
3. Full box spread evaluation logic (box_spread_strategy.cpp)
4. Greeks calculation (risk_calculator.cpp, wasm_bindings.cpp)

**Important for UI/UX**:
1. WebSocket connection (tui_provider.cpp, tui_provider.h)
2. HTTP client implementation (tui_provider.cpp)
3. TUI navigation (tui_app.cpp)

## Next Steps

1. ✅ Review T-209 status (currently In Progress)
2. ⏭️ Convert high-priority code TODOs to Todo2 tasks
3. ⏭️ Run task discovery with correct PROJECT_ROOT
4. ⏭️ Address testing coverage blocker (27.2% from scorecard)

---

**Note**: The exarp task_discovery tool needs to be run with `PROJECT_ROOT=/Users/davidl/Projects/Trading/ib_box_spread_full_universal` to analyze the correct project. The tool's `find_project_root()` function is finding project-management-automation instead.
