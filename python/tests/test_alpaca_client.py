"""Tests for Alpaca client -- option chain, order placement, cancellation."""

import pytest
from unittest.mock import MagicMock, patch


def _make_client(session=None):
    with patch.dict("os.environ", {
        "ALPACA_API_KEY_ID": "test_key",
        "ALPACA_API_SECRET_KEY": "test_secret",
        "ALPACA_PAPER": "1",
    }, clear=False):
        from python.integration.alpaca_client import AlpacaClient
        return AlpacaClient(session=session or MagicMock())


class TestOptionChain:
    def test_get_option_contracts(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {
            "option_contracts": [
                {
                    "id": "c1",
                    "symbol": "SPY250321C00500000",
                    "underlying_symbol": "SPY",
                    "expiration_date": "2025-03-21",
                    "strike_price": "500.00",
                    "type": "call",
                    "status": "active",
                    "tradable": True,
                },
                {
                    "id": "c2",
                    "symbol": "SPY250321P00500000",
                    "underlying_symbol": "SPY",
                    "expiration_date": "2025-03-21",
                    "strike_price": "500.00",
                    "type": "put",
                    "status": "active",
                    "tradable": True,
                },
            ]
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = mock_resp

        client = _make_client(session=mock_session)
        contracts = client.get_option_contracts("SPY", expiration_date="2025-03-21")

        assert len(contracts) == 2
        assert contracts[0]["type"] == "call"
        assert contracts[1]["type"] == "put"

    def test_get_option_contracts_with_strike_filter(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {"option_contracts": [{"id": "c1"}]}
        mock_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = mock_resp

        client = _make_client(session=mock_session)
        contracts = client.get_option_contracts(
            "SPY", strike_price_gte=480.0, strike_price_lte=520.0
        )

        assert len(contracts) == 1
        call_args = mock_session.get.call_args
        params = call_args.kwargs.get("params") or call_args[1].get("params", {})
        assert params["strike_price_gte"] == "480.0"
        assert params["strike_price_lte"] == "520.0"

    def test_get_option_chain_groups_by_expiry(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {
            "option_contracts": [
                {"id": "c1", "expiration_date": "2025-03-21", "type": "call"},
                {"id": "c2", "expiration_date": "2025-03-21", "type": "put"},
                {"id": "c3", "expiration_date": "2025-04-18", "type": "call"},
            ]
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = mock_resp

        client = _make_client(session=mock_session)
        chain = client.get_option_chain("SPY")

        assert "2025-03-21" in chain
        assert "2025-04-18" in chain
        assert len(chain["2025-03-21"]) == 2
        assert len(chain["2025-04-18"]) == 1

    def test_get_option_contracts_http_error(self):
        import requests as req_lib
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 500
        mock_resp.raise_for_status.side_effect = req_lib.HTTPError("500")
        mock_session.get.return_value = mock_resp

        client = _make_client(session=mock_session)
        contracts = client.get_option_contracts("SPY")
        assert contracts == []


class TestOrderPlacement:
    def test_place_market_order(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {
            "id": "order-123",
            "symbol": "SPY",
            "qty": "10",
            "side": "buy",
            "type": "market",
            "status": "accepted",
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = mock_resp

        client = _make_client(session=mock_session)
        order = client.place_order("SPY", qty=10, side="buy")

        assert order is not None
        assert order["id"] == "order-123"
        assert order["status"] == "accepted"

    def test_place_limit_order(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {
            "id": "order-456",
            "type": "limit",
            "limit_price": "500.00",
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = mock_resp

        client = _make_client(session=mock_session)
        order = client.place_order(
            "SPY", qty=5, side="buy", order_type="limit", limit_price=500.00
        )

        assert order is not None
        call_args = mock_session.post.call_args
        body = call_args.kwargs.get("json") or call_args[1].get("json", {})
        assert body["limit_price"] == "500.0"

    def test_place_multi_leg_order(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {
            "id": "mleg-789",
            "order_class": "mleg",
            "status": "accepted",
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = mock_resp

        client = _make_client(session=mock_session)
        legs = [
            {"symbol": "SPY250321C00500000", "qty": "1", "side": "buy", "position_effect": "open"},
            {"symbol": "SPY250321P00500000", "qty": "1", "side": "sell", "position_effect": "open"},
        ]
        order = client.place_multi_leg_order(legs, limit_price=1.50)

        assert order is not None
        assert order["order_class"] == "mleg"

    def test_place_order_failure(self):
        import requests as req_lib
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 422
        mock_resp.raise_for_status.side_effect = req_lib.HTTPError("422")
        mock_session.post.return_value = mock_resp

        client = _make_client(session=mock_session)
        order = client.place_order("SPY", qty=10, side="buy")
        assert order is None


class TestOrderCancellation:
    def test_cancel_order_success(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 204
        mock_session.delete.return_value = mock_resp

        client = _make_client(session=mock_session)
        result = client.cancel_order("order-123")
        assert result is True

    def test_cancel_order_not_found(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 404
        mock_session.delete.return_value = mock_resp

        client = _make_client(session=mock_session)
        result = client.cancel_order("nonexistent")
        assert result is False

    def test_cancel_all_orders(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_session.delete.return_value = mock_resp

        client = _make_client(session=mock_session)
        result = client.cancel_all_orders()
        assert result is True


class TestCredentials:
    def test_missing_credentials_raises(self):
        with patch.dict("os.environ", {}, clear=True):
            with pytest.raises(RuntimeError, match="Missing Alpaca"):
                from python.integration.alpaca_client import AlpacaClient
                AlpacaClient()
