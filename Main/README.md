# Main – LEAN / QuantConnect algorithms

This directory holds **LEAN (QuantConnect) algorithm** entrypoints. These are not pytest tests; they run inside the LEAN engine for backtesting or live trading.

- **test_box_spread_basic.py** – Basic test algorithm to verify LEAN integration (option chain subscription, data flow). Run via LEAN CLI; see `docs/archive/LEAN_TESTING.md` for the historical box spread LEAN integration notes. The old top-level `python/lean_integration/` tree is not part of the current repo layout.
