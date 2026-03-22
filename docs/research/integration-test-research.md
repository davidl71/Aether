# Integration Test Research: api.strategy.execute → place_bag_order

**Source:** Aether codebase analysis  
**Date:** 2026-03-22  
**Status:** Completed

## Existing Implementation

The `place_bag_order` functionality is already implemented:

| Component | Location | Status |
|-----------|----------|--------|
| `IbAdapter::place_bag_order()` | `ib_adapter/src/lib.rs:365` | ✅ Implemented |
| `conId` resolution | `ib_adapter/src/lib.rs:79` | ✅ Implemented |
| Tests | `ib_adapter/src/lib.rs:767-832` | ✅ Unit tests exist |

## Current Test Coverage

### Unit Tests (ib_adapter)

```rust
#[tokio::test]
async fn place_bag_order_rejects_when_not_connected() { ... }

#[tokio::test]
async fn place_bag_order_rejects_empty_legs() { ... }

#[tokio::test]
async fn place_bag_order_rejects_live_trading() { ... }
```

### What's Missing

1. **Integration test** - End-to-end test from api → broker
2. **Mock broker** - No `MockBroker` for strategy testing
3. **Contract resolution** - Real TWS connection required

## Recommended Test Structure

```rust
// agents/backend/crates/api/tests/strategy_integration_test.rs

#[tokio::test]
async fn test_strategy_executes_box_spread() {
    // Setup mock broker
    let mock = MockBrokerEngine::new();
    mock.expect_place_bag_order()
        .returning(|_| Ok(12345));

    // Create strategy context with mock
    let strategy = BoxSpreadStrategy::new(mock);

    // Execute strategy
    let result = strategy.execute(Scenario::default()).await;

    assert!(result.is_ok());
}
```

## Task Assessment

**T-1774192598823632000** describes work that is partially complete:
- `place_bag_order` is implemented ✅
- `conId` resolution is implemented ✅
- Integration test from `api.strategy` is missing ❌

## Recommendation

Create integration test with mock broker:
1. Add `MockBrokerEngine` implementation for testing
2. Create `api/tests/strategy_execution_test.rs`
3. Test strategy → broker adapter path without real TWS

## Related Tasks

- T-1774192598823632000: Integration test (needs clarification - partial implementation exists)
