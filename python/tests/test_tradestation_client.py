"""Tests for TradeStation client position fetching."""

import pytest
from unittest.mock import MagicMock, patch


class TestTradeStationClient:
    """Tests using mocked HTTP responses."""

    def _make_client(self, session=None):
        with patch.dict("os.environ", {
            "TRADESTATION_CLIENT_ID": "test_id",
            "TRADESTATION_CLIENT_SECRET": "test_secret",
            "TRADESTATION_ACCOUNT_ID": "ACCT123",
            "TRADESTATION_SIM": "1",
        }):
            from python.integration.tradestation_client import TradeStationClient
            return TradeStationClient(session=session or MagicMock())

    def test_init_with_env_vars(self):
        client = self._make_client()
        assert client.client_id == "test_id"
        assert client.account_id == "ACCT123"
        assert "sim-api" in client.base_url

    def test_get_positions_formats_response(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.json.return_value = {
            "Positions": [
                {
                    "Symbol": "AAPL",
                    "Quantity": "100",
                    "AveragePrice": "175.50",
                    "Last": "180.00",
                    "MarketValue": "18000.00",
                    "UnrealizedProfitLoss": "450.00",
                    "Currency": "USD",
                },
                {
                    "Symbol": "MSFT",
                    "Quantity": "50",
                    "AveragePrice": "400.00",
                    "Last": "410.00",
                    "MarketValue": "20500.00",
                    "UnrealizedProfitLoss": "500.00",
                },
            ]
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = mock_resp

        # Mock token endpoint
        token_resp = MagicMock()
        token_resp.json.return_value = {"access_token": "tok", "expires_in": 3600}
        token_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = token_resp

        client = self._make_client(session=mock_session)
        positions = client.get_positions()

        assert len(positions) == 2
        assert positions[0]["symbol"] == "AAPL"
        assert positions[0]["quantity"] == 100.0
        assert positions[0]["avg_price"] == 175.50
        assert positions[0]["current_price"] == 180.00
        assert positions[1]["symbol"] == "MSFT"

    def test_get_positions_short(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.json.return_value = {
            "Positions": [
                {
                    "Symbol": "TSLA",
                    "Quantity": "25",
                    "LongShort": "SHORT",
                    "AveragePrice": "300.00",
                },
            ]
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = mock_resp

        token_resp = MagicMock()
        token_resp.json.return_value = {"access_token": "tok", "expires_in": 3600}
        token_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = token_resp

        client = self._make_client(session=mock_session)
        positions = client.get_positions()

        assert len(positions) == 1
        assert positions[0]["quantity"] == -25.0

    def test_get_positions_http_error(self):
        import requests as req_lib
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.raise_for_status.side_effect = req_lib.HTTPError("404")
        mock_session.get.return_value = mock_resp

        token_resp = MagicMock()
        token_resp.json.return_value = {"access_token": "tok", "expires_in": 3600}
        token_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = token_resp

        client = self._make_client(session=mock_session)
        positions = client.get_positions()
        assert positions == []

    def test_get_positions_empty_response(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.json.return_value = {"Positions": []}
        mock_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = mock_resp

        token_resp = MagicMock()
        token_resp.json.return_value = {"access_token": "tok", "expires_in": 3600}
        token_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = token_resp

        client = self._make_client(session=mock_session)
        positions = client.get_positions()
        assert positions == []

    def test_get_accounts(self):
        mock_session = MagicMock()
        mock_resp = MagicMock()
        mock_resp.json.return_value = {
            "Accounts": [
                {"AccountID": "ACCT1", "AccountType": "Margin"},
                {"AccountID": "ACCT2", "AccountType": "Cash"},
            ]
        }
        mock_resp.raise_for_status = MagicMock()
        mock_session.get.return_value = mock_resp

        token_resp = MagicMock()
        token_resp.json.return_value = {"access_token": "tok", "expires_in": 3600}
        token_resp.raise_for_status = MagicMock()
        mock_session.post.return_value = token_resp

        client = self._make_client(session=mock_session)
        accounts = client.get_accounts()
        assert len(accounts) == 2
        assert accounts[0]["AccountID"] == "ACCT1"

    def test_missing_credentials_raises(self):
        with patch.dict("os.environ", {}, clear=True):
            with pytest.raises(RuntimeError, match="Missing TradeStation"):
                from python.integration.tradestation_client import TradeStationClient
                TradeStationClient()
