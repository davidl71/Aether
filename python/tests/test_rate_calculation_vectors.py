"""
test_rate_calculation_vectors.py - Validate Python rate calculations against shared test vectors.

Loads the canonical test vectors from tests/shared/rate_calculation_vectors.json
and verifies that Python implementations produce matching results within tolerance.
"""

import json
import os
import pytest

VECTORS_PATH = os.path.join(
    os.path.dirname(__file__), "..", "..", "tests", "shared", "rate_calculation_vectors.json"
)


@pytest.fixture(scope="module")
def vectors():
    with open(VECTORS_PATH) as f:
        return json.load(f)


@pytest.fixture(scope="module")
def tolerance(vectors):
    return vectors["tolerance"]


def _within_tolerance(actual, expected, tol):
    if expected == 0.0:
        return abs(actual) <= tol["absolute"]
    return abs(actual - expected) <= max(tol["absolute"], abs(expected) * tol["relative_percent"] / 100.0)


class TestTheoreticalValue:
    def test_all_vectors(self, vectors, tolerance):
        for v in vectors["theoretical_value"]:
            strike_width = v["input"]["strike_high"] - v["input"]["strike_low"]
            assert _within_tolerance(strike_width, v["expected"], tolerance), (
                f'{v["id"]}: expected {v["expected"]}, got {strike_width}'
            )


class TestNetDebit:
    @staticmethod
    def _calc_net_debit(inp):
        return (
            inp["long_call_price"]
            - inp["short_call_price"]
            + inp["long_put_price"]
            - inp["short_put_price"]
        )

    def test_all_vectors(self, vectors, tolerance):
        for v in vectors["net_debit"]:
            result = self._calc_net_debit(v["input"])
            assert _within_tolerance(result, v["expected"], tolerance), (
                f'{v["id"]}: expected {v["expected"]}, got {result}'
            )


class TestMaxProfit:
    def test_all_vectors(self, vectors, tolerance):
        for v in vectors["max_profit"]:
            profit = v["input"]["theoretical_value"] - v["input"]["net_debit"]
            assert _within_tolerance(profit, v["expected"], tolerance), (
                f'{v["id"]}: expected {v["expected"]}, got {profit}'
            )


class TestROI:
    def test_all_vectors(self, vectors, tolerance):
        for v in vectors["roi"]:
            nd = v["input"]["net_debit"]
            tv = v["input"]["theoretical_value"]
            roi = ((tv - nd) / nd * 100.0) if nd > 0 else 0.0
            assert _within_tolerance(roi, v["expected_percent"], tolerance), (
                f'{v["id"]}: expected {v["expected_percent"]}, got {roi}'
            )


class TestImpliedInterestRate:
    @staticmethod
    def _calc_implied_rate(strike_width, net_debit, days_to_expiry):
        if days_to_expiry <= 0 or net_debit == 0.0:
            return 0.0
        if net_debit > 0:
            return ((net_debit - strike_width) / strike_width) * (365.0 / days_to_expiry) * 100.0
        else:
            net_credit = -net_debit
            return ((strike_width - net_credit) / net_credit) * (365.0 / days_to_expiry) * 100.0

    def test_all_vectors(self, vectors, tolerance):
        for v in vectors["implied_interest_rate"]:
            inp = v["input"]
            result = self._calc_implied_rate(
                inp["strike_width"], inp["net_debit"], inp["days_to_expiry"]
            )
            assert _within_tolerance(result, v["expected_annual_percent"], tolerance), (
                f'{v["id"]}: expected {v["expected_annual_percent"]}, got {result}'
            )


class TestCommission:
    def test_all_vectors(self, vectors, tolerance):
        for v in vectors["commission"]:
            commission = 4.0 * v["input"]["per_contract_fee"]
            assert _within_tolerance(commission, v["expected"], tolerance), (
                f'{v["id"]}: expected {v["expected"]}, got {commission}'
            )


class TestBuySellDisparity:
    def test_buy_net_debit(self, vectors, tolerance):
        for v in vectors["buy_sell_disparity"]:
            if "expected_buy_net_debit" in v:
                inp = v["input"]
                result = (
                    inp["long_call_ask"]
                    - inp["short_call_bid"]
                    + inp["long_put_ask"]
                    - inp["short_put_bid"]
                )
                assert _within_tolerance(result, v["expected_buy_net_debit"], tolerance), (
                    f'{v["id"]}: expected {v["expected_buy_net_debit"]}, got {result}'
                )

    def test_sell_net_credit(self, vectors, tolerance):
        for v in vectors["buy_sell_disparity"]:
            if "expected_sell_net_credit" in v:
                inp = v["input"]
                result = (
                    inp["long_call_bid"]
                    - inp["short_call_ask"]
                    + inp["long_put_bid"]
                    - inp["short_put_ask"]
                )
                assert _within_tolerance(result, v["expected_sell_net_credit"], tolerance), (
                    f'{v["id"]}: expected {v["expected_sell_net_credit"]}, got {result}'
                )
