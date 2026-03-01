# Coverage Gap Analysis

**Date**: 2025-12-11
**Status**: Initial Analysis Complete
**Target**: 30%+ Overall Coverage

---

## Executive Summary

This document identifies coverage gaps and prioritizes test additions to achieve 30%+ code coverage. Analysis is based on codebase structure, business criticality, and existing test coverage.

---

## High Priority Coverage Gaps

### 🔴 Critical Business Logic (Priority 1)

**Impact**: Core functionality, financial calculations, risk management

#### C++ Components (Blocked - Missing Libraries)

1. **Box Spread Strategy** (`native/src/strategies/box_spread/box_spread_strategy.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Full execution flow, multi-leg scenarios, error handling
   - **Target Coverage**: 50%+
   - **Test File**: `native/tests/test_box_spread_strategy.cpp`

2. **Order Manager** (`native/src/order_manager.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Multi-leg execution, order validation, error recovery
   - **Target Coverage**: 50%+
   - **Test File**: `native/tests/test_order_manager.cpp`

3. **Risk Calculator** (`native/src/risk_calculator.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Edge cases, boundary conditions, calculation accuracy
   - **Target Coverage**: 50%+
   - **Test File**: `native/tests/test_risk_calculator.cpp`

#### Python Components (Ready to Test)

1. **Swiftness API** (`python/services/swiftness_api.py`)
   - **Current**: ⚠️ **No tests found**
   - **Gaps**: All functionality untested
   - **Target Coverage**: 40%+
   - **Test File**: `python/tests/test_swiftness_api.py` (needs creation)
   - **Classes to Test**:
     - `ExchangeRateUpdate`
     - `CashFlowRequest`
     - `PositionSnapshotResponse`
     - `CashFlowEventResponse`
     - `ValidationReportResponse`
     - `PortfolioValueResponse`

2. **Environment Config** (`python/services/environment_config.py`)
   - **Current**: ⚠️ **No tests found**
   - **Gaps**: Configuration loading, validation, reloading
   - **Target Coverage**: 40%+
   - **Test File**: `python/tests/test_environment_config.py` (needs creation)
   - **Functions to Test**:
     - `get_config()`
     - `reload_config()`
     - `EnvironmentConfig` class methods

---

## Medium Priority Coverage Gaps

### 🟡 Supporting Components (Priority 2)

#### C++ Components (Blocked - Missing Libraries)

1. **TWS Client** (`native/src/tws_client.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Market data requests, connection handling, error recovery
   - **Target Coverage**: 30%+
   - **Test File**: `native/tests/test_tws_client.cpp`

2. **Config Manager** (`native/src/config_manager.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Edge cases, validation, file I/O error handling
   - **Target Coverage**: 30%+
   - **Test File**: `native/tests/test_config_manager.cpp`

3. **Option Chain** (`native/src/option_chain.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Chain building, filtering, edge cases
   - **Target Coverage**: 30%+
   - **Test File**: `native/tests/test_option_chain.cpp`

4. **Rate Limiter** (`native/src/rate_limiter.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Concurrent access, edge cases, timeout handling
   - **Target Coverage**: 30%+
   - **Test File**: `native/tests/test_rate_limiter.cpp`

5. **Hedge Manager** (`native/src/hedge_manager.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Hedge calculation, position management
   - **Target Coverage**: 30%+
   - **Test File**: `native/tests/test_hedge_manager.cpp`

6. **Box Spread Bag** (`native/src/strategies/box_spread/box_spread_bag.cpp`)
   - **Current**: Basic tests exist
   - **Gaps**: Bag management, position tracking
   - **Target Coverage**: 30%+
   - **Test File**: `native/tests/test_box_spread_bag.cpp`

#### Python Components (Ready to Test)

1. **TUI Models** (`python/tui/models.py`)
   - **Current**: Basic tests exist (`python/tui/tests/test_models.py`)
   - **Gaps**: Model validation, data transformation, edge cases
   - **Target Coverage**: 30%+
   - **Test File**: `python/tui/tests/test_models.py` (enhance existing)

2. **TUI Components** (`python/tui/components/`)
   - **Current**: ⚠️ **No tests found**
   - **Gaps**: All component functionality untested
   - **Target Coverage**: 25%+
   - **Test Files**: Create component tests
   - **Components**:
     - `cash_flow.py`
     - `opportunity_simulation.py`
     - `relationship_visualization.py`
     - `unified_positions.py`

3. **Security Integration Helper** (`python/services/security_integration_helper.py`)
   - **Current**: ⚠️ **No tests found**
   - **Gaps**: FastAPI integration, middleware setup
   - **Target Coverage**: 30%+
   - **Test File**: `python/tests/test_security_integration_helper.py` (needs creation)
   - **Functions to Test**:
     - `add_security_to_app()`
     - `add_security_headers_middleware()`

4. **Integration Modules** (`python/integration/`)
   - **Current**: Some integration tests exist
   - **Gaps**: Expand coverage for all integration modules
   - **Target Coverage**: 25%+
   - **Test Files**: Expand existing `test_*.py` files

---

## Low Priority Coverage Gaps

### 🟢 Nice to Have (Priority 3)

1. **TUI App** (`python/tui/app.py`)
   - **Current**: ⚠️ **No tests found**
   - **Gaps**: Application initialization, routing
   - **Target Coverage**: 20%+

2. **TUI Providers** (`python/tui/providers.py`)
   - **Current**: ⚠️ **No tests found**
   - **Gaps**: Data provider interfaces
   - **Target Coverage**: 20%+

3. **TUI Config** (`python/tui/config.py`)
   - **Current**: ⚠️ **No tests found**
   - **Gaps**: Configuration management
   - **Target Coverage**: 20%+

---

## Test File Status

### Existing Test Files

**C++ Tests** (`native/tests/`):

- ✅ `test_box_spread_strategy.cpp` - Needs enhancement
- ✅ `test_order_manager.cpp` - Needs enhancement
- ✅ `test_risk_calculator.cpp` - Needs enhancement
- ✅ `test_tws_client.cpp` - Needs enhancement
- ✅ `test_config_manager.cpp` - Needs enhancement
- ✅ `test_option_chain.cpp` - Needs enhancement
- ✅ `test_rate_limiter.cpp` - Needs enhancement
- ✅ `test_hedge_manager.cpp` - Needs enhancement
- ✅ `test_box_spread_bag.cpp` - Needs enhancement
- ✅ `test_path_validator.cpp` - Good coverage
- ✅ `test_convexity_calculator.cpp` - Basic tests
- ✅ `test_alpaca_adapter.cpp` - Basic tests
- ✅ `test_tws_integration.cpp` - Integration tests
- ✅ `test_market_data_integration.cpp` - Integration tests
- ✅ `test_box_spread_e2e.cpp` - End-to-end tests

**Python Tests** (`python/tests/`):

- ✅ `test_security.py` - Good coverage (5 test classes)
- ✅ `run_security_tests.py` - Test runner

**Python Integration Tests** (`python/integration/`):

- ✅ `scripts/swiftness_integration_manual.py` – Manual Swiftness integration check
- ✅ `test_swiftness_import.py` - Import tests
- ✅ `test_relationship_graph.py` - Graph tests
- ✅ `test_nats_client.py` - NATS client tests

**TUI Tests** (`python/tui/tests/`):

- ✅ `test_models.py` - Basic model tests (needs enhancement)

### Missing Test Files (Need Creation)

**Python Tests**:

- ❌ `python/tests/test_swiftness_api.py` - **HIGH PRIORITY**
- ❌ `python/tests/test_environment_config.py` - **HIGH PRIORITY**
- ❌ `python/tests/test_security_integration_helper.py` - **MEDIUM PRIORITY**
- ❌ `python/tui/tests/test_components.py` - **MEDIUM PRIORITY**
- ❌ `python/tui/tests/test_app.py` - **LOW PRIORITY**
- ❌ `python/tui/tests/test_providers.py` - **LOW PRIORITY**
- ❌ `python/tui/tests/test_config.py` - **LOW PRIORITY**

---

## Prioritized Test Addition Plan

### Phase 1: Critical Python Components (Immediate - No Blockers)

**Estimated Time**: 4-6 hours
**Target Coverage Increase**: +5-8%

1. **Create `test_swiftness_api.py`** (Priority 1)
   - Test all model classes
   - Test API request/response handling
   - Test validation logic
   - **Target**: 40%+ coverage

2. **Create `test_environment_config.py`** (Priority 1)
   - Test configuration loading
   - Test configuration validation
   - Test reload functionality
   - **Target**: 40%+ coverage

3. **Enhance `test_models.py`** (Priority 2)
   - Expand existing tests
   - Add edge case tests
   - Add validation tests
   - **Target**: 30%+ coverage

### Phase 2: Integration & Component Tests (After Phase 1)

**Estimated Time**: 3-4 hours
**Target Coverage Increase**: +3-5%

1. **Create `test_security_integration_helper.py`** (Priority 2)
   - Test FastAPI integration
   - Test middleware setup
   - **Target**: 30%+ coverage

2. **Create TUI component tests** (Priority 2)
   - Test cash flow component
   - Test opportunity simulation
   - Test relationship visualization
   - **Target**: 25%+ coverage

3. **Expand integration tests** (Priority 2)
   - Enhance existing integration test files
   - Add more test scenarios
   - **Target**: 25%+ coverage

### Phase 3: C++ Components (Blocked - Wait for Libraries)

**Estimated Time**: 8-12 hours
**Target Coverage Increase**: +10-15%

1. **Enhance existing C++ test files** (Priority 1)
   - Add full execution flow tests
   - Add edge case tests
   - Add error handling tests
   - **Target**: 50%+ coverage for critical components

2. **Add integration tests** (Priority 2)
   - Enhance existing integration tests
   - Add end-to-end workflow tests
   - **Target**: 30%+ coverage

---

## Coverage Targets by Component

### C++ Components

| Component | Current | Target | Priority | Status |
|-----------|---------|--------|----------|--------|
| Box Spread Strategy | ~20% | 50% | High | Blocked |
| Order Manager | ~20% | 50% | High | Blocked |
| Risk Calculator | ~20% | 50% | High | Blocked |
| TWS Client | ~15% | 30% | Medium | Blocked |
| Config Manager | ~20% | 30% | Medium | Blocked |
| Option Chain | ~15% | 30% | Medium | Blocked |
| Rate Limiter | ~20% | 30% | Medium | Blocked |
| Hedge Manager | ~15% | 30% | Medium | Blocked |
| Box Spread Bag | ~15% | 30% | Medium | Blocked |

### Python Components

| Component | Current | Target | Priority | Status |
|-----------|---------|--------|----------|--------|
| Security | ~75% | 30%+ ✅ | High | ✅ Good |
| Swiftness API | 0% | 40% | High | ⚠️ No tests |
| Environment Config | 0% | 40% | High | ⚠️ No tests |
| Security Integration | 0% | 30% | Medium | ⚠️ No tests |
| TUI Models | ~20% | 30% | Medium | Needs enhancement |
| TUI Components | 0% | 25% | Medium | ⚠️ No tests |
| Integration Modules | ~15% | 25% | Medium | Needs expansion |

---

## Recommended Test Addition Order

### Immediate (Can Start Now)

1. ✅ **Create `test_swiftness_api.py`** - Highest impact, no blockers
2. ✅ **Create `test_environment_config.py`** - High impact, no blockers
3. ✅ **Enhance `test_models.py`** - Medium impact, existing file

### After Baseline Measurement

4. Generate baseline coverage report (Task 2.3)
5. Review actual coverage gaps from report
6. Prioritize based on actual metrics
7. Create missing test files
8. Enhance existing test files

### When C++ Libraries Available

9. Enhance C++ test files
10. Add C++ integration tests
11. Verify overall 30%+ coverage target

---

## Success Metrics

- **Overall Coverage**: 30%+ (combined C++ and Python)
- **Critical Paths**: 50%+ (core trading logic, security)
- **Python Services**: 30%+ (all service modules)
- **C++ Core Logic**: 50%+ (when libraries available)
- **Integration Tests**: 25%+ (end-to-end workflows)

---

**Last Updated**: 2025-12-11
**Next Review**: After baseline coverage measurement (Task 2.3)
