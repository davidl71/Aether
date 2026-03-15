# Breadcrumb Logging for Trading Operations Testing

**Purpose**: Guide for using breadcrumb-style logging to test and debug trading operations, similar to OpenAlgo's API Analyzer.

**Status:** The code examples below refer to **removed native C++** modules (`native/src/order_manager.cpp`, `native/src/tws_client.cpp`, etc.). Apply the same patterns in the **Rust backend** (`agents/backend`): add structured logging (e.g. tracing, JSON events) around order placement, TWS connection, and strategy actions in `api`, `ib_adapter`, and related crates.

**Reference**:

- [OpenAlgo API Analyzer](https://github.com/marketcalls/openalgo) - Comprehensive testing and validation tool
- [TUI Breadcrumb Logging Guide](./TUI_BREADCRUMB_LOGGING.md) - General breadcrumb logging documentation

---

## Overview

Breadcrumb logging provides a complete audit trail of trading operations, enabling:

- **Risk-Free Testing**: Test all trading operations without actual execution
- **Real-Time Validation**: Instant feedback on order parameters and strategy logic
- **Complete Debugging**: Full trail of events leading to issues
- **Performance Analysis**: Timing between operations
- **Compliance**: Regulatory audit trail requirements

This is similar to OpenAlgo's API Analyzer, but integrated directly into the trading application.

---

## Integration Points

### 1. Order Manager (concept; implement in Rust)

**Legacy location (removed):** `native/src/order_manager.cpp`. Implement equivalent in **Rust** (`agents/backend`, e.g. order/execution paths in `api` or `ib_adapter`) with structured logging.

**Breadcrumb-style points** (conceptual; adapt to Rust tracing/logging):

```cpp
// Order placement attempt
ExecutionResult OrderManager::place_order(...) {
  // Log order attempt
  TUI_BREADCRUMB_ACTION("place_order_attempt", "order_manager",
                        "symbol=" + contract.symbol +
                        " side=" + types::order_action_to_string(action) +
                        " qty=" + std::to_string(quantity) +
                        " price=" + std::to_string(limit_price));

  // Log validation
  if (!validate_order(contract, action, quantity, limit_price, error_msg)) {
    TUI_BREADCRUMB_ERROR("order_manager", "validation_failed",
                          "error=" + error_msg +
                          " symbol=" + contract.symbol);
    result.success = false;
    result.error_message = error_msg;
    return result;
  }

  // Log state before execution
  nlohmann::json state;
  state["dry_run"] = pimpl_->dry_run_;
  state["total_orders_placed"] = pimpl_->stats_.total_orders_placed;
  state["executed_trades"] = pimpl_->stats_.executed_trades;
  TUI_BREADCRUMB_STATE_CHANGE("order_manager", "before_order_execution",
                              state.dump());

  if (pimpl_->dry_run_) {
    TUI_BREADCRUMB_ACTION("order_placed_dry_run", "order_manager",
                          "order_id=DRY-" + std::to_string(999));
    // ... dry run logic
  } else {
    // Place order through TWS
    int order_id = pimpl_->client_->place_order(...);

    TUI_BREADCRUMB_ACTION("order_placed", "order_manager",
                          "order_id=" + std::to_string(order_id) +
                          " symbol=" + contract.symbol);
  }

  return result;
}
```

**Box Spread Logging**:

```cpp
ExecutionResult OrderManager::place_box_spread(...) {
  // Log box spread attempt
  TUI_BREADCRUMB_ACTION("place_box_spread_attempt", "order_manager",
                        "symbol=" + spread.long_call.symbol +
                        " strikes=" + std::to_string(spread.long_call.strike) +
                        "/" + std::to_string(spread.short_call.strike) +
                        " expiry=" + spread.long_call.expiry);

  // Log 4-leg order details
  nlohmann::json legs;
  legs["long_call"] = spread.long_call.to_string();
  legs["short_call"] = spread.short_call.to_string();
  legs["long_put"] = spread.long_put.to_string();
  legs["short_put"] = spread.short_put.to_string();
  TUI_BREADCRUMB_STATE_CHANGE("order_manager", "box_spread_legs",
                              legs.dump());

  // ... execution logic

  return result;
}
```

### 2. TWS / IB Client (concept; implement in Rust)

**Legacy location (removed):** `native/src/tws_client.cpp`. Implement equivalent in **Rust** (`agents/backend/crates/ib_adapter`) with structured logging.

**Breadcrumb-style points** (conceptual):

```cpp
// Connection events
void TWSClient::Impl::connectAck() override {
  TUI_BREADCRUMB_ACTION("tws_connected", "tws_client",
                        "host=" + config_.host +
                        " port=" + std::to_string(config_.port));
  // ... connection logic
}

void TWSClient::Impl::connectionClosed() override {
  TUI_BREADCRUMB_ERROR("tws_client", "connection_closed",
                        "auto_reconnect=" +
                        std::to_string(config_.auto_reconnect));
  // ... disconnection logic
}

// Order callbacks
void TWSClient::Impl::orderStatus(...) override {
  TUI_BREADCRUMB_STATE_CHANGE("tws_client", "order_status_update",
                              "order_id=" + std::to_string(orderId) +
                              " status=" + status +
                              " filled=" + std::to_string(filled) +
                              " remaining=" + std::to_string(remaining));
  // ... status update logic
}

// Error callbacks
void TWSClient::Impl::error(...) override {
  TUI_BREADCRUMB_ERROR("tws_client", "api_error",
                        "error_code=" + std::to_string(errorCode) +
                        " error=" + errorString +
                        " id=" + std::to_string(id));
  // ... error handling
}
```

### 3. Strategy Execution

**Legacy (removed):** `native/src/box_spread_strategy.cpp`. Implement in **Rust** (e.g. strategy/order logic in `agents/backend`).

**Breadcrumb Logging Points**:

```cpp
// Strategy decision
void BoxSpreadStrategy::execute_opportunity(...) {
  // Log opportunity detection
  TUI_BREADCRUMB_ACTION("opportunity_detected", "box_spread_strategy",
                        "symbol=" + opportunity.symbol +
                        " profit=" + std::to_string(opportunity.profit) +
                        " roi=" + std::to_string(opportunity.roi));

  // Log risk check
  if (!risk_check_passed) {
    TUI_BREADCRUMB_ERROR("box_spread_strategy", "risk_check_failed",
                          "reason=" + risk_reason);
    return;
  }

  // Log order placement
  auto result = order_manager_->place_box_spread(...);

  if (result.success) {
    TUI_BREADCRUMB_ACTION("strategy_executed", "box_spread_strategy",
                          "order_ids=" + join_order_ids(result.order_ids));
  } else {
    TUI_BREADCRUMB_ERROR("box_spread_strategy", "execution_failed",
                          "error=" + result.error_message);
  }
}
```

### 4. Rate Limiter

**Legacy (removed):** `native/src/rate_limiter.cpp`. Implement in **Rust** where rate limiting is used (e.g. `agents/backend`).

**Breadcrumb Logging Points**:

```cpp
bool RateLimiter::check_message_rate() {
  if (!enabled_.load()) {
    return true;
  }

  std::lock_guard<std::mutex> lock(mutex_);
  cleanup_old_message_timestamps();

  int messages_in_last_second = count_messages_in_last_second();

  if (messages_in_last_second >= config_.max_messages_per_second) {
    TUI_BREADCRUMB_ERROR("rate_limiter", "rate_limit_exceeded",
                          "messages=" + std::to_string(messages_in_last_second) +
                          " limit=" + std::to_string(config_.max_messages_per_second));
    return false;
  }

  return true;
}
```

---

## Testing Scenarios

### Scenario 1: Order Placement Test

**Objective**: Verify order placement with full parameter validation

**Breadcrumb Trail**:

```
1. place_order_attempt (order_manager) - symbol=SPX, side=BUY, qty=10, price=5090.50
2. state_change (order_manager) - before_order_execution (dry_run=true, total_orders=5)
3. order_placed_dry_run (order_manager) - order_id=DRY-999
```

**Assertions**:

- Order attempt logged with correct parameters
- State change captured before execution
- Dry-run mode properly logged
- Order ID assigned

### Scenario 2: Validation Failure Test

**Objective**: Verify validation errors are properly logged

**Breadcrumb Trail**:

```
1. place_order_attempt (order_manager) - symbol=SPX, side=BUY, qty=-10, price=5090.50
2. error (order_manager) - validation_failed, error=Invalid quantity: must be positive
```

**Assertions**:

- Validation failure logged with error message
- No order placed (no order_placed breadcrumb)
- Error details captured

### Scenario 3: Box Spread Execution Test

**Objective**: Verify box spread placement with all 4 legs

**Breadcrumb Trail**:

```
1. place_box_spread_attempt (order_manager) - symbol=SPX, strikes=5000/5010, expiry=2025-02-21
2. state_change (order_manager) - box_spread_legs (4 legs with details)
3. order_placed (order_manager) - order_id=1001,1002,1003,1004
4. order_status_update (tws_client) - order_id=1001, status=Submitted
5. order_status_update (tws_client) - order_id=1001, status=Filled
```

**Assertions**:

- All 4 legs logged
- Order IDs assigned
- Status updates tracked
- Fill confirmed

### Scenario 4: Rate Limit Test

**Objective**: Verify rate limiting prevents excessive requests

**Breadcrumb Trail**:

```
1. place_order_attempt (order_manager) - ... (request 1-49)
2. place_order_attempt (order_manager) - ... (request 50)
3. error (rate_limiter) - rate_limit_exceeded, messages=50, limit=50
4. error (order_manager) - rate_limit_failed, error=Rate limit exceeded
```

**Assertions**:

- Rate limit detected
- Request rejected
- Error logged with context

### Scenario 5: Connection Failure Test

**Objective**: Verify connection failures are logged with context

**Breadcrumb Trail**:

```
1. tws_connection_attempt (tws_client) - host=127.0.0.1, port=7497
2. error (tws_client) - connection_failed, error=Connection refused
3. tws_reconnect_attempt (tws_client) - attempt=1, delay=3000ms
4. tws_connected (tws_client) - host=127.0.0.1, port=7497
```

**Assertions**:

- Connection attempts logged
- Failures captured with error details
- Reconnection attempts tracked
- Success logged

---

## Performance Analysis

### Timing Breadcrumbs

Add timing information to breadcrumbs:

```cpp
auto start = std::chrono::steady_clock::now();
// ... operation ...
auto end = std::chrono::steady_clock::now();
auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);

TUI_BREADCRUMB_ACTION("order_placed", "order_manager",
                      "order_id=" + std::to_string(order_id) +
                      " duration_ms=" + std::to_string(duration.count()));
```

### Performance Metrics

Extract performance metrics from breadcrumbs:

```python

# Analyze breadcrumb log for performance

def analyze_performance(breadcrumbs):
    order_times = []
    for entry in breadcrumbs:
        if entry.action == "order_placed":
            # Extract duration from details
            duration = extract_duration(entry.details)
            order_times.append(duration)

    return {
        "avg_order_time_ms": sum(order_times) / len(order_times),
        "min_order_time_ms": min(order_times),
        "max_order_time_ms": max(order_times)
    }
```

---

## Compliance and Audit Trail

### Regulatory Requirements

Breadcrumb logs provide:

- **Complete Audit Trail**: Every trading operation logged
- **Timestamp Accuracy**: Millisecond precision timestamps
- **State Snapshots**: Full state at critical points
- **Error Context**: Complete error information

### Log Retention

```cpp
// Configure breadcrumb logging for compliance
BreadcrumbLogger::Config config;
config.enabled = true;
config.log_file = "/var/log/trading/breadcrumbs.log";
config.max_entries = 100000;  // Large retention
config.capture_state_dumps = true;  // Full state capture
config.capture_screen_dumps = false;  // Not needed for compliance
```

### Log Analysis

```python

# Extract compliance-relevant breadcrumbs

def extract_compliance_logs(breadcrumbs):
    compliance_events = []
    for entry in breadcrumbs:
        if entry.type in [BreadcrumbType::Action, BreadcrumbType::Error]:
            if "order" in entry.action or "trade" in entry.action:
                compliance_events.append({
                    "timestamp": entry.timestamp,
                    "action": entry.action,
                    "details": entry.details,
                    "state": entry.state_snapshot
                })
    return compliance_events
```

---

## Integration with Testing Frameworks

### Catch2 Integration

```cpp
TEST_CASE("Order placement breadcrumb logging") {
  // Initialize breadcrumb logging
  tui::BreadcrumbLogger::Config config;
  config.enabled = true;
  config.log_file = "/tmp/test_breadcrumbs.log";
  tui::InitializeBreadcrumbLogging(config);

  // Clear previous breadcrumbs
  tui::GetBreadcrumbLogger().Clear();

  // Execute test
  auto result = order_manager.place_order(...);

  // Verify breadcrumbs
  auto breadcrumbs = tui::GetBreadcrumbLogger().GetBreadcrumbs();

  REQUIRE(breadcrumbs.size() >= 2);
  REQUIRE(breadcrumbs[0].action == "place_order_attempt");
  REQUIRE(breadcrumbs[1].action == "order_placed");
}
```

### Python Integration Tests

```python
def test_order_placement_breadcrumbs():
    # Initialize breadcrumb logging
    config = BreadcrumbLoggerConfig(
        enabled=True,
        log_file="/tmp/test_breadcrumbs.log"
    )
    initialize_breadcrumb_logging(config)

    # Execute test
    result = order_manager.place_order(...)

    # Verify breadcrumbs
    breadcrumbs = get_breadcrumb_logger().get_breadcrumbs()

    assert len(breadcrumbs) >= 2
    assert breadcrumbs[0].action == "place_order_attempt"
    assert breadcrumbs[1].action == "order_placed"
```

---

## Best Practices

### 1. Log Before State Changes

Always log before modifying state:

```cpp
// Good
TUI_BREADCRUMB_STATE_CHANGE("order_manager", "before_cancel",
                            DumpOrderState());
cancel_order(order_id);
TUI_BREADCRUMB_STATE_CHANGE("order_manager", "after_cancel",
                            DumpOrderState());

// Bad
cancel_order(order_id);
TUI_BREADCRUMB_ACTION("order_cancelled", "order_manager",
                      "order_id=" + std::to_string(order_id));
```

### 2. Include Context in Details

Provide enough context to understand the operation:

```cpp
// Good
TUI_BREADCRUMB_ACTION("order_placed", "order_manager",
                      "order_id=" + std::to_string(order_id) +
                      " symbol=" + contract.symbol +
                      " side=" + action_to_string(action) +
                      " qty=" + std::to_string(quantity) +
                      " price=" + std::to_string(limit_price) +
                      " dry_run=" + std::to_string(dry_run));

// Bad
TUI_BREADCRUMB_ACTION("order_placed", "order_manager",
                      "order_id=" + std::to_string(order_id));
```

### 3. Log Errors with Full Context

Always include state when logging errors:

```cpp
try {
  // ... operation ...
} catch (const std::exception& e) {
  std::string state = DumpCurrentState();
  TUI_BREADCRUMB_ERROR("order_manager", "execution_failed",
                        "error=" + std::string(e.what()), state);
  throw;
}
```

### 4. Use Consistent Action Names

Use consistent naming for similar operations:

- `place_order_attempt` / `place_order_success` / `place_order_failed`
- `cancel_order_attempt` / `cancel_order_success` / `cancel_order_failed`
- `box_spread_attempt` / `box_spread_success` / `box_spread_failed`

---

## Comparison with OpenAlgo API Analyzer

| Feature | OpenAlgo API Analyzer | Breadcrumb Logging |
|---------|----------------------|-------------------|
| **Testing** | ✅ Risk-free testing | ✅ Dry-run mode |
| **Validation** | ✅ Parameter validation | ✅ Pre-execution validation |
| **Monitoring** | ✅ Real-time monitoring | ✅ Real-time breadcrumbs |
| **Debugging** | ✅ Request/response analysis | ✅ Complete event trail |
| **Performance** | ✅ Performance metrics | ✅ Timing breadcrumbs |
| **Compliance** | ⚠️ Limited | ✅ Full audit trail |
| **Integration** | Web-based UI | ✅ Integrated in application |

**Advantages of Breadcrumb Logging**:

- Integrated directly into application (no separate tool)
- Complete event trail (not just API calls)
- State snapshots at critical points
- Works with any testing framework
- No external dependencies

---

## References

- [TUI Breadcrumb Logging Guide](./TUI_BREADCRUMB_LOGGING.md)
- [OpenAlgo API Analyzer](https://github.com/marketcalls/openalgo)
- [OpenAlgo Integration Patterns](research/integration/OPENALGO_INTEGRATION_PATTERNS.md)

---

**Last Updated**: 2025-11-30
