# Python bindings for IB Box Spread C++ engine

Python extension that wraps the native box spread calculator, types, and validators. Same API whether built with **pybind11** (default) or **Cython**.

## Backend: pybind11 (default) or Cython

- **pybind11** – Built by CMake with `pybind11_add_module`; single build system, better maintainability and performance. Default when `PYTHON_BINDINGS_BACKEND` is not set.
- **Cython** – Legacy path via `setup.py`; requires a prior CMake build for deps. Set `-DPYTHON_BINDINGS_BACKEND=Cython` and run `cmake --build build --target python_bindings`.

## Build (pybind11, recommended)

From repo root, configure with pybind11 and build the bindings target. Requires the same native deps as the CLI (TWS API, Intel decimal, etc.).

```bash
cmake --preset macos-arm64-debug -DPYTHON_BINDINGS_BACKEND=pybind11
cmake --build build/macos-arm64-debug --target box_spread_bindings
```

The `.so` is written to `python/bindings/` so that `from box_spread_bindings import ...` works when run from the repo with `python/bindings` on `PYTHONPATH` or when installed.

## Build (Cython)

Run a full CMake build first so FetchContent populates `build/_deps` and `build/lib`, then:

```bash
cd python/bindings
uv run python setup.py build_ext --inplace
```

Or from repo root: `cmake --build build --target python_bindings` (when `PYTHON_BINDINGS_BACKEND=Cython` and Cython is installed).

## Public API

- **Types:** `PyOptionContract`, `PyBoxSpreadLeg`, `PyMarketData`
- **Enums:** `OptionType`, `OptionTypePy` (alias), `OrderActionPy`, `OrderStatusPy`, `TimeInForcePy`
- **Functions:** `calculate_arbitrage_profit(spread)`, `calculate_roi(spread)`, `calculate_implied_interest_rate(spread)`, `validate_box_spread(spread)`
- **Class:** `BoxSpreadCalculator` (static methods: `calculate_max_profit`, `calculate_roi`, `calculate_implied_interest_rate`, etc.)

## Import / runtime

If you see **symbol not found** at import, the extension is missing symbols from other native units (e.g. TWS client). With pybind11, the CMake target links the same native sources and libs as the CLI, so a successful build should import cleanly.

## Tests

- `native/tests/python/test_bindings.py` – uses `OptionType`, `PyOptionContract`, `PyBoxSpreadLeg`, `calculate_arbitrage_profit`, `calculate_roi`, `calculate_implied_interest_rate`, `validate_box_spread`. Skips if the extension fails to import.
