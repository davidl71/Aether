"""
Tests for IB service (FastAPI endpoints and helper functions).

Tests FastAPI endpoints, helper functions, and IBKR Portal client integration.
"""
import unittest
from pathlib import Path
from unittest.mock import Mock, patch
from datetime import datetime
import os
import json
import tempfile
from fastapi.testclient import TestClient

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from integration.ib_service import (
    create_app,
    _now_iso,
    _symbols_from_env,
    _extract_account_value,
    _format_ibcid_display_name,
    build_snapshot_payload,
    ModeRequest,
    AccountRequest,
)
from integration.ibkr_portal_client import IBKRPortalClient, IBKRPortalError


class TestHelperFunctions(unittest.TestCase):
    """Tests for helper functions."""

    def test_now_iso(self):
        """Test _now_iso() returns ISO format string."""
        result = _now_iso()
        # Should be ISO 8601 format
        assert isinstance(result, str)
        assert 'T' in result or 'Z' in result or '+' in result
        # Should be parseable
        datetime.fromisoformat(result.replace('Z', '+00:00'))

    def test_symbols_from_env_default(self):
        """Test _symbols_from_env() with default value."""
        with patch.dict(os.environ, {}, clear=True):
            result = _symbols_from_env()
            assert result == ["SPY", "QQQ"]

    def test_symbols_from_env_custom(self):
        """Test _symbols_from_env() with custom SYMBOLS env var."""
        with patch.dict(os.environ, {"SYMBOLS": "AAPL,MSFT,GOOGL"}):
            result = _symbols_from_env()
            assert result == ["AAPL", "MSFT", "GOOGL"]

    def test_symbols_from_env_with_spaces(self):
        """Test _symbols_from_env() handles spaces correctly."""
        with patch.dict(os.environ, {"SYMBOLS": " AAPL , MSFT , GOOGL "}):
            result = _symbols_from_env()
            assert result == ["AAPL", "MSFT", "GOOGL"]

    def test_symbols_from_env_empty(self):
        """Test _symbols_from_env() with empty SYMBOLS."""
        with patch.dict(os.environ, {"SYMBOLS": ""}):
            result = _symbols_from_env()
            assert result == []

    def test_format_ibcid_display_name_t_bill(self):
        """IBCID T-bill becomes readable T-Bill (conid) with optional maturity."""
        assert _format_ibcid_display_name("IBCID123", "BILL", 2, None) == "T-Bill (2)"
        assert _format_ibcid_display_name("IBCID123", "BILL", 2, "2025-06-15") == "T-Bill 2025-06-15 (2)"

    def test_format_ibcid_display_name_bond(self):
        """IBCID bond becomes Bond (conid)."""
        assert _format_ibcid_display_name("IBCID456", "BOND", 99, None) == "Bond (99)"

    def test_format_ibcid_display_name_passthrough(self):
        """Non-IBCID names and unknown asset class are unchanged."""
        assert _format_ibcid_display_name("SPY", "STK", 100, None) == "SPY"
        assert _format_ibcid_display_name("BND", "STK", 99, None) == "BND"
        assert _format_ibcid_display_name("IBCID1", "OPT", 1, None) == "IBCID1"

    def test_extract_account_value_valid(self):
        """Test _extract_account_value() with valid data."""
        summary = {
            "NetLiquidation": [{"value": "100523.45"}],
            "BuyingPower": [{"value": "50000.00"}],
        }
        result = _extract_account_value(summary, "NetLiquidation")
        assert result == 100523.45

    def test_extract_account_value_default(self):
        """Test _extract_account_value() with missing key."""
        summary = {"OtherKey": [{"value": "123"}]}
        result = _extract_account_value(summary, "NetLiquidation", default=0.0)
        assert result == 0.0

    def test_extract_account_value_invalid_dict(self):
        """Test _extract_account_value() with invalid summary structure."""
        summary = {"NetLiquidation": "not a list"}
        result = _extract_account_value(summary, "NetLiquidation", default=999.0)
        assert result == 999.0

    def test_extract_account_value_empty_list(self):
        """Test _extract_account_value() with empty list."""
        summary = {"NetLiquidation": []}
        result = _extract_account_value(summary, "NetLiquidation", default=0.0)
        assert result == 0.0

    def test_extract_account_value_invalid_value(self):
        """Test _extract_account_value() with non-numeric value."""
        summary = {"NetLiquidation": [{"value": "not_a_number"}]}
        result = _extract_account_value(summary, "NetLiquidation", default=0.0)
        assert result == 0.0

    def test_extract_account_value_not_dict(self):
        """Test _extract_account_value() with non-dict summary."""
        result = _extract_account_value("not a dict", "NetLiquidation", default=0.0)
        assert result == 0.0

    def test_extract_account_value_total_cash_aliases(self):
        """Test _extract_account_value() with TotalCashValue and camelCase/flat aliases."""
        # PascalCase nested (existing Client Portal format)
        assert _extract_account_value({"TotalCashValue": [{"value": "12345.67"}]}, "TotalCashValue") == 12345.67
        # camelCase nested
        assert _extract_account_value({"totalCashValue": [{"value": "20000.00"}]}, "TotalCashValue") == 20000.00
        # Flat scalar string
        assert _extract_account_value({"TotalCashValue": "5000.50"}, "TotalCashValue") == 5000.50
        # Flat scalar number
        assert _extract_account_value({"CashBalance": 999.99}, "TotalCashValue") == 999.99

    def test_extract_cash_by_currency_from_summary_nested(self):
        """Test _extract_cash_by_currency_from_summary with nested cash dict (USD + CHF)."""
        from integration.ib_service import _extract_cash_by_currency_from_summary
        summary = {"cash": {"USD": 10000.0, "CHF": 2500.50}}
        result = _extract_cash_by_currency_from_summary(summary)
        assert len(result) == 2
        by_curr = {r["currency"]: r["balance"] for r in result}
        assert by_curr["USD"] == 10000.0
        assert by_curr["CHF"] == 2500.50

    def test_extract_cash_by_currency_from_summary_list(self):
        """Test _extract_cash_by_currency_from_summary with balanceByCurrency list."""
        from integration.ib_service import _extract_cash_by_currency_from_summary
        summary = {
            "balanceByCurrency": [
                {"currency": "USD", "value": "5000.00"},
                {"currency": "CHF", "balance": 1000.0},
            ]
        }
        result = _extract_cash_by_currency_from_summary(summary)
        assert len(result) == 2
        by_curr = {r["currency"]: r["balance"] for r in result}
        assert by_curr["USD"] == 5000.0
        assert by_curr["CHF"] == 1000.0


