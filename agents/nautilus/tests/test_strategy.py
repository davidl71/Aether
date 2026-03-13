"""Tests for BoxSpreadStrategy — box spread detection logic.

Tests _find_box_spreads() in isolation, without NautilusTrader runtime or IB.
Mirrors the logic from C++ BoxSpreadStrategy::find_box_spreads_in_chain()
(native/src/strategies/box_spread/box_spread_strategy.cpp:213-248).
"""

from __future__ import annotations

import sys
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock

import pytest

sys.path.insert(0, str(Path(__file__).parents[1] / "src"))

# strategy imports nautilus_trader types; skip if not installed
try:
    from nautilus_agent.strategy import (
        BoxSpreadConfig,
        BoxSpreadOpportunity,
        BoxSpreadStrategy,
        _OptionQuote,
    )
    from nautilus_trader.model.identifiers import InstrumentId
    HAS_NT = True
except ImportError:
    HAS_NT = False

pytestmark = pytest.mark.skipif(
    not HAS_NT,
    reason="nautilus_trader not installed — run: just nautilus-sync",
)


def _make_instrument_id(symbol: str) -> "InstrumentId":
    return InstrumentId.from_str(f"{symbol}.CBOE")


def _make_quote(
    symbol: str,
    expiry: str,
    strike: float,
    option_type: str,  # "C" or "P"
    bid: float,
    ask: float,
) -> "_OptionQuote":
    inst_id = _make_instrument_id(f"{symbol}{expiry[2:]}{option_type}{int(strike * 1000):08d}")
    return _OptionQuote(
        instrument_id=inst_id,
        expiry=expiry,
        strike=strike,
        option_type=option_type,
        bid=bid,
        ask=ask,
    )


def _make_strategy(
    min_arbitrage_profit: float = 0.10,
    min_roi_percent: float = 0.5,
    max_bid_ask_spread: float = 0.10,
    max_position_size: float = 10_000.0,
) -> "BoxSpreadStrategy":
    cfg = BoxSpreadConfig(
        strategy_id="TEST-001",
        symbols=("SPX",),
        min_arbitrage_profit=min_arbitrage_profit,
        min_roi_percent=min_roi_percent,
        max_bid_ask_spread=max_bid_ask_spread,
        max_position_size=max_position_size,
    )
    mock_bridge = MagicMock()
    mock_bridge.schedule = MagicMock()
    # Strategy.__init__ calls super().__init__ which requires NT runtime; patch it
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(
            "nautilus_trader.trading.strategy.Strategy.__init__",
            lambda self, config: None,
        )
        strategy = BoxSpreadStrategy.__new__(BoxSpreadStrategy)
        strategy._nats = mock_bridge
        strategy._cfg = cfg
        strategy._chain_cache = {"SPX": {}}
        strategy._last_eval = {"SPX": 0.0}
    return strategy


# ------------------------------------------------------------------
# _find_box_spreads tests
# ------------------------------------------------------------------

def test_find_box_spreads_detects_profitable_opportunity():
    """Detects box spread when net_debit < theoretical_value."""
    strategy = _make_strategy()
    # Strike width = 100; theoretical value = 100
    # Net debit = 99.50 → profit = 0.50
    chain = {}
    for q in [
        _make_quote("SPX", "20241220", 5000.0, "C", bid=150.0, ask=150.50),  # long call
        _make_quote("SPX", "20241220", 5100.0, "C", bid=50.20, ask=50.50),   # short call
        _make_quote("SPX", "20241220", 5100.0, "P", bid=100.10, ask=100.50), # long put
        _make_quote("SPX", "20241220", 5000.0, "P", bid=1.50, ask=1.80),     # short put
    ]:
        chain[q.instrument_id.value] = q

    opps = strategy._find_box_spreads("SPX", chain)
    assert len(opps) >= 1
    opp = opps[0]
    assert opp.lower_strike == 5000.0
    assert opp.upper_strike == 5100.0
    assert opp.theoretical_value == pytest.approx(100.0)
    assert opp.arbitrage_profit > 0


