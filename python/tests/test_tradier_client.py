"""Tests for TradierClient."""

import pytest
from unittest.mock import MagicMock, patch
from python.integration.tradier_client import TradierClient, TradierError


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

def _make_client(session=None):
    """Create a TradierClient with a mocked session."""
    mock_session = session or MagicMock()
    mock_session.headers = {}
    return TradierClient(
        access_token="test-token",
        base_url="https://sandbox.tradier.com",
        session=mock_session,
    )


def _mock_response(json_data, status_code=200):
    resp = MagicMock()
    resp.status_code = status_code
    resp.ok = status_code < 400
    resp.json.return_value = json_data
    resp.raise_for_status.side_effect = None
    return resp


# ---------------------------------------------------------------------------
# Init
# ---------------------------------------------------------------------------

class TestInit:
    def test_init_with_token(self):
        client = _make_client()
        assert client.access_token == "test-token"
        assert client.base_url == "https://sandbox.tradier.com"

    def test_init_sandbox_env(self):
        with patch.dict("os.environ", {"TRADIER_ACCESS_TOKEN": "tok", "TRADIER_SANDBOX": "1"}):
            client = TradierClient(session=MagicMock())
            assert client.sandbox is True
            assert "sandbox" in client.base_url

    def test_init_missing_credentials(self):
        with patch.dict("os.environ", {}, clear=True):
            with pytest.raises(RuntimeError, match="Missing Tradier"):
                TradierClient()


# ---------------------------------------------------------------------------
# Quotes
# ---------------------------------------------------------------------------

class TestGetQuotes:
    def test_single_symbol(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({
            "quotes": {
                "quote": {
                    "symbol": "SPY",
                    "bid": 450.10,
                    "ask": 450.20,
                    "last": 450.15,
                    "volume": 100000,
                    "open": 449.0,
                    "high": 451.0,
                    "low": 448.5,
                    "close": 449.8,
                    "change": 0.35,
                    "change_percentage": 0.08,
                }
            }
        })
        client = _make_client(session)
        quotes = client.get_quotes(["SPY"])
        assert len(quotes) == 1
        assert quotes[0]["symbol"] == "SPY"
        assert quotes[0]["bid"] == 450.10
        assert quotes[0]["ask"] == 450.20
        assert quotes[0]["volume"] == 100000

    def test_multiple_symbols(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({
            "quotes": {
                "quote": [
                    {"symbol": "SPY", "bid": 450.0, "ask": 450.5, "last": 450.2,
                     "volume": 100, "open": 449, "high": 451, "low": 448,
                     "close": 449, "change": 1, "change_percentage": 0.2},
                    {"symbol": "QQQ", "bid": 380.0, "ask": 380.3, "last": 380.1,
                     "volume": 200, "open": 379, "high": 381, "low": 378,
                     "close": 379, "change": 1.1, "change_percentage": 0.3},
                ]
            }
        })
        client = _make_client(session)
        quotes = client.get_quotes(["SPY", "QQQ"])
        assert len(quotes) == 2
        assert quotes[1]["symbol"] == "QQQ"

    def test_empty_symbols(self):
        client = _make_client()
        assert client.get_quotes([]) == []

    def test_http_error(self):
        session = MagicMock()
        session.headers = {}
        resp = MagicMock()
        resp.raise_for_status.side_effect = Exception("HTTP 401")
        session.request.return_value = resp
        client = _make_client(session)
        with pytest.raises(Exception, match="401"):
            client.get_quotes(["SPY"])


# ---------------------------------------------------------------------------
# Snapshot
# ---------------------------------------------------------------------------

class TestGetSnapshot:
    def test_snapshot(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({
            "quotes": {
                "quote": {
                    "symbol": "SPY", "bid": 450.0, "ask": 450.5, "last": 450.2,
                    "volume": 5000, "open": 0, "high": 0, "low": 0,
                    "close": 0, "change": 0, "change_percentage": 0,
                }
            }
        })
        client = _make_client(session)
        snap = client.get_snapshot("SPY")
        assert snap["symbol"] == "SPY"
        assert snap["spread"] == pytest.approx(0.5)

    def test_snapshot_no_data(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({"quotes": {}})
        client = _make_client(session)
        snap = client.get_snapshot("UNKNOWN")
        assert snap["last"] == 0.0


# ---------------------------------------------------------------------------
# Option Expirations
# ---------------------------------------------------------------------------

class TestGetOptionExpirations:
    def test_expirations(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({
            "expirations": {
                "date": ["2025-12-19", "2025-11-21", "2026-01-16"]
            }
        })
        client = _make_client(session)
        exps = client.get_option_expirations("SPY")
        assert len(exps) == 3
        assert exps[0] == "2025-11-21"  # sorted

    def test_single_expiration(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({
            "expirations": {"date": "2025-12-19"}
        })
        client = _make_client(session)
        exps = client.get_option_expirations("SPY")
        assert exps == ["2025-12-19"]

    def test_empty_expirations(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({"expirations": {}})
        client = _make_client(session)
        assert client.get_option_expirations("SPY") == []


# ---------------------------------------------------------------------------
# Option Chain
# ---------------------------------------------------------------------------

class TestGetOptionChain:
    def test_option_chain(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({
            "options": {
                "option": [
                    {
                        "symbol": "SPY251219C00450000",
                        "underlying": "SPY",
                        "expiration_date": "2025-12-19",
                        "strike": 450.0,
                        "option_type": "call",
                        "bid": 12.5,
                        "ask": 12.8,
                        "last": 12.6,
                        "volume": 500,
                        "open_interest": 15000,
                        "greeks": {
                            "delta": 0.55,
                            "gamma": 0.02,
                            "theta": -0.05,
                            "vega": 0.18,
                            "mid_iv": 0.22,
                        },
                    },
                    {
                        "symbol": "SPY251219P00450000",
                        "underlying": "SPY",
                        "expiration_date": "2025-12-19",
                        "strike": 450.0,
                        "option_type": "put",
                        "bid": 11.0,
                        "ask": 11.3,
                        "last": 11.1,
                        "volume": 300,
                        "open_interest": 12000,
                        "greeks": {
                            "delta": -0.45,
                            "gamma": 0.02,
                            "theta": -0.04,
                            "vega": 0.17,
                            "mid_iv": 0.21,
                        },
                    },
                ]
            }
        })
        client = _make_client(session)
        chain = client.get_option_chain("SPY", expiration_date="2025-12-19")
        assert "2025-12-19" in chain
        assert len(chain["2025-12-19"]) == 2
        call = chain["2025-12-19"][0]
        assert call["option_type"] == "call"
        assert call["delta"] == pytest.approx(0.55)
        assert call["implied_volatility"] == pytest.approx(0.22)

    def test_option_chain_empty(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({"options": {}})
        client = _make_client(session)
        assert client.get_option_chain("SPY") == {}

    def test_option_chain_single(self):
        session = MagicMock()
        session.headers = {}
        session.request.return_value = _mock_response({
            "options": {
                "option": {
                    "symbol": "SPY251219C00450000",
                    "underlying": "SPY",
                    "expiration_date": "2025-12-19",
                    "strike": 450.0,
                    "option_type": "call",
                    "bid": 10.0,
                    "ask": 10.5,
                    "last": 10.2,
                    "volume": 100,
                    "open_interest": 5000,
                }
            }
        })
        client = _make_client(session)
        chain = client.get_option_chain("SPY")
        assert len(chain["2025-12-19"]) == 1
