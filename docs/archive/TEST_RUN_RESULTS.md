# Test Run Results

**Date**: 2025-12-25
**Status**: Historical; native C++ tests removed. Use Rust/nautilus for current results.

---

## Current test commands

- **Rust**: `cd agents/backend && cargo test`
- **Python (nautilus)**: `just test-nautilus` or `cd agents/nautilus && uv run pytest tests/ -v`
- **TUI E2E**: `just test-tui-e2e`
- **ShellSpec**: `./scripts/run_tests.sh`

---

## Historical summary (for reference)

The sections below describe results from when the repo had a native C++ test suite and Python tests under `python/`. Those C++ tests and paths are obsolete.

### Python tests (T-202)

- 65 tests passed; coverage was tracked under `python/`. Current Python tests live in `agents/nautilus/tests/`.

### C++ tests (T-213, T-214)

- Referenced `native/tests/test_market_hours.cpp`, `test_tws_client.cpp`, `test_tws_integration.cpp`, `test_box_spread_e2e.cpp`. The native build and these test files were removed. Equivalent coverage is in Rust crates and nautilus tests.

For up-to-date results, run the commands above and check CI or local output.
