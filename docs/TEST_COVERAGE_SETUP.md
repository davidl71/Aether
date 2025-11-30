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

Create `python/.coveragerc`:

```ini
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
```

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
- `python/integration/test_swiftness_integration.py` - Integration tests

---

## Running Tests

### C++ Tests

```bash

# Build tests

cd native
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Debug
cmake --build .

# Run all tests

ctest --output-on-failure

# Run specific test

./box_spread_tests "[path_validator]"
```

### Python Tests

```bash

# Run all tests

pytest python/tests/ python/integration/

# Run specific test file

pytest python/tests/test_security.py

# Run with unittest

python python/tests/run_security_tests.py
```

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

## Next Steps

1. ✅ **COMPLETE**: Test infrastructure setup
2. ✅ **COMPLETE**: Added path_validator tests
3. 🔄 **IN PROGRESS**: Run tests to identify failures
4. 🔄 **PENDING**: Measure coverage baseline
5. 🔄 **PENDING**: Add tests to reach 30% coverage

---

**Last Updated**: 2025-11-29
**Status**: Test infrastructure ready, coverage measurement setup complete
