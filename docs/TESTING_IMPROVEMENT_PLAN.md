# Testing Improvement Plan: Fix Failing Tests & Increase Coverage to 30%

**Date**: 2025-01-27
**Status**: 📋 Plan Created
**Target**: Fix all failing tests and achieve 30%+ code coverage

---

## Current Status

### Test Status

- **Python Tests**: ✅ 30 tests passing (75% coverage for `security.py`)
- **C++ Tests**: ⚠️ Status unknown (needs verification)
- **Overall Coverage**: 🟠 14.5% (scorecard metric)
- **Target Coverage**: 🎯 30%+

### Known Issues

- Python tests fixed (per `TEST_FIXES_COMPLETE.md`)
- C++ test build status unknown
- Coverage measurement not automated
- No baseline coverage report for entire codebase

---

## Phase 1: Identify and Fix Failing Tests

### 1.1 Build and Run C++ Tests

**Objective**: Verify C++ tests compile and run successfully

**Tasks**:

1. [ ] Check if build directory exists
2. [ ] Configure CMake build with test support
3. [ ] Build test executable (`box_spread_tests`)
4. [ ] Run all C++ tests: `ctest --test-dir build --output-on-failure`
5. [ ] Document any compilation errors
6. [ ] Document any runtime failures
7. [ ] Document any assertion failures

**Commands**:

```bash
cd native
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Debug -DBUILD_TESTING=ON
cmake --build .
ctest --output-on-failure
```

**Expected Output**: All tests passing or list of failures

**Estimated Time**: 1-2 hours

---

### 1.2 Fix C++ Compilation Errors

**Objective**: Resolve any build/compilation issues

**Common Issues to Check**:

- Missing dependencies
- Incorrect include paths
- Linker errors
- CMake configuration issues
- Missing source files

**Tasks**:

1. [ ] Fix missing includes
2. [ ] Fix linker errors
3. [ ] Update CMakeLists.txt if needed
4. [ ] Verify all dependencies available
5. [ ] Rebuild and verify compilation succeeds

**Estimated Time**: 2-4 hours (depends on errors)

---

### 1.3 Fix C++ Runtime Errors

**Objective**: Resolve test execution failures

**Common Issues**:

- Test setup/teardown problems
- Mock data initialization
- API client initialization
- Test environment configuration
- Memory leaks or crashes

**Tasks**:

1. [ ] Fix test fixture initialization
2. [ ] Fix mock data setup
3. [ ] Fix API client configuration
4. [ ] Fix test environment setup
5. [ ] Add missing test data files
6. [ ] Fix memory management issues

**Estimated Time**: 2-4 hours (depends on errors)

---

### 1.4 Fix C++ Assertion Failures

**Objective**: Fix incorrect test expectations

**Common Issues**:

- Outdated assertions
- Floating-point comparison issues
- Timing-related failures
- Race conditions
- Incorrect test logic

**Tasks**:

1. [ ] Review failing assertions
2. [ ] Update outdated expectations
3. [ ] Fix floating-point comparisons (use Approx)
4. [ ] Fix timing issues (add delays, use mocks)
5. [ ] Fix race conditions
6. [ ] Verify test logic correctness

**Estimated Time**: 2-4 hours (depends on failures)

---

### 1.5 Verify Python Tests

**Objective**: Ensure all Python tests still pass

**Tasks**:

1. [ ] Run all Python tests: `pytest python/tests/ python/integration/ -v`
2. [ ] Fix any new failures
3. [ ] Address deprecation warnings (optional)
4. [ ] Verify test coverage still > 30% for security module

**Commands**:

```bash
pytest python/tests/ python/integration/ -v
pytest python/tests/test_security.py --cov=python/services/security --cov-report=term
```

**Estimated Time**: 30 minutes - 1 hour

---

## Phase 2: Measure Current Coverage

### 2.1 Set Up C++ Coverage Tools

**Objective**: Configure coverage measurement for C++ code

**Tasks**:

1. [ ] Install coverage tools (gcov, lcov)
2. [ ] Configure CMake with coverage flags
3. [ ] Build with coverage enabled
4. [ ] Run tests with coverage
5. [ ] Generate coverage report

**Commands**:

```bash
# macOS
brew install lcov

# Configure build with coverage
cd native
mkdir -p build-coverage
cd build-coverage
cmake .. \
  -DCMAKE_BUILD_TYPE=Debug \
  -DCMAKE_CXX_FLAGS="--coverage -g -O0" \
  -DCMAKE_EXE_LINKER_FLAGS="--coverage"

# Build
cmake --build .

# Run tests
ctest --output-on-failure

# Generate coverage
lcov --capture --directory . --output-file coverage.info
lcov --remove coverage.info \
  '/usr/*' \
  '*/third_party/*' \
  '*/tests/*' \
  --output-file coverage_filtered.info

# Generate HTML report
genhtml coverage_filtered.info --output-directory coverage_html
open coverage_html/index.html
```

