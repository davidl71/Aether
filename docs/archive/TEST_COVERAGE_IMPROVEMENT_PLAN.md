# Test Coverage Improvement Plan

**Date**: 2025-12-24
**Status**: Updated for Rust-first stack (native C++ removed)

> **Note:** The native C++ build and Catch2 tests were removed. This plan now targets **Rust** (`agents/backend`) and **Python** (`agents/nautilus`).

## Current State Analysis

### Test Infrastructure

- ✅ Rust: `cargo test` in `agents/backend`
- ✅ Python: pytest in `agents/nautilus`
- ✅ TUI E2E: `just test-tui-e2e`
- ✅ ShellSpec: `./scripts/run_tests.sh`
- ⚠️ Coverage reporting: Use `cargo llvm-cov` / `cargo tarpaulin` (Rust), `pytest --cov` (nautilus)
- ❌ Coverage enforcement: No minimum coverage threshold in CI

### Current Test Coverage

- **Rust**: Run `cd agents/backend && cargo llvm-cov test` (or tarpaulin) for coverage.
- **Python (nautilus)**: `cd agents/nautilus && uv run pytest tests/ --cov=nautilus_agent`.

### Coverage Gaps Identified

**Critical areas (Rust):**

1. **ib_adapter** – IB/TWS integration
2. **risk** – Risk and position sizing
3. **api** – REST and order/position handling
4. **quant** – Pricing/margin
5. **nats_adapter** – Messaging

**Python (nautilus):** Strategy and NATS bridge in `agents/nautilus/tests/`.

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

**Priority order (Rust crates):**

1. **ib_adapter** – IB/TWS connectivity, market data, order placement, error handling
2. **risk** – Position risk, portfolio aggregation, sizing
3. **api** – REST, order/position handling, validation
4. **quant** – Pricing, margin, options logic

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

**Priority test coverage:**

1. **Rust** `ib_adapter` – TWS/IB connectivity and client behavior
2. **Rust** `risk` – Risk and position sizing
3. **Rust** `api` – Order/position and validation
4. **Python** `agents/nautilus/tests/` – Strategy and NATS bridge

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
