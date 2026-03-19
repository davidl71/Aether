# Comprehensive Testing Strategy

**Date**: 2025-01-16
**Version**: 1.0
**Status**: Active (updated for Rust-first; native C++ tests removed)

> **Note:** The native C++ build and Catch2 test suite were removed. Current testing is **Rust** (`agents/backend`, `cargo test`), **Python** (`agents/nautilus`, `pytest`), **TUI E2E** (`just test-tui-e2e`), and **ShellSpec** (`./scripts/run_tests.sh`). See [FUTURE_IMPROVEMENTS.md](../../planning/FUTURE_IMPROVEMENTS.md) for the list of stale test docs.

---

## Table of Contents

1. [Overview](#overview)
2. [Test Pyramid](#test-pyramid)
3. [Unit Testing](#unit-testing)
4. [Integration Testing](#integration-testing)
5. [End-to-End Testing](#end-to-end-testing)
6. [Paper Trading Validation](#paper-trading-validation)
7. [Performance Testing](#performance-testing)
8. [Security Testing](#security-testing)
9. [CI/CD Integration](#cicd-integration)
10. [Test Data Management](#test-data-management)

---

## Overview

### Current State

- **Rust unit/integration**: `cd agents/backend && cargo test`
- **Python (nautilus)**: `just test-python` or `just test-nautilus`
- **TUI E2E**: `just test-tui-e2e`
- **ShellSpec (scripts)**: `./scripts/run_tests.sh`
- **Paper Trading**: Use Rust backend/TUI with paper port (7497)
- **CI/CD**: 🟡 Basic setup exists

### Goals

1. Achieve >80% code coverage for critical paths
2. Validate all workflows end-to-end before production
3. Catch integration issues early
4. Ensure production stability through paper trading
5. Automate regression testing

### Test Framework

- **Rust**: `cargo test` (agents/backend)
- **Python**: pytest (agents/nautilus)
- **TUI E2E**: @microsoft/tui-test (tui-e2e)
- **Shell scripts**: ShellSpec
- **CI/CD**: GitHub Actions (recommended)

---

## Test Pyramid

```
                  /\
                 /  \
                /E2E \          End-to-End (5%)
               /------\         - Full workflows
              /   IT   \        - Paper trading
             /----------\       Integration (15%)
            / Unit Tests \      - Component interactions
           /--------------\
          /    80% Base    \    Unit (80%)
         /------------------\   - Individual components
```

### Distribution Target

- **Unit Tests**: 80% - Fast, isolated, comprehensive
- **Integration Tests**: 15% - Component interactions
- **End-to-End Tests**: 5% - Critical workflows

---

## Unit Testing

### Status: ✅ Rust and Python (current)

**Rust** (`agents/backend`):

- `cargo test` — all crates (api, risk, quant, ib_adapter, nats_adapter, etc.)
- Add tests in `crates/<name>/src/` or `crates/<name>/tests/`

**Python** (`agents/nautilus`):

- `just test-nautilus` or `cd agents/nautilus && uv run pytest tests/ -v`
- Strategy and NATS bridge tests in `agents/nautilus/tests/`

**Legacy (removed):** The previous C++ Catch2 suite (e.g. test_box_spread_strategy.cpp, test_order_manager.cpp, test_tws_client.cpp, test_rate_limiter.cpp, test_risk_calculator.cpp) was removed with the native build. Equivalent coverage is in Rust crates and nautilus tests
   - Validation tests

10. `test_main.cpp` (18 lines)
    - Catch2 main entry point

### Unit Test Best Practices

**Naming Convention**:

```cpp
TEST_CASE("ComponentName action expected_behavior", "[tag]") {
    // Arrange
    // Act
    // Assert
}
```

**Example**:

```cpp
TEST_CASE("BoxSpreadValidator detects invalid strike configuration", "[strategy][validation]") {
    // Arrange: Create box spread with invalid strikes
    BoxSpreadLeg spread;
    spread.long_call.strike = 510.0;  // Invalid: same as short call
    spread.short_call.strike = 510.0;

    // Act: Validate the spread
    bool is_valid = BoxSpreadValidator::validate_strikes(spread);

    // Assert: Validation should fail
    REQUIRE_FALSE(is_valid);
}
```

### Gaps to Fill (Priority: LOW)

**Missing Unit Tests**:

1. Exception handling in callbacks (after adding try-catch)
2. Contract details parsing
3. Combo order construction with real conIds
4. Edge cases in market data updates

**Estimated Effort**: 2-3 hours to add missing tests

---

## Integration Testing

### Status: 🟡 Minimal (Needs Expansion)

### Priority: ⚡ HIGH

Integration tests verify that components work together correctly.

### Required Integration Test Suites

#### 1. TWS Connection Integration Tests

**Rust**: `agents/backend` integration tests (e.g. `cargo test -p backend_service --test integration_test`). Legacy: `native/tests/test_tws_integration.cpp` removed.
**Purpose**: Verify TWS client connection lifecycle
**Estimated Lines**: 300-400

**Test Cases**:

```cpp
TEST_CASE("TWS connection lifecycle", "[integration][tws]") {
    SECTION("Initial connection succeeds") {
        // Arrange: Mock TWS on port 7497
        MockTWS mock_tws(7497);
        TWSClient client;

        // Act: Connect
        bool connected = client.connect("127.0.0.1", 7497, 0);

        // Assert
        REQUIRE(connected);
        REQUIRE(client.is_connected());
        REQUIRE(client.get_next_order_id() > 0);
    }

    SECTION("Disconnection detected and handled") {
        // Arrange: Connected client
        TWSClient client;
        client.connect("127.0.0.1", 7497, 0);

        // Act: Simulate disconnection
        mock_tws.disconnect();
        std::this_thread::sleep_for(std::chrono::seconds(1));

        // Assert: Client detects disconnection
        REQUIRE_FALSE(client.is_connected());
    }

    SECTION("Auto-reconnection works after disconnection") {
        // Arrange: Client with auto-reconnect enabled
        TWSConfig config;
        config.auto_reconnect = true;
        config.reconnect_max_attempts = 3;
        TWSClient client(config);
        client.connect("127.0.0.1", 7497, 0);

        // Act: Simulate disconnection, then TWS comes back online
        mock_tws.disconnect();
        std::this_thread::sleep_for(std::chrono::seconds(2));
        mock_tws.reconnect();
        std::this_thread::sleep_for(std::chrono::seconds(3));

        // Assert: Client reconnected
        REQUIRE(client.is_connected());
    }

    SECTION("State synchronization after reconnection") {
        // Arrange: Client with open orders and positions
        TWSClient client;
        client.connect("127.0.0.1", 7497, 0);
        // Place test order
        int order_id = client.place_order(...);

        // Act: Disconnect and reconnect
        mock_tws.disconnect();
        mock_tws.reconnect();
        client.connect("127.0.0.1", 7497, 0);

        // Assert: Orders and positions recovered
        auto order = client.get_order(order_id);
        REQUIRE(order.has_value());
    }
}
```

**Estimated Effort**: 4-6 hours

---

#### 2. Market Data Pipeline Integration Tests

**Rust**: Market data flows in `agents/backend` (ib_adapter, NATS). Legacy: `native/tests/test_market_data_integration.cpp` removed.
**Purpose**: Verify market data subscription, updates, and processing
**Estimated Lines**: 400-500

**Test Cases**:

```cpp
TEST_CASE("Market data pipeline end-to-end", "[integration][market_data]") {
    SECTION("Subscribe to option chain and receive updates") {
        // Arrange: Mock TWS with SPY option chain data
        MockTWS mock_tws;
        mock_tws.load_option_chain("test_data/spy_option_chain.json");
        TWSClient client;
        client.connect("127.0.0.1", 7497, 0);

        // Act: Request option chain
        option_chain::OptionChain chain;
        bool success = client.request_option_chain("SPY", "20250620", chain);

        // Assert: Chain populated
        REQUIRE(success);
        REQUIRE(chain.get_strike_count() > 0);
        REQUIRE(chain.has_expiry("20250620"));
    }

    SECTION("Market data updates trigger price changes") {
        // Arrange: Subscribed to SPY options
        TWSClient client;
        client.connect("127.0.0.1", 7497, 0);
        int req_id = client.request_market_data(spy_call_contract);

        // Act: Mock TWS sends price update
        mock_tws.send_tick_price(req_id, TickType::BID, 5.25);
        std::this_thread::sleep_for(std::chrono::milliseconds(100));

        // Assert: Market data updated
        auto market_data = client.get_market_data(req_id);
        REQUIRE(market_data.has_value());
        REQUIRE(market_data->bid == 5.25);
    }

    SECTION("Stale market data is detected and rejected") {
        // Arrange: Market data with old timestamp
        MockMarketData stale_data;
        stale_data.timestamp = std::chrono::system_clock::now() - std::chrono::minutes(10);

        // Act: Attempt to use stale data
        bool is_valid = market_data_validator.is_fresh(stale_data, std::chrono::seconds(60));

        // Assert: Stale data rejected
        REQUIRE_FALSE(is_valid);
    }

    SECTION("Rate limiter prevents excessive market data subscriptions") {
        // Arrange: Rate limiter enabled with max 100 lines
        RateLimiterConfig config;
        config.enabled = true;
        config.max_market_data_lines = 100;
        TWSClient client(config);
        client.connect("127.0.0.1", 7497, 0);

        // Act: Subscribe to 101 contracts
        int successful_subscriptions = 0;
        for (int i = 0; i < 101; i++) {
            if (client.request_market_data(test_contracts[i])) {
                successful_subscriptions++;
            }
        }

        // Assert: Only 100 subscriptions allowed
        REQUIRE(successful_subscriptions == 100);
    }
}
```

**Estimated Effort**: 6-8 hours

---

#### 3. Box Spread End-to-End Integration Tests

**Rust/TUI**: E2E via `just test-tui-e2e`. Legacy: `native/tests/test_box_spread_e2e.cpp` removed.
**Purpose**: Verify complete box spread workflow from discovery to execution
**Estimated Lines**: 500-600

**Test Cases**:

```cpp
TEST_CASE("Box spread workflow end-to-end", "[integration][e2e][box_spread]") {
    SECTION("Find opportunities in option chain") {
        // Arrange: Load test option chain with known arbitrage
        option_chain::OptionChain chain = load_test_chain("spy_arbitrage_chain.json");
        BoxSpreadStrategy strategy(&client, &order_mgr, params);

        // Act: Find opportunities
        auto opportunities = strategy.find_box_spreads_in_chain(chain, 500.0);

        // Assert: At least one opportunity found
        REQUIRE(opportunities.size() > 0);
        REQUIRE(opportunities[0].expected_profit > 0);
        REQUIRE(opportunities[0].is_actionable());
    }

    SECTION("Validate box spread before execution") {
        // Arrange: Valid box spread
        BoxSpreadLeg spread = create_valid_box_spread();
        std::vector<std::string> errors;

        // Act: Validate
        bool is_valid = BoxSpreadValidator::validate(spread, errors);

        // Assert: Validation passes
        REQUIRE(is_valid);
        REQUIRE(errors.empty());
    }

    SECTION("Execute box spread with individual orders (dry-run)") {
        // Arrange: Dry-run order manager
        OrderManager order_mgr(&client, /*dry_run=*/true);
        BoxSpreadLeg spread = create_valid_box_spread();

        // Act: Place box spread
        auto result = order_mgr.place_box_spread(spread, "test_strategy_1");

        // Assert: All 4 orders "placed" (dry-run)
        REQUIRE(result.success);
        REQUIRE(result.order_ids.size() == 4);
    }

    SECTION("Rollback on partial fill (simulated)") {
        // Arrange: Order manager with mock TWS
        OrderManager order_mgr(&client, /*dry_run=*/false);
        BoxSpreadLeg spread = create_valid_box_spread();

        // Mock: Leg 1 fills, Leg 2 fills, Leg 3 rejected, Leg 4 pending
        mock_tws.set_order_status_sequence({
            OrderStatus::Filled,
            OrderStatus::Filled,
            OrderStatus::Rejected,
            OrderStatus::Submitted
        });

        // Act: Place box spread
        auto result = order_mgr.place_box_spread(spread, "test_rollback");

        // Assert: Rollback triggered
        REQUIRE_FALSE(result.success);
        REQUIRE(result.error_message.find("rollback") != std::string::npos);

        // Assert: Filled orders cancelled
        REQUIRE(client.get_cancelled_order_count() == 2);  // Legs 1 & 2
    }

    SECTION("Combo order execution (when contract IDs available)") {
        // Arrange: Order manager with contract details
        OrderManager order_mgr(&client, /*dry_run=*/false);
        BoxSpreadLeg spread = create_valid_box_spread_with_conids();

        // Act: Place combo order
        auto result = order_mgr.place_box_spread(spread, "test_combo");

        // Assert: Single combo order placed
        REQUIRE(result.success);
        REQUIRE(result.order_ids.size() == 1);  // Single combo order ID
    }
}
```

**Estimated Effort**: 8-10 hours

---

#### 4. Order Manager Integration Tests (Extension)

**Rust**: Order/execution paths in `agents/backend` (api, ib_adapter). Legacy: `native/tests/test_order_manager.cpp` removed.
**Purpose**: Test order manager with real TWS client (mocked)
**Additional Lines**: +200-300

**New Test Cases**:

```cpp
TEST_CASE("Order manager with TWS client integration", "[integration][order]") {
    SECTION("Place order and receive confirmation") {
        // Arrange
        MockTWS mock_tws;
        TWSClient client;
        client.connect("127.0.0.1", 7497, 0);
        OrderManager order_mgr(&client, /*dry_run=*/false);

        // Act: Place order
        auto result = order_mgr.place_order(
            spy_call_500,
            OrderAction::Buy,
            1,
            5.25,
            TimeInForce::Day
        );

        // Simulate TWS confirmation
        mock_tws.send_order_status(result.order_ids[0], OrderStatus::Submitted);

        // Assert
        REQUIRE(result.success);
        auto status = client.get_order(result.order_ids[0]);
        REQUIRE(status.has_value());
        REQUIRE(status->status == OrderStatus::Submitted);
    }

    SECTION("Order efficiency tracking updates correctly") {
        // Arrange
        OrderManager order_mgr(&client, /*dry_run=*/false);

        // Act: Place 10 orders, fill 8
        for (int i = 0; i < 10; i++) {
            auto result = order_mgr.place_order(...);
            if (i < 8) {
                mock_tws.send_order_status(result.order_ids[0], OrderStatus::Filled);
                order_mgr.on_order_filled(result.order_ids[0]);
            }
        }

        // Assert: Efficiency ratio is 80%
        auto stats = order_mgr.get_stats();
        REQUIRE(stats.executed_trades == 8);
        REQUIRE(stats.total_orders_placed == 10);
        REQUIRE(stats.efficiency_ratio == 0.8);
    }
}
```

**Estimated Effort**: 3-4 hours

---

### Integration Test Infrastructure

**Mock TWS Server** (Required):

```cpp
class MockTWS {
public:
    MockTWS(int port = 7497);

    // Control methods
    void start();
    void stop();
    void disconnect();
    void reconnect();

    // Data injection
    void load_option_chain(const std::string& json_file);
    void send_tick_price(int req_id, TickType type, double price);
    void send_order_status(int order_id, OrderStatus status);
    void set_order_status_sequence(const std::vector<OrderStatus>& sequence);

    // Verification
    int get_request_count() const;
    int get_cancelled_order_count() const;
};
```

**Test Data Files**:

```
native/tests/data/
├── spy_option_chain.json          # Normal SPY option chain
├── spy_arbitrage_chain.json       # Chain with known arbitrage
├── spy_low_liquidity_chain.json   # Chain with wide bid/ask
└── spy_stale_data_chain.json      # Chain with old timestamps
```

**Estimated Effort to Build Infrastructure**: 6-8 hours

---

## End-to-End Testing

### Status: ❌ Missing (High Priority)

### Priority: ⚡ CRITICAL

End-to-end tests verify complete user workflows from start to finish.

### E2E Test Scenarios

#### Scenario 1: Discovery to Execution (Dry-Run)

```cpp
TEST_CASE("E2E: Discover and execute box spread (dry-run)", "[e2e]") {
    // This test runs the complete workflow:
    // 1. Connect to TWS
    // 2. Subscribe to option chain
    // 3. Find box spread opportunities
    // 4. Validate best opportunity
    // 5. Execute (dry-run)
    // 6. Verify order placement
    // 7. Disconnect cleanly

    // Arrange: Full system setup
    TWSClient client;
    OrderManager order_mgr(&client, /*dry_run=*/true);
    config::StrategyParams params = load_test_config();
    BoxSpreadStrategy strategy(&client, &order_mgr, params);

    // Act 1: Connect
    REQUIRE(client.connect("127.0.0.1", 7497, 0));

    // Act 2: Evaluate opportunities
    strategy.evaluate_symbol("SPY");

    // Assert: Strategy attempted execution (dry-run)
    auto stats = strategy.get_stats();
    REQUIRE(stats.total_opportunities_found > 0);

    // Act 3: Disconnect
    client.disconnect();
    REQUIRE_FALSE(client.is_connected());
}
```

#### Scenario 2: Reconnection Resilience

```cpp
TEST_CASE("E2E: System recovers from disconnection", "[e2e][resilience]") {
    // 1. Connect and place order
    // 2. Simulate disconnection
    // 3. Reconnect
    // 4. Verify order recovered
    // 5. Verify positions synced
}
```

#### Scenario 3: Multi-Symbol Discovery

```cpp
TEST_CASE("E2E: Multi-symbol box spread discovery", "[e2e]") {
    // 1. Connect
    // 2. Subscribe to SPY, QQQ, IWM option chains
    // 3. Find opportunities across all symbols
    // 4. Execute best opportunity
    // 5. Track positions for all symbols
}
```

**Estimated Effort**: 4-6 hours

---

## Paper Trading Validation

### Status: ❌ Not Documented (CRITICAL)

### Priority: ⚡⚡⚡ HIGHEST

Paper trading is **mandatory** before production. Test with real TWS but fake money.

### Phase 1: Basic Functionality (Day 1)

**Objectives**:

- Verify system can connect to paper trading account
- Verify option chain data is received
- Verify opportunities are found
- Verify dry-run execution works

**Test Plan**:

```bash

# 1. Configure paper trading credentials

export IB_PAPER_ACCOUNT="DU123456"
export IB_PAPER_PORT=7497

# 2. Start TWS/Gateway in paper trading mode

# 3. Run application in dry-run mode

./ib_box_spread --dry-run --symbol SPY --log-level debug

# 4. Verify logs

tail -f logs/ib_box_spread.log
```

**Success Criteria**:

- [ ] Connection established
- [ ] Option chain data received
- [ ] At least 5 opportunities found
- [ ] All validations passing
- [ ] No crashes or exceptions

**Log to**: `docs/paper_trading/day1_basic_functionality.md`

---

### Phase 2: Live Execution (Day 2)

**Objectives**:

- Execute box spread with real paper trading orders
- Verify order submission
- Monitor order fills
- Verify position tracking

**Test Plan**:

```bash

# 1. Run with live execution (paper trading)

./ib_box_spread --symbol SPY --max-positions 1 --log-level debug

# 2. Monitor TWS Trader Workstation
# - Open "Orders" panel
# - Open "Positions" panel
# - Watch for 4-leg box spread orders

# 3. Verify fills
# - Wait for orders to fill (may take time in paper trading)
# - Verify positions match expected

# 4. Close position
# - Reverse the box spread
# - Verify P&L
```

**Success Criteria**:

- [ ] All 4 orders submitted to TWS
- [ ] Orders visible in TWS interface
- [ ] Orders filled (or partially filled with rollback)
- [ ] Positions tracked correctly
- [ ] P&L calculated correctly

**Screenshots Required**:

- TWS Orders panel with 4 legs
- TWS Positions panel after fill
- Application logs showing execution

**Log to**: `docs/paper_trading/day2_live_execution.md`

---

### Phase 3: Edge Cases (Day 3)

**Objectives**:

- Test partial fill scenario
- Test low liquidity filtering
- Test stale data rejection
- Test rate limiting

**Test Scenarios**:

**3.1: Partial Fill with Rollback**

```bash

# Manually cancel one leg after submission
# 1. Run application
# 2. When box spread placed, go to TWS
# 3. Manually cancel leg #3
# 4. Verify rollback triggered
# 5. Verify other legs cancelled
```

**3.2: Low Liquidity Filtering**

```bash

# Use symbol with wide bid/ask spreads

./ib_box_spread --symbol XYZ --max-bid-ask-spread 0.10 --log-level debug

# Verify: Opportunities filtered out
```

**3.3: Rate Limiting**

```bash

# Burst 100 requests rapidly
# Enable rate limiter

./ib_box_spread --enable-rate-limiter --symbols SPY,QQQ,IWM,...[50 symbols]

# Verify: Rate limiter warnings logged
# Verify: No TWS errors about pacing violations
```

**Success Criteria**:

- [ ] Rollback works on partial fill
- [ ] Low liquidity opportunities filtered
- [ ] Rate limiter prevents TWS errors
- [ ] No system crashes

**Log to**: `docs/paper_trading/day3_edge_cases.md`

---

### Phase 4: Reconnection Resilience (Day 4)

**Objectives**:

- Test disconnection handling
- Test reconnection with state sync
- Verify order recovery

**Test Plan**:

```bash

# 1. Start application

./ib_box_spread --symbol SPY --auto-reconnect

# 2. Place box spread

# 3. While orders pending, disconnect TWS
# (Close TWS application)

# 4. Verify disconnection detected
# (Check logs for error 1100)

# 5. Restart TWS

# 6. Verify auto-reconnection
# (Check logs for reconnection)

# 7. Verify orders recovered
# (Check TWS orders panel)
```

**Success Criteria**:

- [ ] Disconnection detected immediately
- [ ] Auto-reconnect triggered
- [ ] Reconnection successful
- [ ] Open orders recovered
- [ ] Positions synced

**Log to**: `docs/paper_trading/day4_reconnection.md`

---

### Phase 5: Extended Run (Day 5-7)

**Objectives**:

- Run for 8 hours continuously
- Monitor for memory leaks
- Monitor for crashes
- Monitor efficiency ratio

**Test Plan**:

```bash

# 1. Start application for extended run

nohup ./ib_box_spread \
    --symbols SPY,QQQ,IWM \
    --max-positions 5 \
    --log-level info \
    > extended_run.log 2>&1 &

# 2. Monitor every hour

watch -n 3600 'tail -100 extended_run.log'

# 3. Monitor memory usage

watch -n 300 'ps aux | grep ib_box_spread'

# 4. After 8 hours, analyze results

./scripts/analyze_paper_trading_run.sh extended_run.log
```

**Metrics to Track**:

- Total opportunities found
- Total orders placed
- Total orders filled
- Order efficiency ratio
- Total P&L (paper)
- Memory usage (RSS)
- CPU usage (avg)
- Exception count
- Reconnection count

**Success Criteria**:

- [ ] No crashes for 8+ hours
- [ ] Memory usage stable (no leaks)
- [ ] Efficiency ratio > 5%
- [ ] No unhandled exceptions
- [ ] At least 10 opportunities evaluated

**Log to**: `docs/paper_trading/day5_extended_run.md`

---

### Paper Trading Checklist

**Before Starting**:

- [ ] TWS/Gateway installed
- [ ] Paper trading account created
- [ ] Application compiled in Release mode
- [ ] Configuration file validated
- [ ] Test plan reviewed

**During Testing**:

- [ ] Take screenshots of TWS interface
- [ ] Log all unexpected behavior
- [ ] Record all error messages
- [ ] Monitor system resources
- [ ] Document workarounds

**After Testing**:

- [ ] Review all logs
- [ ] Calculate success metrics
- [ ] Document issues found
- [ ] Update code if needed
- [ ] Re-test fixes

---

## Performance Testing

### Status: ❌ Missing (Medium Priority)

### Objectives

- Measure latency from market data to execution
- Measure throughput (opportunities/second)
- Test under load (1000+ option contracts)
- Identify bottlenecks

### Performance Test Cases

#### 1. Market Data Processing Latency

```cpp
TEST_CASE("Performance: Market data processing latency", "[performance]") {
    // Measure time from tick received to opportunity evaluated

    auto start = std::chrono::high_resolution_clock::now();

    // Receive market data tick
    client.on_tick_price(req_id, BID, 5.25);

    // Evaluate opportunity
    strategy.evaluate_opportunities();

    auto end = std::chrono::high_resolution_clock::now();
    auto latency = std::chrono::duration_cast<std::chrono::microseconds>(end - start);

    // Assert: Latency < 1ms for single update
    REQUIRE(latency.count() < 1000);
}
```

#### 2. Option Chain Scanning Performance

```cpp
TEST_CASE("Performance: Option chain scanning with 1000 strikes", "[performance]") {
    // Load large option chain (500 strikes x 2 types = 1000 options)
    option_chain::OptionChain chain = load_large_chain("spy_1000_options.json");

    auto start = std::chrono::high_resolution_clock::now();

    // Scan for opportunities
    auto opportunities = strategy.find_box_spreads_in_chain(chain, 500.0);

    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);

    // Assert: Scan completes in < 100ms for 1000 options
    REQUIRE(duration.count() < 100);

    // Log: Performance metrics
    spdlog::info("Scanned {} options in {}ms ({} opp/sec)",
                 1000, duration.count(),
                 opportunities.size() * 1000.0 / duration.count());
}
```

#### 3. Memory Usage Under Load

```cpp
TEST_CASE("Performance: Memory usage with 100 active subscriptions", "[performance]") {
    // Measure memory before
    size_t mem_before = get_current_rss();

    // Subscribe to 100 options
    std::vector<int> req_ids;
    for (int i = 0; i < 100; i++) {
        req_ids.push_back(client.request_market_data(test_contracts[i]));
    }

    // Measure memory after
    size_t mem_after = get_current_rss();
    size_t mem_increase = mem_after - mem_before;

    // Assert: Memory increase < 10MB for 100 subscriptions
    REQUIRE(mem_increase < 10 * 1024 * 1024);
}
```

**Estimated Effort**: 4-6 hours

---

## Security Testing

### Status: 🟡 Partial (Basic validation exists)

### Priority: MEDIUM

### Security Checklist

**Input Validation**:

- [ ] Symbol validation (alphanumeric only, max length)
- [ ] Strike validation (positive, reasonable range)
- [ ] Quantity validation (positive, max size limits)
- [ ] Price validation (positive, reasonable range)
- [ ] Date validation (YYYYMMDD format, future date)

**Credential Management**:

- [ ] API credentials not hardcoded
- [ ] Configuration files not in git
- [ ] Secure storage for paper/live account IDs
- [ ] Environment variables for sensitive data

**Network Security**:

- [ ] TWS connection uses localhost only (no remote)
- [ ] No listening ports (unless intentional)
- [ ] TLS/SSL for any external APIs (if used)

**Error Handling**:

- [ ] No sensitive data in error messages
- [ ] No stack traces exposed to users
- [ ] All exceptions caught at top level

**Testing**:

```cpp
TEST_CASE("Security: Invalid symbol rejected", "[security]") {
    // Test SQL injection attempt
    std::string malicious_symbol = "'; DROP TABLE orders; --";

    std::string error;
    bool valid = OrderValidator::validate_symbol(malicious_symbol, error);

    REQUIRE_FALSE(valid);
    REQUIRE(error.find("Invalid symbol") != std::string::npos);
}
```

**Estimated Effort**: 2-3 hours

---

## CI/CD Integration

### Status: 🟡 Basic Setup Exists

### Priority: HIGH

### GitHub Actions Workflow

**File**: `.github/workflows/ci.yml`

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build-and-test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        build_type: [Debug, Release]

    steps:
    - uses: actions/checkout@v3
      with:
        submodules: recursive

    - name: Install Dependencies
      run: |
        if [ "$RUNNER_OS" == "Linux" ]; then
          sudo apt-get update
          sudo apt-get install -y cmake ninja-build
        elif [ "$RUNNER_OS" == "macOS" ]; then
          brew install cmake ninja
        fi

    - name: Configure CMake
      run: |
        cmake -B build -G Ninja \
          -DCMAKE_BUILD_TYPE=${{ matrix.build_type }} \
          -DBUILD_TESTING=ON

    - name: Build
      run: cmake --build build --config ${{ matrix.build_type }}

    - name: Run Unit Tests
      run: ctest --test-dir build --output-on-failure

    - name: Run Integration Tests
      if: matrix.build_type == 'Release'
      run: ctest --test-dir build --output-on-failure -L integration

    - name: Generate Coverage Report
      if: matrix.build_type == 'Debug' && runner.os == 'Linux'
      run: |
        # Install gcov/lcov
        # Generate coverage
        # Upload to codecov.io

    - name: Upload Artifacts
      uses: actions/upload-artifact@v3
      with:
        name: binaries-${{ matrix.os }}-${{ matrix.build_type }}
        path: build/ib_box_spread
```

### Test Execution in CI

**Tag Tests by Speed**:

```cpp
TEST_CASE("Fast unit test", "[unit][fast]") { /* ... */ }
TEST_CASE("Slow integration test", "[integration][slow]") { /* ... */ }
```

**Run Fast Tests on Every Commit**:

```bash
ctest --test-dir build -L "unit,fast" --output-on-failure
```

**Run All Tests on PR**:

```bash
ctest --test-dir build --output-on-failure
```

**Estimated Effort**: 2-3 hours

---

## Test Data Management

### Test Data Repository

**Structure**:

```
native/tests/data/
├── option_chains/
│   ├── spy_20250620_normal.json       # Typical SPY chain
│   ├── spy_20250620_arbitrage.json    # Known arbitrage opportunity
│   ├── spy_20250620_low_liq.json      # Wide bid/ask spreads
│   └── spy_20250620_stale.json        # Old timestamps
├── market_data/
│   ├── spy_500c_ticks.json            # Tick data for SPY 500 call
│   └── qqq_300p_ticks.json            # Tick data for QQQ 300 put
├── orders/
│   ├── valid_box_spread.json          # Valid 4-leg box spread
│   └── invalid_box_spread.json        # Invalid configuration
└── configs/
    ├── conservative.json              # Conservative strategy params
    ├── aggressive.json                # Aggressive strategy params
    └── test.json                      # Test/development params
```

### Test Data Generators

```cpp
// Generate realistic option chain for testing
option_chain::OptionChain generate_test_chain(
    const std::string& symbol,
    const std::string& expiry,
    double underlying_price,
    int num_strikes = 50,
    double strike_interval = 5.0);

// Generate box spread with known arbitrage
types::BoxSpreadLeg generate_arbitrage_opportunity(
    double strike_low,
    double strike_high,
    double arbitrage_amount);
```

**Estimated Effort**: 3-4 hours

---

## Test Metrics & Reporting

### Code Coverage Target

**Overall**: 80%+ code coverage
**Critical Components**: 90%+ coverage

- BoxSpreadStrategy
- OrderManager
- TWSClient (connection/reconnection)
- BoxSpreadValidator

### Coverage Tools

**Linux**: gcov + lcov

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Debug -DENABLE_COVERAGE=ON
make -C build
ctest --test-dir build
lcov --capture --directory build --output-file coverage.info
genhtml coverage.info --output-directory coverage_html
```

**macOS**: Xcode Instruments or llvm-cov

### Test Reporting

**JUnit XML Output** (for CI):

```bash
ctest --test-dir build --output-junit test_results.xml
```

**HTML Report** (for developers):

```bash
ctest --test-dir build --output-on-failure > test_report.txt
./scripts/generate_test_report.sh test_report.txt > test_report.html
```

---

## Testing Timeline

### Week 1

**Mon-Tue**: Integration test infrastructure (Mock TWS, test data)
**Wed-Thu**: Write integration tests (TWS, market data, box spread)
**Fri**: Write E2E tests

### Week 2

**Mon**: Paper trading Day 1 (basic functionality)
**Tue**: Paper trading Day 2 (live execution)
**Wed**: Paper trading Day 3 (edge cases)
**Thu**: Paper trading Day 4 (reconnection)
**Fri**: Paper trading Day 5 (extended run setup)

### Week 3

**Mon-Wed**: Extended paper trading run (3 days continuous)
**Thu**: Analyze results, fix issues
**Fri**: Re-test fixes, prepare production deployment

**Total Estimated Effort**: 80-100 hours over 3 weeks

---

## Success Criteria (Overall)

### Before Production Deployment

**Unit Tests**:

- [x] 80%+ coverage (already achieved)
- [ ] All tests passing
- [ ] No flaky tests

**Integration Tests**:

- [ ] All component integrations tested
- [ ] Mock TWS infrastructure complete
- [ ] 100% pass rate

**End-to-End Tests**:

- [ ] Critical workflows tested
- [ ] Dry-run E2E tests passing
- [ ] Reconnection scenarios validated

**Paper Trading**:

- [ ] 1 week of stable operation
- [ ] No crashes or unhandled exceptions
- [ ] Order efficiency > 5%
- [ ] Atomic execution verified (combo or rollback)
- [ ] P&L reconciles with TWS

**CI/CD**:

- [ ] Automated tests on every commit
- [ ] Coverage reports generated
- [ ] Release artifacts built automatically

---

## Related Documentation

- **TWS_API_BEST_PRACTICES.md** - TWS integration patterns

---

## Appendix: Test Commands Reference

**Run All Tests**:

```bash
ctest --test-dir build --output-on-failure
```

**Run Specific Test**:

```bash
ctest --test-dir build -R test_box_spread_strategy --output-on-failure
```

**Run Tests by Tag**:

```bash
cd agents/backend && cargo test
cd agents/backend && cargo test --test integration_test
just test-tui-e2e
```

**Run with Verbose Output**:

```bash
ctest --test-dir build --output-on-failure --verbose
```

**Generate Coverage**:

```bash
cmake -B build -DCMAKE_BUILD_TYPE=Debug -DENABLE_COVERAGE=ON
cmake --build build
ctest --test-dir build
lcov --capture --directory build --output-file coverage.info
genhtml coverage.info --output-directory coverage_html
open coverage_html/index.html
```

---

**End of Testing Strategy**