**Expected Output**: Coverage report showing current coverage percentage

**Estimated Time**: 1-2 hours

---

### 2.2 Set Up Python Coverage Tools

**Objective**: Configure coverage measurement for Python code

**Tasks**:

1. [ ] Install pytest-cov: `pip install pytest-cov coverage`
2. [ ] Create `.coveragerc` configuration file
3. [ ] Run tests with coverage
4. [ ] Generate coverage report

**Commands**:

```bash
pip install pytest-cov coverage

# Create .coveragerc
cat > python/.coveragerc << 'EOF'
[run]
source = python
omit =
    */tests/*
    */test_*.py
    */__pycache__/*

[report]
exclude_lines =
    pragma: no cover
    def __repr__
    raise AssertionError
    raise NotImplementedError
    if __name__ == .__main__.:
    if TYPE_CHECKING:
EOF

# Run with coverage
pytest python/tests/ python/integration/ \
  --cov=python/services \
  --cov=python/tui \
  --cov-report=html \
  --cov-report=term

# View report
open htmlcov/index.html
```

**Expected Output**: Coverage report showing current Python coverage

**Estimated Time**: 30 minutes - 1 hour

---

### 2.3 Generate Baseline Coverage Report

**Objective**: Document current coverage state

**Tasks**:

1. [ ] Run C++ coverage analysis
2. [ ] Run Python coverage analysis
3. [ ] Combine coverage metrics
4. [ ] Document baseline in `docs/COVERAGE_BASELINE.md`
5. [ ] Identify coverage gaps

**Coverage Metrics to Track**:

- Overall coverage percentage
- Coverage by module/component
- Uncovered lines/files
- Critical path coverage
- Test coverage trends

**Estimated Time**: 1 hour

---

## Phase 3: Increase Coverage to 30%

### 3.1 Prioritize Coverage Gaps

**Objective**: Identify highest-impact areas for coverage improvement

**Priority Areas** (based on `TEST_COVERAGE_ANALYSIS.md`):

**High Priority**:

1. Core trading logic (`box_spread_strategy.cpp`)
2. Order management (`order_manager.cpp`)
3. Risk calculations (`risk_calculator.cpp`)
4. TWS client (`tws_client.cpp`)
5. Configuration (`config_manager.cpp`)

**Medium Priority**:

1. Option chain (`option_chain.cpp`)
2. Rate limiter (`rate_limiter.cpp`)
3. Hedge manager (`hedge_manager.cpp`)
4. Box spread bag (`box_spread_bag.cpp`)

**Low Priority**:

1. ML predictor (optional)
2. Mock data generator (testing utility)
3. TUI components (separate testing)

**Tasks**:

1. [ ] Review coverage report
2. [ ] Identify uncovered critical paths
3. [ ] Prioritize by business impact
4. [ ] Create task list for test additions

**Estimated Time**: 1 hour

---

### 3.2 Add Tests for Critical C++ Components

**Objective**: Increase C++ coverage to 30%+

**Focus Areas**:

- Box spread calculation logic
- Order validation and execution
- Risk management calculations
- Market data handling
- Configuration validation

**Tasks**:

1. [ ] Review existing test files
2. [ ] Identify missing test cases
3. [ ] Add tests for uncovered code paths
4. [ ] Add edge case tests
5. [ ] Add error handling tests
6. [ ] Verify coverage increases

**Test Files to Enhance**:

- `test_box_spread_strategy.cpp` - Add full execution flow tests
- `test_order_manager.cpp` - Add multi-leg execution tests
- `test_tws_client.cpp` - Add market data request tests
- `test_config_manager.cpp` - Add edge case tests

**Estimated Time**: 8-12 hours

---

### 3.3 Add Tests for Python Components

**Objective**: Increase Python coverage to 30%+

**Focus Areas**:

- Service layer (`python/services/`)
- Integration modules (`python/integration/`)
- TUI models (`python/tui/`)

**Tasks**:

1. [ ] Review existing Python tests
2. [ ] Identify uncovered modules
3. [ ] Add tests for `swiftness_api.py`
4. [ ] Add integration tests
5. [ ] Add TUI model tests
6. [ ] Verify coverage increases

**Test Files to Create/Enhance**:

- `python/tests/test_swiftness_api.py` (if missing)
- `python/integration/test_*.py` (expand existing)
- `python/tui/tests/test_models.py` (enhance existing)

**Estimated Time**: 4-6 hours

---

### 3.4 Add Integration Tests

**Objective**: Test component interactions

**Tasks**:

1. [ ] Review existing integration tests
2. [ ] Add TWS client integration tests (if missing)
3. [ ] Add market data pipeline tests
4. [ ] Add end-to-end workflow tests
5. [ ] Add multi-component interaction tests

**Integration Test Files**:

- `test_tws_integration.cpp` (verify exists and passes)
- `test_market_data_integration.cpp` (verify exists and passes)
- `test_box_spread_e2e.cpp` (verify exists and passes)

