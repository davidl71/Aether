# Test Coverage Improvement Plan

**Date**: 2025-12-24
**Current Coverage**: 27.2% (9.1% test ratio)
**Target Coverage**: 50%+ (30%+ test ratio)
**Blocker**: Testing score is preventing production readiness

## Current State Analysis

### Test Infrastructure

- ✅ Python: pytest with pytest-cov configured
- ✅ C++: Catch2 framework with CMake/CTest
- ✅ Coverage scripts: `generate_coverage.sh`, `generate_python_coverage.sh`, `generate_cpp_coverage.sh`
- ✅ CI/CD: GitHub Actions test workflow exists
- ⚠️ Coverage reporting: Not fully integrated into CI/CD
- ❌ Coverage enforcement: No minimum coverage threshold

### Current Test Coverage

- **Test Files**: 43 files
- **Test Lines**: 10,806 lines
- **Test Ratio**: 9.1% (target: 30%+)
- **Status**: ⚠️ Coverage too low

### Coverage Gaps Identified

**Critical Modules with Low/No Coverage:**

1. **TWS Client** (`native/src/tws_client.cpp`) - Core trading integration
2. **Risk Calculator** (`native/src/risk_calculator.cpp`) - Risk management
3. **Box Spread Strategy** (`native/src/strategies/box_spread/`) - Core strategy logic
4. **Order Manager** (`native/src/order_manager.cpp`) - Order execution
5. **Python Services** (`python/services/`) - Backend services

## Improvement Plan

### Phase 1: Infrastructure Setup (High Priority)

**Tasks:**

1. ✅ Add coverage reporting to CI/CD workflow
2. ✅ Set minimum coverage threshold (50%)
3. ✅ Generate coverage reports in CI/CD
4. ✅ Upload coverage to Codecov or similar service
5. ✅ Add coverage badges to README

**Files to Update:**

- `.github/workflows/test.yml` - Add coverage collection and reporting
- `pyproject.toml` or `setup.cfg` - Add coverage configuration
- `CMakeLists.txt` - Ensure C++ coverage is enabled

### Phase 2: Critical Module Testing (High Priority)

**Priority Order:**

1. **TWS Client** - Core trading functionality
   - Connection management
   - Market data requests
   - Order placement
   - Error handling

2. **Risk Calculator** - Risk management
   - Position risk calculation
   - Portfolio risk aggregation
   - Greeks calculation (when implemented)
   - Correlation calculation (when implemented)

3. **Box Spread Strategy** - Core strategy
   - Opportunity identification
   - Profitability calculation
   - Risk validation
   - Execution logic

4. **Order Manager** - Order execution
   - Order validation
   - Multi-leg order coordination
   - Order state management

### Phase 3: Integration Testing (Medium Priority)

**Areas:**

1. End-to-end box spread workflow
2. TWS API integration (paper trading)
3. Risk limit enforcement
4. Error recovery scenarios

### Phase 4: Python Services Testing (Medium Priority)

**Modules:**

1. `python/services/` - Backend services
2. `python/integration/` - Integration modules
3. `python/tui/` - TUI components

## Implementation Steps

### Step 1: Update CI/CD for Coverage

**File**: `.github/workflows/test.yml`

```yaml
- name: Run tests with coverage
  run: |
    pytest python/tests/ -v --cov=python/services --cov=python/integration --cov-report=xml --cov-report=term

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./coverage.xml
    fail_ci_if_error: false
    minimum_coverage: 50.0
```

### Step 2: Add Coverage Threshold

**File**: `pyproject.toml` or `.coveragerc`

```ini
[coverage:run]
source = python
omit = */tests/*,*/test_*.py,*/__pycache__/*

[coverage:report]
precision = 2
show_missing = True
skip_covered = False
fail_under = 50.0
```

### Step 3: Create Test Files for Critical Modules

**Priority Test Files:**

1. `native/tests/tws_client_test.cpp` - TWS client tests
2. `native/tests/risk_calculator_test.cpp` - Risk calculator tests
3. `native/tests/box_spread_strategy_test.cpp` - Strategy tests
4. `python/tests/test_tws_client.py` - Python TWS wrapper tests
5. `python/tests/test_risk_calculator.py` - Python risk tests

### Step 4: Add Mock/Stub Infrastructure

**For TWS API Testing:**

- Create mock TWS client for unit tests
- Use paper trading for integration tests
- Stub market data responses

## Success Metrics

### Short-term (1-2 weeks)

- Coverage increases from 27.2% to 40%+
- Critical modules (TWS client, risk calculator) have 60%+ coverage
- CI/CD reports coverage in every run

### Medium-term (1 month)

- Overall coverage reaches 50%+
- All critical trading paths have tests
- Coverage reports generated automatically

### Long-term (2-3 months)

- Coverage reaches 70%+ (stretch goal)
- Integration tests cover end-to-end workflows
- Test ratio reaches 30%+

## Dependencies

- T-213: Option chain request (needed for strategy tests)
- T-216: Greeks calculation (needed for risk tests)
- T-217: Correlation calculation (needed for portfolio risk tests)

## Next Steps

1. ✅ Update CI/CD workflow with coverage reporting
2. ⏭️ Add coverage threshold configuration
3. ⏭️ Create test files for TWS client
4. ⏭️ Create test files for risk calculator
5. ⏭️ Create test files for box spread strategy
6. ⏭️ Run coverage analysis to identify gaps
7. ⏭️ Add tests for uncovered critical paths

---

**Related Tasks:**

- T-198: Analyze test coverage gaps and create improvement plan
- T-199: Add test coverage for python/services modules
- T-200: Add test coverage for critical integration modules
- T-201: Add test coverage for TUI modules
- T-202: Set up automated coverage reporting in CI/CD
