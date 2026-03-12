# Python bindings for IB Box Spread C++ engine

Python extension that wraps the native box spread calculator, types, and validators. The active backend is **pybind11**.

## Backend: pybind11

- **pybind11** – Built by CMake with `pybind11_add_module`; single build system, better maintainability and performance. Default when `PYTHON_BINDINGS_BACKEND` is not set.
- **Cython** – Removed from the active repo layout. `PYTHON_BINDINGS_BACKEND=Cython` is no longer supported.

## Build (pybind11, recommended)

From repo root, configure with pybind11 and build the bindings target. Requires the same native deps as the CLI (TWS API, Intel decimal, etc.).

```bash
cmake --preset macos-arm64-debug -DPYTHON_BINDINGS_BACKEND=pybind11
cmake --build build/macos-arm64-debug --target box_spread_bindings
```

The extension is written to `build/<preset-or-build-dir>/python/bindings/`.

## Public API

- **Types:** `PyOptionContract`, `PyBoxSpreadLeg`, `PyMarketData`
- **Enums:** `OptionType`, `OptionTypePy` (alias), `OrderActionPy`, `OrderStatusPy`, `TimeInForcePy`
- **Functions:** `calculate_arbitrage_profit(spread)`, `calculate_roi(spread)`, `calculate_implied_interest_rate(spread)`, `validate_box_spread(spread)`
- **Class:** `BoxSpreadCalculator` (static methods: `calculate_max_profit`, `calculate_roi`, `calculate_implied_interest_rate`, etc.)

## Import / runtime

If you see **symbol not found** at import, the extension is missing symbols from other native units (e.g. TWS client). With pybind11, the CMake target links the same native sources and libs as the CLI, so a successful build should import cleanly.

## Tests

- `native/tests/python/test_bindings.py` – uses `OptionType`, `PyOptionContract`, `PyBoxSpreadLeg`, `calculate_arbitrage_profit`, `calculate_roi`, `calculate_implied_interest_rate`, `validate_box_spread`. Skips if the extension fails to import.
