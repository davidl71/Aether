"""
test_treasury_api_client.py - Tests for the Treasury Fiscal Data API client.

Uses mocked HTTP responses to avoid hitting the real API during testing.
"""

import pytest
from unittest.mock import patch, MagicMock
from datetime import datetime

from python.integration.treasury_api_client import (
    TreasuryAPIClient,
    TreasuryRate,
    TREASURY_API_BASE,
)


@pytest.fixture
def client():
    return TreasuryAPIClient(cache_duration=3600)


@pytest.fixture
def sample_api_response():
    return {
        "data": [
            {
                "record_date": "2026-02-25",
                "security_type_desc": "Treasury Bills",
                "security_term_desc": "3-Month",
                "avg_interest_rate_amt": "4.350",
            },
            {
                "record_date": "2026-02-25",
                "security_type_desc": "Treasury Bills",
                "security_term_desc": "6-Month",
                "avg_interest_rate_amt": "4.500",
            },
            {
                "record_date": "2026-02-25",
                "security_type_desc": "Treasury Notes",
                "security_term_desc": "1-Year",
                "avg_interest_rate_amt": "4.600",
            },
            {
                "record_date": "2026-02-25",
                "security_type_desc": "Treasury Notes",
                "security_term_desc": "2-Year",
                "avg_interest_rate_amt": "4.200",
            },
        ]
    }


class TestTreasuryRate:
    def test_valid_rate(self):
        rate = TreasuryRate(
            record_date="2026-02-25",
            security_type="Treasury Bills",
            security_term="3-Month",
            avg_interest_rate=4.35,
            timestamp=datetime.now(),
        )
        assert rate.is_valid()

    def test_invalid_zero_rate(self):
        rate = TreasuryRate(
            record_date="2026-02-25",
            security_type="Treasury Bills",
            security_term="3-Month",
            avg_interest_rate=0.0,
            timestamp=datetime.now(),
        )
        assert not rate.is_valid()

    def test_invalid_empty_date(self):
        rate = TreasuryRate(
            record_date="",
            security_type="Treasury Bills",
            security_term="3-Month",
            avg_interest_rate=4.35,
            timestamp=datetime.now(),
        )
        assert not rate.is_valid()

    def test_invalid_empty_term(self):
        rate = TreasuryRate(
            record_date="2026-02-25",
            security_type="Treasury Bills",
            security_term="",
            avg_interest_rate=4.35,
            timestamp=datetime.now(),
        )
        assert not rate.is_valid()


class TestTreasuryAPIClient:
    @patch("python.integration.treasury_api_client.requests.get")
    def test_fetch_average_interest_rates(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rates = client.fetch_average_interest_rates(use_cache=False)

        assert len(rates) == 4
        assert all(isinstance(r, TreasuryRate) for r in rates)
        assert rates[0].security_term == "3-Month"
        assert rates[0].avg_interest_rate == 4.35

    @patch("python.integration.treasury_api_client.requests.get")
    def test_caching(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rates1 = client.fetch_average_interest_rates(use_cache=True)
        rates2 = client.fetch_average_interest_rates(use_cache=True)

        assert mock_get.call_count == 1
        assert len(rates1) == len(rates2)

    @patch("python.integration.treasury_api_client.requests.get")
    def test_cache_bypass(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        client.fetch_average_interest_rates(use_cache=False)
        client.fetch_average_interest_rates(use_cache=False)

        assert mock_get.call_count == 2

    @patch("python.integration.treasury_api_client.requests.get")
    def test_get_latest_rate(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rate = client.get_latest_rate("3-Month")
        assert rate is not None
        assert rate.avg_interest_rate == 4.35
        assert "3-Month" in rate.security_term

    @patch("python.integration.treasury_api_client.requests.get")
    def test_get_latest_rate_not_found(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rate = client.get_latest_rate("30-Year")
        assert rate is None

    @patch("python.integration.treasury_api_client.requests.get")
    def test_get_rate_for_days(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rate = client.get_rate_for_days(90)
        assert rate is not None
        assert "3-Month" in rate.security_term

    @patch("python.integration.treasury_api_client.requests.get")
    def test_get_rate_for_days_short_tenor(self, mock_get, client):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [
                {
                    "record_date": "2026-02-25",
                    "security_type_desc": "Treasury Bills",
                    "security_term_desc": "1-Month",
                    "avg_interest_rate_amt": "4.200",
                }
            ]
        }
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rate = client.get_rate_for_days(15)
        assert rate is not None
        assert "1-Month" in rate.security_term

    @patch("python.integration.treasury_api_client.requests.get")
    def test_compare_to_box_spread_rate(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        result = client.compare_to_box_spread_rate(
            box_spread_rate=4.60,
            days_to_expiry=90,
        )

        assert result is not None
        assert result["box_spread_rate"] == 4.60
        assert "treasury_rate" in result
        assert "spread_bps" in result
        assert "beats_treasury" in result

    @patch("python.integration.treasury_api_client.requests.get")
    def test_compare_to_box_spread_rate_positive_spread(self, mock_get, client, sample_api_response):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        result = client.compare_to_box_spread_rate(
            box_spread_rate=5.00,
            days_to_expiry=90,
        )

        assert result is not None
        assert result["beats_treasury"] is True
        assert result["spread_bps"] > 0

    @patch("python.integration.treasury_api_client.requests.get")
    def test_api_failure_returns_empty(self, mock_get, client):
        import requests as req
        mock_get.side_effect = req.RequestException("Connection failed")

        rates = client.fetch_average_interest_rates(use_cache=False)
        assert rates == []

    @patch("python.integration.treasury_api_client.requests.get")
    def test_api_failure_returns_stale_cache(self, mock_get, client, sample_api_response):
        import requests as req

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = sample_api_response
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rates1 = client.fetch_average_interest_rates(use_cache=True)
        assert len(rates1) == 4

        mock_get.side_effect = req.RequestException("Connection failed")
        rates2 = client.fetch_average_interest_rates(use_cache=True)
        assert len(rates2) == 4

    @patch("python.integration.treasury_api_client.requests.get")
    def test_malformed_record_skipped(self, mock_get, client):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [
                {
                    "record_date": "2026-02-25",
                    "security_type_desc": "Treasury Bills",
                    "security_term_desc": "3-Month",
                    "avg_interest_rate_amt": "not_a_number",
                },
                {
                    "record_date": "2026-02-25",
                    "security_type_desc": "Treasury Bills",
                    "security_term_desc": "6-Month",
                    "avg_interest_rate_amt": "4.500",
                },
            ]
        }
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        rates = client.fetch_average_interest_rates(use_cache=False)
        assert len(rates) == 1
        assert rates[0].security_term == "6-Month"

    def test_rate_limit_interval(self, client):
        assert client._min_request_interval == 1.0

    def test_default_base_url(self, client):
        assert client.base_url == TREASURY_API_BASE

    def test_day_to_term_mapping(self, client):
        """Verify the days-to-term mapping covers expected ranges."""
        test_cases = [
            (30, "1-Month"),
            (90, "3-Month"),
            (180, "6-Month"),
            (365, "1-Year"),
            (500, "2-Year"),
            (900, "3-Year"),
            (1500, "5-Year"),
            (3000, "10-Year"),
        ]
        for days, expected_term in test_cases:
            with patch.object(client, "get_latest_rate", return_value=None) as mock:
                client.get_rate_for_days(days)
                call_args = mock.call_args
                assert expected_term in call_args[0][0], (
                    f"Days={days}: expected term containing '{expected_term}', "
                    f"got '{call_args[0][0]}'"
                )