class TestBuildSnapshotPayload(unittest.TestCase):
    """Tests for build_snapshot_payload() function."""

    def setUp(self):
        """Set up test fixtures."""
        self.mock_client = Mock(spec=IBKRPortalClient)

    def test_build_snapshot_payload_success(self):
        """Test build_snapshot_payload() with successful client calls."""
        self.mock_client.get_snapshots_batch.return_value = [
            {"last": 450.0, "bid": 449.5, "ask": 450.5, "close": 449.0, "volume": 1000000},
            {"last": 380.0, "bid": 379.5, "ask": 380.5, "close": 379.0, "volume": 500000},
        ]
        self.mock_client.get_account_summary.return_value = {
            "NetLiquidation": [{"value": "100000.00"}],
            "BuyingPower": [{"value": "50000.00"}],
            "TotalCashValue": [{"value": "25000.00"}],
        }
        self.mock_client.get_accounts.return_value = ["DU123456"]
        self.mock_client.get_portfolio_positions.return_value = [
            {
                "ticker": "SPY",
                "position": "100",
                "averageCost": "450.0",
                "markPrice": "451.0",
                "markValue": "45100.0",
                "unrealizedPnl": "100.0",
            }
        ]

        symbols = ["SPY", "QQQ"]
        result = build_snapshot_payload(symbols, self.mock_client, "DU123456")

        assert result["account_id"] == "DU123456"
        assert len(result["symbols"]) == 2
        assert result["symbols"][0]["symbol"] == "SPY"
        assert result["symbols"][0]["last"] == 450.0
        assert result["symbols"][0]["bid"] == 449.5
        assert result["symbols"][0]["ask"] == 450.5
        assert result["symbols"][0]["spread"] == 1.0
        assert result["metrics"]["net_liq"] == 100000.0
        # Positions: SPY + Cash (USD) when ledger is not used
        assert len(result["positions"]) == 2
        assert result["positions"][0]["symbol"] == "SPY"
        cash_pos = next(p for p in result["positions"] if p.get("instrument_type") == "cash")
        assert cash_pos["name"] == "Cash (USD)" and cash_pos["market_value"] == 25000.0
        self.mock_client.get_snapshots_batch.assert_called_once_with(["SPY", "QQQ"])
        # Session ensured once before parallel block; flag set/cleared.
        self.mock_client.ensure_session.assert_called_once()
        self.mock_client.set_session_ensured_for_request.assert_any_call(True)
        self.mock_client.set_session_ensured_for_request.assert_any_call(False)
        # When account_id is passed, get_accounts is only called once (at start), not from summary/positions.
        self.mock_client.get_accounts.assert_called_once()

    def test_build_snapshot_payload_client_error(self):
        """Test build_snapshot_payload() handles IBKRPortalError."""
        self.mock_client.get_snapshots_batch.side_effect = IBKRPortalError("Connection failed")
        self.mock_client.get_account_summary.side_effect = IBKRPortalError("Auth failed")
        self.mock_client.get_accounts.return_value = []
        self.mock_client.get_portfolio_positions.side_effect = IBKRPortalError("Positions failed")

        symbols = ["SPY"]
        result = build_snapshot_payload(symbols, self.mock_client)

        # Should still return payload with empty/zero values; one Cash (USD) position at 0 when summary fails
        assert len(result["symbols"]) == 1
        assert result["symbols"][0]["symbol"] == "SPY"
        assert result["symbols"][0]["last"] == 0.0
        assert result["metrics"]["net_liq"] == 0.0
        assert len(result["positions"]) == 1
        assert result["positions"][0]["instrument_type"] == "cash" and result["positions"][0]["market_value"] == 0.0

    def test_build_snapshot_payload_missing_data(self):
        """Test build_snapshot_payload() with missing market data fields."""
        self.mock_client.get_snapshots_batch.return_value = [
            {"last": 450.0},
        ]
        self.mock_client.get_account_summary.return_value = {}
        self.mock_client.get_accounts.return_value = []
        self.mock_client.get_portfolio_positions.return_value = []

        symbols = ["SPY"]
        result = build_snapshot_payload(symbols, self.mock_client)

        assert result["symbols"][0]["bid"] == 0.0
        assert result["symbols"][0]["ask"] == 0.0
        assert result["symbols"][0]["spread"] == 0.0

    def test_build_snapshot_payload_ensure_session_once(self):
        """ensure_session() is called at most once per build_snapshot_payload when using session flag."""
        self.mock_client.get_snapshots_batch.return_value = [{"last": 100.0}]
        self.mock_client.get_account_summary.return_value = {}
        self.mock_client.get_accounts.return_value = ["DU123"]
        self.mock_client.get_portfolio_positions.return_value = []

        build_snapshot_payload(["SPY"], self.mock_client, "DU123")

        self.mock_client.ensure_session.assert_called_once()
        self.mock_client.set_session_ensured_for_request.assert_any_call(True)
        self.mock_client.set_session_ensured_for_request.assert_any_call(False)

    def test_build_snapshot_payload_no_redundant_get_accounts_when_account_id_passed(self):
        """When account_id is passed, get_accounts() is not called from summary/positions paths."""
        self.mock_client.get_snapshots_batch.return_value = [{"last": 100.0}]
        self.mock_client.get_account_summary.return_value = {}
        self.mock_client.get_accounts.return_value = ["DU123"]
        self.mock_client.get_portfolio_positions.return_value = []

        build_snapshot_payload(["SPY"], self.mock_client, "DU999")

        # get_accounts only once (initial resolution in build_snapshot_payload); summary/positions use fast path.
        self.assertEqual(self.mock_client.get_accounts.call_count, 1)


