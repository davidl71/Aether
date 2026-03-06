"""Tests for combo_detector: parse option contractDesc and detect box spreads."""
from __future__ import annotations

import pytest
from integration.combo_detector import (
    parse_opt_contract_desc,
    legs_to_option_legs,
    detect_box_spreads,
)


class TestParseOptContractDesc:
    def test_parses_spx_call(self):
        desc = "SPX    MAR2027 6825 C [SPX   270319C06825000 100]"
        out = parse_opt_contract_desc(desc)
        assert out == ("SPX", "MAR2027", 6825.0, "C")

    def test_parses_spx_put(self):
        desc = "SPX    MAR2027 6950 P [SPX   270319P06950000 100]"
        out = parse_opt_contract_desc(desc)
        assert out == ("SPX", "MAR2027", 6950.0, "P")

    def test_returns_none_for_non_option(self):
        assert parse_opt_contract_desc("BND") is None
        assert parse_opt_contract_desc("") is None
        assert parse_opt_contract_desc("IBCID123") is None


class TestDetectBoxSpreads:
    def test_detects_one_box_four_legs(self):
        positions = [
            {"assetClass": "OPT", "conid": 1, "contractDesc": "SPX    MAR2027 6825 C [X]", "position": -1.0, "mktValue": -100.0, "unrealizedPnl": 0.0, "avgCost": 100.0},
            {"assetClass": "OPT", "conid": 2, "contractDesc": "SPX    MAR2027 6825 P [X]", "position": 1.0, "mktValue": 50.0, "unrealizedPnl": 0.0, "avgCost": 50.0},
            {"assetClass": "OPT", "conid": 3, "contractDesc": "SPX    MAR2027 6950 C [X]", "position": 1.0, "mktValue": 60.0, "unrealizedPnl": 0.0, "avgCost": 60.0},
            {"assetClass": "OPT", "conid": 4, "contractDesc": "SPX    MAR2027 6950 P [X]", "position": -1.0, "mktValue": -40.0, "unrealizedPnl": 0.0, "avgCost": 40.0},
        ]
        combos, remaining = detect_box_spreads(positions)
        assert len(combos) == 1
        assert combos[0]["type"] == "box_spread"
        assert combos[0]["underlying"] == "SPX"
        assert combos[0]["expiry"] == "MAR2027"
        assert combos[0]["k1"] == 6825.0
        assert combos[0]["k2"] == 6950.0
        assert combos[0]["quantity"] == 1
        assert len(remaining) == 0

    def test_non_box_options_remain_flat(self):
        positions = [
            {"assetClass": "OPT", "conid": 1, "contractDesc": "SPX    MAR2027 6825 C [X]", "position": 1.0, "mktValue": 100.0, "unrealizedPnl": 0.0, "avgCost": 100.0},
            {"assetClass": "STK", "conid": 99, "contractDesc": "BND", "position": 100.0, "mktValue": 5000.0, "unrealizedPnl": 0.0, "avgCost": 50.0},
        ]
        combos, remaining = detect_box_spreads(positions)
        assert len(combos) == 0
        assert len(remaining) == 2

    def test_stocks_and_bills_unchanged(self):
        positions = [
            {"assetClass": "STK", "conid": 1, "contractDesc": "BND", "position": 140.0, "mktValue": 10409.0, "unrealizedPnl": -62.2, "avgCost": 74.79},
            {"assetClass": "BILL", "conid": 2, "contractDesc": "IBCID123", "position": 4.0, "mktValue": 3973.0, "unrealizedPnl": 9.2, "avgCost": 99.1},
        ]
        combos, remaining = detect_box_spreads(positions)
        assert len(combos) == 0
        assert len(remaining) == 2
