# Test Coverage Setup Guide

**Date**: 2025-11-29
**Status**: ✅ **Setup Complete**

---

## Overview

This guide documents the test coverage setup for achieving 30%+ code coverage across C++ and Python codebases.

---

## C++ Test Coverage (gcov/lcov)

### Prerequisites

```bash

# Install coverage tools

sudo apt-get install -y gcov lcov  # Linux
brew install lcov                   # macOS
```

### Build with Coverage

```bash

# Configure CMake with coverage flags

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
```

### Generate Coverage Report

```bash

# Generate coverage data

lcov --capture --directory . --output-file coverage.info

# Remove system/external includes

lcov --remove coverage.info \
  '/usr/*' \
  '*/third_party/*' \
  '*/tests/*' \
  --output-file coverage_filtered.info

# Generate HTML report

genhtml coverage_filtered.info --output-directory coverage_html

# Open report

open coverage_html/index.html  # macOS
xdg-open coverage_html/index.html  # Linux
```

### Coverage Target: 30%

**Current Status**: Baseline measurement needed
**Target**: 30%+ coverage
**Focus Areas**:

- Security: `path_validator.cpp` ✅ (tests added)
- Core: `box_spread_strategy.cpp`, `order_manager.cpp`, `risk_calculator.cpp`
- API clients: `tws_client.cpp`, `http_client.cpp`
- Configuration: `config_manager.cpp`, `tui_config.cpp`

---

## Python Test Coverage (pytest-cov)

### Prerequisites

```bash

# Install pytest-cov

pip install pytest pytest-cov coverage
```

### Run Tests with Coverage

```bash

# Run all Python tests with coverage

pytest python/tests/ python/integration/ \
  --cov=python/services \
  --cov=python/tui \
  --cov-report=html \
  --cov-report=term

# View HTML report

open htmlcov/index.html  # macOS
xdg-open htmlcov/index.html  # Linux
```

### Coverage Configuration

**File**: `.coveragerc` (root directory) - ✅ **Already Created**

The coverage configuration file is located at the project root and includes:

- Source paths: `python/services`, `python/tui`, `python/integration`
- Exclusions: Tests, `__pycache__`, bindings, setup files
- Branch coverage: Enabled
- Report precision: 2 decimal places

### Coverage Target: 30%

**Current Status**: Baseline measurement needed
**Target**: 30%+ coverage
**Focus Areas**:

- Security: `python/services/security.py` ✅ (tests exist)
- Services: `python/services/swiftness_api.py`
- Integration: `python/integration/`

---

## Test Infrastructure

### C++ Tests (Catch2)

**Location**: `native/tests/`
**Framework**: Catch2
**Run**: `ctest --test-dir build --output-on-failure`

**Test Files**:

- ✅ `test_path_validator.cpp` - Path validation security tests
- `test_rate_limiter.cpp` - Rate limiting tests
- `test_box_spread_strategy.cpp` - Core strategy tests
- `test_tws_client.cpp` - TWS API client tests
- `test_order_manager.cpp` - Order management tests
- `test_risk_calculator.cpp` - Risk calculation tests

### Python Tests (pytest/unittest)

**Location**: `python/tests/`, `python/integration/`
**Framework**: pytest, unittest
**Run**: `pytest python/tests/` or `python -m unittest discover`

**Test Files**:

- ✅ `python/tests/test_security.py` - Security tests (unittest)
- `python/tests/run_security_tests.py` - Security test runner
- `scripts/swiftness_integration_manual.py` – Manual Swiftness integration check

---

## Running Tests

### C++ Tests

```bash

# Build tests

cd native
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Debug -DBUILD_TESTING=ON
cmake --build .

# Run all tests

ctest --output-on-failure

# Run specific test

./box_spread_tests "[path_validator]"
```

### Python Tests

```bash

# Run all tests (using test runner script)

./scripts/run_python_tests.sh

# Run with coverage

./scripts/run_python_tests.sh --coverage

# Run with HTML coverage report

./scripts/run_python_tests.sh --html

# Or use pytest directly

pytest python/tests/ python/integration/

# Run specific test file

pytest python/tests/test_security.py

# Run with unittest

python python/tests/run_security_tests.py
```

---

## Coverage Generation Scripts

### Automated Coverage Generation

