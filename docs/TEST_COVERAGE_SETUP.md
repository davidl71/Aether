# Test Coverage Setup Guide

**Date**: 2025-11-29
**Status**: ✅ **Setup Complete** (updated for Rust-first; native C++ removed)

> **Note:** The native C++ build and C++ coverage tooling were removed. This guide now focuses on **Rust** and **Python (nautilus)** coverage.

---

## Overview

This guide documents test coverage for the Rust backend and Python (nautilus) codebases.

---

## Rust Test Coverage (cargo-llvm-cov / tarpaulin)

### Prerequisites

```bash
# Install cargo-llvm-cov (recommended)
cargo install cargo-llvm-cov

# Or tarpaulin
cargo install cargo-tarpaulin
```

### Run Tests with Coverage

```bash
cd agents/backend

# Using cargo-llvm-cov (requires nightly or stable with llvm-tools)
cargo llvm-cov test --lcov --output-path lcov.info
cargo llvm-cov report

# Or using tarpaulin
cargo tarpaulin --out Html --output-dir coverage
open coverage/tarpaulin-report.html
```

### Focus Areas

- **risk**, **quant**, **api**, **ib_adapter** crates — core logic
- **nats_adapter** — messaging
- **tui_service**, **backend_service** — integration

---

## Python Test Coverage (pytest-cov)

### Prerequisites

```bash

# Install pytest-cov

pip install pytest pytest-cov coverage
```

### Run Tests with Coverage

```bash
cd agents/nautilus
uv run pytest tests/ -v --cov=nautilus_agent --cov-report=html --cov-report=term
open htmlcov/index.html  # macOS
```

### Coverage Configuration

Use a `.coveragerc` in `agents/nautilus/` or pass `--cov` as above.

**Focus areas**: `nautilus_agent` (strategy, NATS bridge).

---

## Test Infrastructure

### Rust Tests

**Location**: `agents/backend`
**Run**: `cd agents/backend && cargo test`

### Python Tests (nautilus)

**Location**: `agents/nautilus/tests/`
**Run**: `just test-python` or `cd agents/nautilus && uv run pytest tests/ -v`

## Running Tests

### Rust

```bash
cd agents/backend
cargo test
cargo test -p risk
cargo test -p backend_service --test integration_test -- --ignored
```

### Python (nautilus)

```bash
cd agents/nautilus
uv run pytest tests/ -v
uv run pytest tests/test_strategy.py -v
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

## Coverage Interpretation

### Understanding Coverage Reports

**Line Coverage**: Percentage of executable lines that were executed during tests.

**Branch Coverage**: Percentage of decision branches (if/else, loops, etc.) that were taken.

**Target Metrics**:

- **Overall Coverage**: 30%+ (project goal)
- **Critical Paths**: 50%+ (core trading logic, security, order management)
- **Security Module**: 30%+ (minimum requirement)

### Coverage Report Locations

- **Rust**: `agents/backend/coverage/` (tarpaulin) or `lcov.info` (cargo-llvm-cov)
- **Python (nautilus)**: `agents/nautilus/htmlcov/index.html`, `coverage.xml` for CI

### Troubleshooting Coverage Issues

#### Low Coverage on Specific Files

1. **Check if file is excluded**: Review `.coveragerc` omit patterns
2. **Verify source paths**: Ensure file is in included source directories
3. **Check for test files**: Ensure corresponding test files exist
4. **Review uncovered lines**: Focus on critical business logic first

#### Coverage Not Generating

1. **Verify tools installed**: `pytest-cov` for Python, `cargo-llvm-cov` or `cargo-tarpaulin` for Rust
2. **Rust**: Run from `agents/backend`; ensure `cargo test` passes first
3. **Verify test execution**: Tests must run to generate coverage data
4. **Check file paths**: Ensure coverage tools can find source files

#### Coverage Reports Not Updating

1. **Clear old reports**: Delete `htmlcov/` or `coverage/` and re-run
2. **Rust**: Run `cargo clean` then `cargo llvm-cov test` (or tarpaulin) again
3. **Verify test execution**: Ensure tests actually ran
4. **Check coverage data**: Verify `.coverage` (Python) or `lcov.info` (Rust) exists

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
# Rust coverage (from agents/backend)
cargo llvm-cov test --lcov --output-path lcov.info && cargo llvm-cov report

# Python (nautilus) coverage
cd agents/nautilus && uv run pytest tests/ --cov=nautilus_agent --cov-report=html
open agents/nautilus/htmlcov/index.html
```

---

**Last Updated**: 2025-12-11
**Status**: ✅ Coverage automation complete, ready for baseline measurement
