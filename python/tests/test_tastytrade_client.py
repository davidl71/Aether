"""Tests for TastyTrade client -- option chain, orders, positions."""

import pytest
from unittest.mock import MagicMock, patch


def _make_client(session=None):
    with patch.dict("os.environ", {
        "TASTYTRADE_USERNAME": "testuser",
        "TASTYTRADE_PASSWORD": "testpass",
        "TASTYTRADE_SANDBOX": "1",
    }, clear=False):
        from python.integration.tastytrade_client import TastytradeClient
        return TastytradeClient(session=session or MagicMock())


def _mock_login(mock_session):
    """Pre-configure session mock so login() succeeds."""
    login_resp = MagicMock()
    login_resp.status_code = 200
    login_resp.json.return_value = {
        "data": {
            "user": {"username": "testuser"},
            "session-token": "tok-123",
        }
    }
    login_resp.raise_for_status = MagicMock()
    login_resp.headers = {"session-token": "tok-123"}
    login_resp.cookies = {}
    mock_session.post.return_value = login_resp


class TestOptionChain:
    def test_get_option_chain(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        chain_resp = MagicMock()
        chain_resp.status_code = 200
        chain_resp.json.return_value = {
            "data": {
                "items": [
                    {
                        "underlying-symbol": "SPY",
                        "expirations": [
                            {
                                "expiration-date": "2025-03-21",
                                "strikes": [
                                    {"strike-price": "500.0", "call": "SPY 03/21 500C"},
                                    {"strike-price": "500.0", "put": "SPY 03/21 500P"},
                                ],
                            },
                            {
                                "expiration-date": "2025-04-18",
                                "strikes": [
                                    {"strike-price": "510.0", "call": "SPY 04/18 510C"},
                                ],
                            },
                        ],
                    }
                ]
            }
        }
        chain_resp.raise_for_status = MagicMock()

        # First call = login POST, subsequent GET = chain
        mock_session.post.return_value = MagicMock(
            status_code=200,
            json=MagicMock(return_value={
                "data": {"user": {"username": "testuser"}, "session-token": "tok"},
            }),
            raise_for_status=MagicMock(),
            headers={"session-token": "tok"},
            cookies={},
        )
        mock_session.get.return_value = chain_resp

        client = _make_client(session=mock_session)
        chain = client.get_option_chain("SPY")

        assert "2025-03-21" in chain
        assert "2025-04-18" in chain
        assert len(chain["2025-03-21"]) == 2
        assert len(chain["2025-04-18"]) == 1

    def test_get_option_expirations(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        chain_resp = MagicMock()
        chain_resp.status_code = 200
        chain_resp.json.return_value = {
            "data": {
                "items": [
                    {
                        "expirations": [
                            {"expiration-date": "2025-04-18", "strikes": [{}]},
                            {"expiration-date": "2025-03-21", "strikes": [{}]},
                        ]
                    }
                ]
            }
        }
        chain_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = chain_resp

        client = _make_client(session=mock_session)
        exps = client.get_option_expirations("SPY")

        assert exps == ["2025-03-21", "2025-04-18"]

    def test_get_option_chain_error(self):
        import requests as req_lib
        mock_session = MagicMock()
        _mock_login(mock_session)

        err_resp = MagicMock()
        err_resp.status_code = 500
        err_resp.raise_for_status.side_effect = req_lib.HTTPError("500")
        mock_session.get.return_value = err_resp

        client = _make_client(session=mock_session)
        chain = client.get_option_chain("BAD")
        assert chain == {}


class TestOrders:
    def test_get_orders(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        orders_resp = MagicMock()
        orders_resp.status_code = 200
        orders_resp.json.return_value = {
            "data": {
                "items": [
                    {"id": "o1", "status": "Filled"},
                    {"id": "o2", "status": "Received"},
                ]
            }
        }
        orders_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = orders_resp

        client = _make_client(session=mock_session)
        orders = client.get_orders("ACCT123")

        assert len(orders) == 2
        assert orders[0]["id"] == "o1"

    def test_place_order_limit(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        order_resp = MagicMock()
        order_resp.status_code = 200
        order_resp.json.return_value = {
            "data": {
                "id": "new-order-1",
                "order-type": "Limit",
                "status": "Received",
            }
        }
        order_resp.raise_for_status = MagicMock()
        # POST calls: first = login, second = place order
        mock_session.post.side_effect = [
            MagicMock(
                status_code=200,
                json=MagicMock(return_value={
                    "data": {"user": {"username": "testuser"}, "session-token": "tok"},
                }),
                raise_for_status=MagicMock(),
                headers={"session-token": "tok"},
                cookies={},
            ),
            order_resp,
        ]

        client = _make_client(session=mock_session)
        legs = [
            {
                "instrument-type": "Equity Option",
                "action": "Buy to Open",
                "symbol": "SPY 250321C00500000",
                "quantity": 1,
            }
        ]
        result = client.place_order(
            "ACCT123", "Limit", "Day", legs, price=2.50
        )

        assert result is not None
        assert result["id"] == "new-order-1"

    def test_place_order_failure(self):
        import requests as req_lib
        mock_session = MagicMock()
        _mock_login(mock_session)

        err_resp = MagicMock()
        err_resp.status_code = 422
        err_resp.raise_for_status.side_effect = req_lib.HTTPError("422")
        mock_session.post.side_effect = [
            MagicMock(
                status_code=200,
                json=MagicMock(return_value={
                    "data": {"user": {"username": "testuser"}, "session-token": "tok"},
                }),
                raise_for_status=MagicMock(),
                headers={"session-token": "tok"},
                cookies={},
            ),
            err_resp,
        ]

        client = _make_client(session=mock_session)
        result = client.place_order("ACCT123", "Market", "Day", [])
        assert result is None

    def test_cancel_order_success(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        del_resp = MagicMock()
        del_resp.status_code = 204
        mock_session.delete.return_value = del_resp

        client = _make_client(session=mock_session)
        assert client.cancel_order("ACCT123", "order-1") is True

    def test_cancel_order_not_found(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        del_resp = MagicMock()
        del_resp.status_code = 404
        mock_session.delete.return_value = del_resp

        client = _make_client(session=mock_session)
        assert client.cancel_order("ACCT123", "nonexistent") is False


class TestPositions:
    def test_get_positions(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        pos_resp = MagicMock()
        pos_resp.status_code = 200
        pos_resp.json.return_value = {
            "data": {
                "items": [
                    {
                        "symbol": "SPY",
                        "quantity": "100",
                        "average-price": "500.00",
                    },
                    {
                        "instrument-symbol": "AAPL",
                        "quantity": "50",
                        "average-price": "175.00",
                    },
                ]
            }
        }
        pos_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = pos_resp

        client = _make_client(session=mock_session)
        positions = client.get_positions("ACCT123")

        assert len(positions) == 2


class TestAccounts:
    def test_get_accounts(self):
        mock_session = MagicMock()
        _mock_login(mock_session)

        acct_resp = MagicMock()
        acct_resp.status_code = 200
        acct_resp.json.return_value = {
            "data": {
                "items": [
                    {"account-number": "ACCT1"},
                    {"account-number": "ACCT2"},
                ]
            }
        }
        acct_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = acct_resp

        client = _make_client(session=mock_session)
        accounts = client.get_accounts()

        assert len(accounts) == 2
        assert accounts[0]["account-number"] == "ACCT1"


class TestCredentials:
    def test_missing_credentials_raises(self):
        with patch.dict("os.environ", {}, clear=True):
            with pytest.raises(RuntimeError, match="Missing Tastytrade"):
                from python.integration.tastytrade_client import TastytradeClient
                TastytradeClient()

    def test_oauth_mode(self):
        with patch.dict("os.environ", {
            "TASTYTRADE_CLIENT_SECRET": "secret",
            "TASTYTRADE_REFRESH_TOKEN": "refresh",
        }, clear=False):
            from python.integration.tastytrade_client import TastytradeClient
            client = TastytradeClient(session=MagicMock())
            assert client._use_oauth is True

    def test_sandbox_mode(self):
        mock_session = MagicMock()
        client = _make_client(session=mock_session)
        assert client.sandbox is True
        assert "cert" in client.base_url