class TestFastAPIEndpoints(unittest.TestCase):
    """Tests for FastAPI endpoints."""

    def setUp(self):
        """Set up test fixtures."""
        # Create app with mocked client
        with patch('integration.ib_service.IBKRPortalClient') as mock_client_class:
            self.mock_client = Mock(spec=IBKRPortalClient)
            mock_client_class.return_value = self.mock_client
            self.app = create_app()
            self.client = TestClient(self.app)

    def test_health_endpoint_returns_immediately(self):
        """Health returns 200 immediately from cached state (no blocking gateway call in request path)."""
        response = self.client.get("/api/health")
        assert response.status_code == 200
        data = response.json()
        assert "status" in data
        assert data["status"] in ("starting", "ok", "error")
        assert "ib_connected" in data
        assert "ts" in data

    def test_health_endpoint_success(self):
        """When connection_state is updated (e.g. by background task), health returns it."""
        self.app.state.connection_state.update({
            "status": "ok",
            "ib_connected": True,
            "gateway_logged_in": True,
            "accounts": ["DU123456"],
            "ts": _now_iso(),
        })
        response = self.client.get("/api/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"
        assert data["ib_connected"] is True
        assert "DU123456" in data["accounts"]

    def test_health_endpoint_error(self):
        """When connection_state has error, health returns it."""
        self.app.state.connection_state.update({
            "status": "error",
            "ib_connected": False,
            "gateway_logged_in": False,
            "error": "Connection failed",
            "accounts": [],
            "ts": _now_iso(),
        })
        response = self.client.get("/api/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "error"
        assert data["ib_connected"] is False
        assert "error" in data

    def test_snapshot_endpoint_success(self):
        """Test /api/v1/snapshot endpoint with successful data."""
        with patch('integration.ib_service._symbols_from_env', return_value=["SPY"]):
            self.mock_client.get_snapshots_batch.return_value = [
                {"last": 450.0, "bid": 449.5, "ask": 450.5, "close": 449.0, "volume": 1000000},
            ]
            self.mock_client.get_account_summary.return_value = {
                "NetLiquidation": [{"value": "100000.00"}],
            }
            self.mock_client.get_accounts.return_value = ["DU123456"]
            self.mock_client.get_portfolio_positions.return_value = []

            response = self.client.get("/api/v1/snapshot")
            assert response.status_code == 200
            data = response.json()
            assert "symbols" in data
            assert "positions" in data
            assert "metrics" in data
            assert len(data["symbols"]) == 1

    def test_snapshot_endpoint_with_account_id(self):
        """Test /api/v1/snapshot endpoint with account_id parameter."""
        with patch('integration.ib_service._symbols_from_env', return_value=["SPY"]):
            self.mock_client.get_snapshots_batch.return_value = [
                {"last": 450.0, "bid": 449.5, "ask": 450.5, "close": 449.0, "volume": 1000000},
            ]
            self.mock_client.get_account_summary.return_value = {}
            self.mock_client.get_accounts.return_value = ["DU123456"]
            self.mock_client.get_portfolio_positions.return_value = []

            response = self.client.get("/api/v1/snapshot?account_id=DU123456")
            assert response.status_code == 200
            self.mock_client.get_account_summary.assert_called_with("DU123456")

    def test_snapshot_endpoint_with_symbols_query_param(self):
        """Test /api/v1/snapshot?symbols=MNQ uses requested symbols (e.g. for nano/micro futures)."""
        self.mock_client.get_snapshots_batch.return_value = [
            {"last": 21500.0, "bid": 21498.0, "ask": 21502.0, "close": 21499.0, "volume": 50000},
        ]
        self.mock_client.get_account_summary.return_value = {}
        self.mock_client.get_accounts.return_value = ["DU123"]
        self.mock_client.get_portfolio_positions.return_value = []

        response = self.client.get("/api/v1/snapshot?symbols=MNQ")
        assert response.status_code == 200
        data = response.json()
        assert "symbols" in data
        assert len(data["symbols"]) == 1
        assert data["symbols"][0]["symbol"] == "MNQ"
        assert data["symbols"][0]["last"] == 21500.0
        self.mock_client.get_snapshots_batch.assert_called_once_with(["MNQ"])

    def test_snapshot_endpoint_writes_file(self):
        """Test /api/v1/snapshot endpoint writes file when SNAPSHOT_FILE_PATH is set."""
        with patch('integration.ib_service._symbols_from_env', return_value=["SPY"]):
            with patch.dict(os.environ, {"SNAPSHOT_FILE_PATH": "/tmp/test_snapshot.json"}):
                self.mock_client.get_snapshots_batch.return_value = [
                    {"last": 450.0, "bid": 449.5, "ask": 450.5, "close": 449.0, "volume": 1000000},
                ]
                self.mock_client.get_account_summary.return_value = {}
                self.mock_client.get_accounts.return_value = []
                self.mock_client.get_portfolio_positions.return_value = []

                # Recreate app to pick up env var
                with patch('integration.ib_service.IBKRPortalClient') as mock_client_class:
                    mock_client_class.return_value = self.mock_client
                    app = create_app()
                    TestClient(app)

                    with tempfile.TemporaryDirectory() as tmpdir:
                        snapshot_path = os.path.join(tmpdir, "snapshot.json")
                        with patch.dict(os.environ, {"SNAPSHOT_FILE_PATH": snapshot_path}):
                            # Recreate app again with new path
                            with patch('integration.ib_service.IBKRPortalClient') as mock_client_class2:
                                mock_client_class2.return_value = self.mock_client
                                app2 = create_app()
                                client2 = TestClient(app2)

                                response = client2.get("/api/v1/snapshot")
                                assert response.status_code == 200

                                # Check if file was created
                                if os.path.exists(snapshot_path):
                                    with open(snapshot_path, 'r') as f:
                                        file_data = json.load(f)
                                        assert "symbols" in file_data

    def test_snapshot_endpoint_error_handling(self):
        """Test /api/v1/snapshot endpoint handles exceptions."""
        with patch('integration.ib_service._symbols_from_env', return_value=["SPY"]):
            self.mock_client.get_snapshots_batch.side_effect = Exception("Unexpected error")

            response = self.client.get("/api/v1/snapshot")
            assert response.status_code == 200  # Endpoint doesn't fail, returns error in payload
            data = response.json()
            assert "error" in data

    def test_positions_endpoint_success(self):
        """Test /api/positions endpoint with successful data."""
        self.mock_client.get_portfolio_positions.return_value = [
            {
                "ticker": "SPY",
                "position": "100",
                "averageCost": "450.0",
                "markPrice": "451.0",
                "markValue": "45100.0",
                "unrealizedPnl": "100.0",
            }
        ]

        response = self.client.get("/api/positions")
        assert response.status_code == 200
        data = response.json()
        assert len(data) == 1
        assert data[0]["symbol"] == "SPY"
        assert data[0]["quantity"] == 100.0

    def test_positions_endpoint_with_account_id(self):
        """Test /api/positions endpoint with account_id parameter."""
        self.mock_client.get_portfolio_positions.return_value = []

        response = self.client.get("/api/positions?account_id=DU123456")
        assert response.status_code == 200
        self.mock_client.get_portfolio_positions.assert_called_with("DU123456")

    def test_positions_endpoint_error(self):
        """Test /api/positions endpoint handles IBKRPortalError."""
        self.mock_client.get_portfolio_positions.side_effect = IBKRPortalError("Connection failed")

        response = self.client.get("/api/positions")
        assert response.status_code == 200
        data = response.json()
        assert len(data) == 1
        assert "error" in data[0]

    def test_list_accounts_endpoint_success(self):
        """Test /api/accounts endpoint with successful data."""
        self.mock_client.get_accounts.return_value = ["DU123456", "DU789012"]
        self.mock_client.get_account_summary.return_value = {
            "NetLiquidation": [{"value": "100000.00"}],
            "BuyingPower": [{"value": "50000.00"}],
        }

        response = self.client.get("/api/accounts")
        assert response.status_code == 200
        data = response.json()
        assert "accounts" in data
        assert len(data["accounts"]) == 2
        assert data["accounts"][0]["id"] == "DU123456"

    def test_list_accounts_endpoint_summary_error(self):
        """Test /api/accounts endpoint handles account summary errors."""
        self.mock_client.get_accounts.return_value = ["DU123456"]
        self.mock_client.get_account_summary.side_effect = IBKRPortalError("Summary failed")

        response = self.client.get("/api/accounts")
        assert response.status_code == 200
        data = response.json()
        assert len(data["accounts"]) == 1
        # Account should still be listed even if summary fails
        assert data["accounts"][0]["id"] == "DU123456"

    def test_list_accounts_endpoint_error(self):
        """Test /api/accounts endpoint handles IBKRPortalError."""
        self.mock_client.get_accounts.side_effect = IBKRPortalError("Connection failed")

        response = self.client.get("/api/accounts")
        assert response.status_code == 200
        data = response.json()
        assert "accounts" in data
        assert len(data["accounts"]) == 0
        assert "error" in data

    def test_set_account_endpoint_success(self):
        """Test POST /api/account endpoint with valid account."""
        self.mock_client.get_accounts.return_value = ["DU123456", "DU789012"]

        response = self.client.post("/api/account", json={"account_id": "DU123456"})
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"
        assert data["account_id"] == "DU123456"

    def test_set_account_endpoint_invalid(self):
        """Test POST /api/account endpoint with invalid account."""
        self.mock_client.get_accounts.return_value = ["DU123456"]

        response = self.client.post("/api/account", json={"account_id": "INVALID"})
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "error"
        assert "not found" in data["message"]

    def test_set_account_endpoint_clear(self):
        """Test POST /api/account endpoint clears account when account_id is None."""
        response = self.client.post("/api/account", json={"account_id": None})
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"
        assert data["account_id"] is None

    def test_get_account_endpoint_success(self):
        """Test GET /api/account endpoint with active account."""
        self.mock_client.get_accounts.return_value = ["DU123456"]
        self.mock_client.get_account_summary.return_value = {
            "NetLiquidation": [{"value": "100000.00"}],
            "BuyingPower": [{"value": "50000.00"}],
            "ExcessLiquidity": [{"value": "30000.00"}],
            "MaintMarginReq": [{"value": "20000.00"}],
        }

        response = self.client.get("/api/account")
        assert response.status_code == 200
        data = response.json()
        assert data["account_id"] == "DU123456"
        assert data["net_liquidation"] == 100000.0

    def test_get_account_endpoint_no_accounts(self):
        """Test GET /api/account endpoint with no accounts."""
        self.mock_client.get_accounts.return_value = []

        response = self.client.get("/api/account")
        assert response.status_code == 200
        data = response.json()
        assert data["account_id"] is None

    def test_get_account_endpoint_error(self):
        """Test GET /api/account endpoint handles IBKRPortalError."""
        self.mock_client.get_accounts.side_effect = IBKRPortalError("Connection failed")

        response = self.client.get("/api/account")
        assert response.status_code == 200
        data = response.json()
        assert data["account_id"] is None
        assert "error" in data


class TestPydanticModels(unittest.TestCase):
    """Tests for Pydantic model classes."""

    def test_mode_request(self):
        """Test ModeRequest model."""
        request = ModeRequest(mode="LIVE")
        assert request.mode == "LIVE"

    def test_account_request_with_id(self):
        """Test AccountRequest model with account_id."""
        request = AccountRequest(account_id="DU123456")
        assert request.account_id == "DU123456"

    def test_account_request_without_id(self):
        """Test AccountRequest model without account_id."""
        request = AccountRequest(account_id=None)
        assert request.account_id is None


if __name__ == "__main__":
    unittest.main()