def test_find_box_spreads_no_opportunity_when_net_debit_exceeds_value():
    """No opportunity when net_debit >= theoretical_value."""
    strategy = _make_strategy()
    chain = {}
    # Set prices so net_debit = strike_width (no profit)
    for q in [
        _make_quote("SPX", "20241220", 5000.0, "C", bid=200.0, ask=200.50),
        _make_quote("SPX", "20241220", 5100.0, "C", bid=99.0, ask=100.00),  # short: receive bid
        _make_quote("SPX", "20241220", 5100.0, "P", bid=0.10, ask=0.20),
        _make_quote("SPX", "20241220", 5000.0, "P", bid=99.0, ask=100.00),  # short: receive bid
    ]:
        chain[q.instrument_id.value] = q

    opps = strategy._find_box_spreads("SPX", chain)
    profitable = [o for o in opps if o.arbitrage_profit > 0]
    assert len(profitable) == 0


def test_find_box_spreads_filters_wide_bid_ask():
    """Spread with wide bid-ask on any leg is excluded."""
    strategy = _make_strategy(max_bid_ask_spread=0.05)  # tight 5% spread filter
    chain = {}
    for q in [
        _make_quote("SPX", "20241220", 5000.0, "C", bid=100.0, ask=120.0),  # 18% spread → rejected
        _make_quote("SPX", "20241220", 5100.0, "C", bid=50.0, ask=51.0),
        _make_quote("SPX", "20241220", 5100.0, "P", bid=50.0, ask=51.0),
        _make_quote("SPX", "20241220", 5000.0, "P", bid=1.0, ask=1.02),
    ]:
        chain[q.instrument_id.value] = q

    opps = strategy._find_box_spreads("SPX", chain)
    assert len(opps) == 0


def test_is_actionable_profit_filter():
    """_is_actionable rejects opps below min_arbitrage_profit."""
    strategy = _make_strategy(min_arbitrage_profit=1.00)
    chain = {}
    for q in [
        _make_quote("SPX", "20241220", 5000.0, "C", bid=150.0, ask=150.20),
        _make_quote("SPX", "20241220", 5100.0, "C", bid=50.20, ask=50.40),
        _make_quote("SPX", "20241220", 5100.0, "P", bid=100.10, ask=100.30),
        _make_quote("SPX", "20241220", 5000.0, "P", bid=1.50, ask=1.70),
    ]:
        chain[q.instrument_id.value] = q

    opps = strategy._find_box_spreads("SPX", chain)
    actionable = [o for o in opps if strategy._is_actionable(o)]
    # With min_profit=1.00, marginal opportunity should be filtered
    for opp in actionable:
        assert opp.arbitrage_profit >= 1.00


def test_find_box_spreads_multiple_expiries_independent():
    """Box spreads at different expiries are found independently."""
    strategy = _make_strategy()
    chain = {}
    for expiry in ("20241220", "20250117"):
        for q in [
            _make_quote("SPX", expiry, 5000.0, "C", bid=150.0, ask=150.50),
            _make_quote("SPX", expiry, 5100.0, "C", bid=50.20, ask=50.50),
            _make_quote("SPX", expiry, 5100.0, "P", bid=100.10, ask=100.50),
            _make_quote("SPX", expiry, 5000.0, "P", bid=1.50, ask=1.80),
        ]:
            chain[q.instrument_id.value] = q

    opps = strategy._find_box_spreads("SPX", chain)
    expiries = {o.expiry for o in opps if o.arbitrage_profit > 0}
    assert len(expiries) == 2


def test_option_quote_mid_and_spread_fraction():
    """_OptionQuote computes mid and spread_fraction correctly."""
    q = _OptionQuote(
        instrument_id=_make_instrument_id("SPX"),
        expiry="20241220",
        strike=5000.0,
        option_type="C",
        bid=100.0,
        ask=102.0,
    )
    assert q.mid == pytest.approx(101.0)
    assert q.spread_fraction == pytest.approx(2.0 / 101.0)
