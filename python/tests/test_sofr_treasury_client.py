"""
test_sofr_treasury_client.py - Tests for the SOFR/Treasury benchmark rate client.

Uses mocked HTTP responses to avoid hitting FRED/NY Fed APIs during testing.
"""

import pytest
from unittest.mock import patch, MagicMock
from datetime import datetime

from python.integration.sofr_treasury_client import (
    SOFRTreasuryClient,
    BenchmarkRate,
    RateComparison,
)


def _mock_fred_response(value, date="2026-02-25"):
    """Helper to build a mock FRED API response."""
    resp = MagicMock()
    resp.status_code = 200
    resp.json.return_value = {
        "observations": [{"date": date, "value": str(value)}]
    }
    return resp


@pytest.fixture
def mock_session():
    with patch("python.integration.sofr_treasury_client.requests.Session") as mock_cls:
        session = MagicMock()
        mock_cls.return_value = session
        yield session


@pytest.fixture
def client_with_key(mock_session):
    return SOFRTreasuryClient(fred_api_key="test_key_123")


@pytest.fixture
def client_no_key(mock_session):
    return SOFRTreasuryClient(fred_api_key=None)


class TestBenchmarkRate:
    def test_dataclass_fields(self):
        rate = BenchmarkRate(
            rate_type="SOFR",
            tenor="Overnight",
            days_to_expiry=1,
            rate=4.32,
            timestamp=datetime.now(),
            source="FRED",
        )
        assert rate.rate_type == "SOFR"
        assert rate.tenor == "Overnight"
        assert rate.days_to_expiry == 1
        assert rate.rate == 4.32

    def test_optional_metadata(self):
        rate = BenchmarkRate(
            rate_type="Treasury",
            tenor="3M",
            days_to_expiry=90,
            rate=4.35,
            timestamp=datetime.now(),
            source="FRED",
            metadata={"series_id": "DGS3MO"},
        )
        assert rate.metadata["series_id"] == "DGS3MO"


class TestSOFROvernightRate:
    def test_sofr_overnight_from_fred(self, client_with_key, mock_session):
        mock_session.get.return_value = _mock_fred_response(4.32)

        rate = client_with_key.get_sofr_overnight()

        assert rate is not None
        assert rate.rate_type == "SOFR"
        assert rate.tenor == "Overnight"
        assert rate.rate == 4.32
        assert rate.days_to_expiry == 1
        assert "FRED" in rate.source

    def test_sofr_overnight_zero_rate_skipped(self, client_with_key, mock_session):
        fred_resp = MagicMock()
        fred_resp.status_code = 200
        fred_resp.json.return_value = {
            "observations": [{"date": "2026-02-25", "value": "0"}]
        }
        nyfed_resp = MagicMock()
        nyfed_resp.status_code = 200
        nyfed_resp.json.return_value = {}
        mock_session.get.side_effect = [fred_resp, nyfed_resp]

        rate = client_with_key.get_sofr_overnight()
        assert rate is None

    def test_sofr_overnight_no_api_key_tries_nyfed(self, client_no_key, mock_session):
        resp = MagicMock()
        resp.status_code = 200
        resp.json.return_value = {"sofr": {"rate": 4.31}}
        mock_session.get.return_value = resp

        rate = client_no_key.get_sofr_overnight()

        assert rate is not None
        assert rate.rate == 4.31
        assert rate.source == "New York Fed"

    def test_sofr_overnight_all_sources_fail(self, client_no_key, mock_session):
        mock_session.get.side_effect = Exception("Network error")

        rate = client_no_key.get_sofr_overnight()
        assert rate is None


class TestSOFRTermRates:
    def test_sofr_term_rates(self, client_with_key, mock_session):
        mock_session.get.side_effect = [
            _mock_fred_response(4.30),  # 1M
            _mock_fred_response(4.35),  # 3M
            _mock_fred_response(4.40),  # 6M
            _mock_fred_response(4.45),  # 1Y
        ]

        rates = client_with_key.get_sofr_term_rates()

        assert len(rates) == 4
        assert all(r.rate_type == "SOFR" for r in rates)
        tenors = [r.tenor for r in rates]
        assert "1M" in tenors
        assert "3M" in tenors
        assert "6M" in tenors
        assert "1Y" in tenors

    def test_sofr_term_rates_partial_failure(self, client_with_key, mock_session):
        mock_session.get.side_effect = [
            _mock_fred_response(4.30),  # 1M succeeds
            Exception("API timeout"),   # 3M fails
            _mock_fred_response(4.40),  # 6M succeeds
            _mock_fred_response(4.45),  # 1Y succeeds
        ]

        rates = client_with_key.get_sofr_term_rates()
        assert len(rates) == 3

    def test_sofr_term_rates_no_api_key(self, client_no_key):
        rates = client_no_key.get_sofr_term_rates()
        assert rates == []