We've created automated scripts to generate coverage reports:

#### Python Coverage

```bash
# Generate Python coverage report
./scripts/generate_python_coverage.sh

# Generate with HTML report
./scripts/generate_python_coverage.sh --html

# Generate with XML report (for CI/CD)
./scripts/generate_python_coverage.sh --xml

# Generate all report formats
./scripts/generate_python_coverage.sh --all
```

#### C++ Coverage

```bash
# Generate C++ coverage report
./scripts/generate_cpp_coverage.sh

# Rebuild with coverage enabled
./scripts/generate_cpp_coverage.sh --rebuild
```

#### Combined Coverage

```bash
# Generate both C++ and Python coverage
./scripts/generate_coverage.sh

# Generate only Python coverage
./scripts/generate_coverage.sh --python-only

# Generate only C++ coverage
./scripts/generate_coverage.sh --cpp-only

# Generate with HTML reports
./scripts/generate_coverage.sh --html
```

### Coverage Configuration

**File**: `.coveragerc` (root directory)

**Source paths**: `python/services`, `python/tui`, `python/integration`

**Exclusions**: Tests, `__pycache__`, bindings, setup files

**Branch coverage**: Enabled

---

## Coverage Goals

### Phase 1: Baseline (Current)

- ✅ Set up coverage measurement tools
- ✅ Add tests for new security features (path_validator)
- 🔄 Measure current coverage baseline

### Phase 2: Critical Paths (Target: 20%)

- Add tests for core trading logic
- Add tests for API clients
- Add tests for configuration management

### Phase 3: Comprehensive (Target: 30%+)

- Add tests for edge cases
- Add integration tests
- Add performance tests

---

## Coverage Interpretation

### Understanding Coverage Reports

**Line Coverage**: Percentage of executable lines that were executed during tests.

**Branch Coverage**: Percentage of decision branches (if/else, loops, etc.) that were taken.

**Target Metrics**:

- **Overall Coverage**: 30%+ (project goal)
- **Critical Paths**: 50%+ (core trading logic, security, order management)
- **Security Module**: 30%+ (minimum requirement)

### Coverage Report Locations

- **Python HTML Report**: `htmlcov/index.html`
- **C++ HTML Report**: `native/build-coverage/coverage_html/index.html`
- **Python XML Report**: `coverage.xml` (for CI/CD integration)
- **C++ Coverage Data**: `native/build-coverage/coverage_filtered.info`

### Troubleshooting Coverage Issues

#### Low Coverage on Specific Files

1. **Check if file is excluded**: Review `.coveragerc` omit patterns
2. **Verify source paths**: Ensure file is in included source directories
3. **Check for test files**: Ensure corresponding test files exist
4. **Review uncovered lines**: Focus on critical business logic first

#### Coverage Not Generating

1. **Verify tools installed**: `pytest-cov` for Python, `lcov` for C++
2. **Check build flags**: C++ needs `--coverage` flags in CMake
3. **Verify test execution**: Tests must run to generate coverage data
4. **Check file paths**: Ensure coverage tools can find source files

#### Coverage Reports Not Updating

1. **Clear old reports**: Delete `htmlcov/` and rebuild
2. **Rebuild with coverage**: Use `--rebuild` flag for C++
3. **Verify test execution**: Ensure tests actually ran
4. **Check coverage data**: Verify `.coverage` (Python) or `coverage.info` (C++) exists

---

## Next Steps

1. ✅ **COMPLETE**: Test infrastructure setup
2. ✅ **COMPLETE**: Coverage configuration and automation scripts
3. ✅ **COMPLETE**: Test runner scripts created
4. 🔄 **IN PROGRESS**: Run tests to identify failures
5. 🔄 **PENDING**: Measure coverage baseline
6. 🔄 **PENDING**: Add tests to reach 30% coverage

---

## Quick Reference

### Common Commands

```bash
# Run Python tests with coverage
./scripts/run_python_tests.sh --coverage

# Generate all coverage reports
./scripts/generate_coverage.sh --html

# View Python coverage report
open htmlcov/index.html

# View C++ coverage report
open native/build-coverage/coverage_html/index.html
```

---

**Last Updated**: 2025-12-11
**Status**: ✅ Coverage automation complete, ready for baseline measurement
