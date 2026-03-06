"""
test_alpha_vantage_client.py - Tests for the Alpha Vantage API client.

Uses mocked HTTP responses to avoid hitting Alpha Vantage during testing.
"""

import pytest
from unittest.mock import patch, MagicMock

from python.integration.alpha_vantage_client import AlphaVantageClient


def _mock_quote_response(price="180.50", symbol="IBM"):
    """Build a mock GLOBAL_QUOTE response."""
    return {
        "Global Quote": {
            "01. symbol": symbol,
            "05. price": price,
            "09. change": "1.25",
            "10. change percent": "0.70%",
            "06. volume": "4500000",
        }
    }


@pytest.fixture
def mock_session():
    with patch("python.integration.alpha_vantage_client.requests.Session") as mock_cls:
        session = MagicMock()
        mock_cls.return_value = session
        yield session


@pytest.fixture
def client_with_key(mock_session):
    return AlphaVantageClient(api_key="test_key_123")


@pytest.fixture
def client_no_key(mock_session):
    with patch("python.integration.alpha_vantage_client.getenv_or_resolve", return_value=""):
        with patch("python.integration.alpha_vantage_client.get_alpha_vantage_api_key_from_1password", return_value=None):
            return AlphaVantageClient(api_key=None)


class TestAlphaVantageClientInit:
    def test_init_with_key(self, client_with_key):
        assert client_with_key.api_key == "test_key_123"

    def test_init_no_key(self, client_no_key):
        assert client_no_key.api_key == ""


class TestGetQuote:
    def test_get_quote_success(self, client_with_key, mock_session):
        mock_session.get.return_value = MagicMock(
            status_code=200,
            json=lambda: _mock_quote_response("185.00", "SPY"),
        )
        quote = client_with_key.get_quote("SPY")
        assert quote is not None
        assert quote.get("01. symbol") == "SPY"
        assert quote.get("05. price") == "185.00"

    def test_get_quote_no_key_returns_none(self, client_no_key, mock_session):
        # No key -> _get returns {} -> get_quote returns None
        quote = client_no_key.get_quote("IBM")
        assert quote is None

    def test_get_quote_api_error_message(self, client_with_key, mock_session):
        mock_session.get.return_value = MagicMock(
            status_code=200,
            json=lambda: {"Error Message": "Invalid API key"},
        )
        quote = client_with_key.get_quote("IBM")
        assert quote is None


class TestGetDaily:
    def test_get_daily_no_key_returns_none(self, client_no_key):
        result = client_no_key.get_daily("SPY")
        assert result is None


class TestSearch:
    def test_search_no_key_returns_empty(self, client_no_key):
        result = client_no_key.search("apple")
        assert result == []