class TestTreasuryRates:
    def test_treasury_rates(self, client_with_key, mock_session):
        responses = [_mock_fred_response(rate) for rate in [4.1, 4.3, 4.5, 4.6, 4.2, 4.0, 3.9, 3.8]]
        mock_session.get.side_effect = responses

        rates = client_with_key.get_treasury_rates()

        assert len(rates) == 8
        assert all(r.rate_type == "Treasury" for r in rates)
        tenors = [r.tenor for r in rates]
        assert "1M" in tenors
        assert "10Y" in tenors
        assert "30Y" in tenors

    def test_treasury_rates_missing_data_skipped(self, client_with_key, mock_session):
        """FRED uses '.' for missing data points."""
        good = _mock_fred_response(4.3)
        missing = MagicMock()
        missing.status_code = 200
        missing.json.return_value = {
            "observations": [{"date": "2026-02-25", "value": "."}]
        }
        responses = [good, missing, good, good, good, good, good, good]
        mock_session.get.side_effect = responses

        rates = client_with_key.get_treasury_rates()
        assert len(rates) == 7

    def test_treasury_rates_no_api_key(self, client_no_key):
        rates = client_no_key.get_treasury_rates()
        assert rates == []


class TestBenchmarkAtDTE:
    @patch.object(SOFRTreasuryClient, "get_sofr_overnight")
    @patch.object(SOFRTreasuryClient, "get_sofr_term_rates")
    @patch.object(SOFRTreasuryClient, "get_treasury_rates")
    def test_find_closest_benchmark(self, mock_treas, mock_term, mock_ovn, mock_session):
        mock_ovn.return_value = BenchmarkRate(
            rate_type="SOFR", tenor="Overnight", days_to_expiry=1,
            rate=4.32, timestamp=datetime.now(), source="FRED"
        )
        mock_term.return_value = [
            BenchmarkRate(rate_type="SOFR", tenor="3M", days_to_expiry=90,
                          rate=4.35, timestamp=datetime.now(), source="FRED"),
        ]
        mock_treas.return_value = [
            BenchmarkRate(rate_type="Treasury", tenor="6M", days_to_expiry=180,
                          rate=4.50, timestamp=datetime.now(), source="FRED"),
        ]

        client = SOFRTreasuryClient(fred_api_key="test")
        result = client.get_benchmark_at_dte(88, tolerance=5)

        assert result is not None
        assert result.tenor == "3M"
        assert result.rate == 4.35

    @patch.object(SOFRTreasuryClient, "get_sofr_overnight")
    @patch.object(SOFRTreasuryClient, "get_sofr_term_rates")
    @patch.object(SOFRTreasuryClient, "get_treasury_rates")
    def test_no_match_within_tolerance(self, mock_treas, mock_term, mock_ovn, mock_session):
        mock_ovn.return_value = None
        mock_term.return_value = []
        mock_treas.return_value = [
            BenchmarkRate(rate_type="Treasury", tenor="10Y", days_to_expiry=3650,
                          rate=3.90, timestamp=datetime.now(), source="FRED"),
        ]

        client = SOFRTreasuryClient(fred_api_key="test")
        result = client.get_benchmark_at_dte(90, tolerance=5)
        assert result is None


class TestRateComparison:
    def test_calculate_spread(self):
        spread = RateComparison.calculate_spread(4.60, 4.35)
        assert abs(spread - 25.0) < 0.01

    def test_calculate_spread_negative(self):
        spread = RateComparison.calculate_spread(4.00, 4.50)
        assert spread < 0

    def test_calculate_spread_zero(self):
        spread = RateComparison.calculate_spread(4.35, 4.35)
        assert abs(spread) < 0.01