**Estimated Time**: 4-6 hours

---

### 3.5 Verify Coverage Target Achieved

**Objective**: Confirm 30%+ coverage across codebase

**Tasks**:

1. [ ] Run full coverage analysis
2. [ ] Verify overall coverage >= 30%
3. [ ] Verify critical paths covered
4. [ ] Document coverage achievement
5. [ ] Update coverage baseline

**Success Criteria**:

- ✅ Overall coverage >= 30%
- ✅ All critical paths covered
- ✅ All tests passing
- ✅ Coverage report generated

**Estimated Time**: 1 hour

---

## Phase 4: Set Up Coverage Reporting

### 4.1 Automate Coverage Generation

**Objective**: Make coverage reporting part of build process

**Tasks**:

1. [ ] Create coverage generation script
2. [ ] Add CMake target for coverage
3. [ ] Add Python coverage script
4. [ ] Document coverage commands
5. [ ] Test coverage automation

**Scripts to Create**:

- `scripts/generate_coverage.sh` - Combined C++ and Python coverage
- `scripts/generate_cpp_coverage.sh` - C++ coverage only
- `scripts/generate_python_coverage.sh` - Python coverage only

**Estimated Time**: 2-3 hours

---

### 4.2 Add Coverage to CI/CD

**Objective**: Track coverage in continuous integration

**Tasks**:

1. [ ] Review existing CI/CD setup
2. [ ] Add coverage generation step
3. [ ] Add coverage threshold check (30%)
4. [ ] Upload coverage reports as artifacts
5. [ ] Add coverage badge to README (optional)

**CI/CD Integration**:

- GitHub Actions workflow
- Coverage threshold enforcement
- Coverage report artifacts
- Coverage trend tracking

**Estimated Time**: 2-3 hours

---

### 4.3 Document Coverage Process

**Objective**: Make coverage measurement repeatable

**Tasks**:

1. [ ] Update `TEST_COVERAGE_SETUP.md` with current process
2. [ ] Document coverage commands
3. [ ] Document coverage interpretation
4. [ ] Add coverage troubleshooting guide
5. [ ] Update project README with coverage info

**Estimated Time**: 1-2 hours

---

## Implementation Timeline

### Week 1: Fix Failing Tests

- **Day 1**: Build and run C++ tests, identify failures
- **Day 2**: Fix compilation errors
- **Day 3**: Fix runtime errors
- **Day 4**: Fix assertion failures
- **Day 5**: Verify all tests passing

### Week 2: Measure Coverage

- **Day 1**: Set up C++ coverage tools
- **Day 2**: Set up Python coverage tools
- **Day 3**: Generate baseline coverage report
- **Day 4**: Analyze coverage gaps
- **Day 5**: Prioritize coverage improvements

### Week 3: Increase Coverage

- **Days 1-2**: Add C++ tests for critical components
- **Days 3-4**: Add Python tests
- **Day 5**: Add integration tests

### Week 4: Automation & Documentation

- **Days 1-2**: Set up coverage automation
- **Day 3**: Add CI/CD integration
- **Days 4-5**: Documentation and verification

**Total Estimated Time**: 40-60 hours over 4 weeks

---

## Success Criteria

### Phase 1: Tests Fixed

- ✅ All C++ tests compile successfully
- ✅ All C++ tests pass
- ✅ All Python tests pass
- ✅ No flaky tests

### Phase 2: Coverage Measured

- ✅ Coverage tools configured
- ✅ Baseline coverage report generated
- ✅ Coverage gaps identified
- ✅ Coverage metrics documented

### Phase 3: Coverage Increased

- ✅ Overall coverage >= 30%
- ✅ Critical paths covered
- ✅ All new tests passing
- ✅ Coverage report shows improvement

### Phase 4: Automation Complete

- ✅ Coverage generation automated
- ✅ CI/CD integration working
- ✅ Coverage documentation complete
- ✅ Coverage process repeatable

---

## Risk Mitigation

### Risk 1: C++ Tests Don't Compile

**Mitigation**: Fix dependencies, update CMake configuration, check compiler compatibility

### Risk 2: Coverage Tools Not Available

**Mitigation**: Use alternative tools (llvm-cov for macOS), document platform-specific setup

### Risk 3: Coverage Target Too Ambitious

**Mitigation**: Focus on critical paths first, set intermediate targets (20%, 25%, 30%)

### Risk 4: Tests Take Too Long

**Mitigation**: Prioritize high-impact tests, use test fixtures, optimize test execution

---

## Related Documentation

- `docs/TEST_COVERAGE_SETUP.md` - Coverage setup guide
- `docs/TEST_FIXES_COMPLETE.md` - Previous test fixes
- `docs/research/analysis/TEST_COVERAGE_ANALYSIS.md` - Coverage analysis
- `docs/research/integration/TESTING_STRATEGY.md` - Testing strategy
---

**Last Updated**: 2025-01-27
**Status**: Plan created, ready for implementation
