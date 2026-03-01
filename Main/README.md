# Main – LEAN / QuantConnect algorithms

This directory holds **LEAN (QuantConnect) algorithm** entrypoints. These are not pytest tests; they run inside the LEAN engine for backtesting or live trading.

- **test_box_spread_basic.py** – Basic test algorithm to verify LEAN integration (option chain subscription, data flow). Run via LEAN CLI; see `docs/research/integration/LEAN_TESTING.md` and `python/lean_integration/` for the full box spread LEAN integration.
