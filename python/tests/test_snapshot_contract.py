"""
Snapshot contract and golden-file tests.

Ensures the snapshot API and SnapshotPayload stay compatible with the
shared contract (web/src/types/snapshot.ts, TUI, PWA).

Live contract data test: run with RUN_LIVE_SNAPSHOT_TESTS=1 and the IB
service running (e.g. ./scripts/service.sh start ib). Default symbol is SPY;
set LIVE_SNAPSHOT_SYMBOLS=MNQ or NQ for nano/micro futures.
"""
from pathlib import Path
import json
import os
import sys

import pytest

# Add project root for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from tui.models import SnapshotPayload


# Required top-level keys in snapshot API response (GET /api/snapshot)
SNAPSHOT_REQUIRED_KEYS = {
    "generated_at",
    "mode",
    "strategy",
    "account_id",
    "metrics",
    "symbols",
    "positions",
    "historic",
    "orders",
    "alerts",
}
# Required keys inside metrics
METRICS_REQUIRED_KEYS = {
    "net_liq",
    "buying_power",
    "excess_liquidity",
    "margin_requirement",
    "commissions",
    "portal_ok",
    "tws_ok",
    "questdb_ok",
}
# Required keys per symbol in symbols[]
SYMBOL_REQUIRED_KEYS = {
    "symbol",
    "last",
    "bid",
    "ask",
    "spread",
    "roi",
    "maker_count",
    "taker_count",
    "volume",
    "candle",
    "option_chains",
}
CANDLE_KEYS = {"open", "high", "low", "close", "volume", "entry", "updated"}


def _assert_snapshot_shape(data: dict) -> None:
    """Assert dict has snapshot contract shape (required keys and types)."""
    assert isinstance(data, dict), "Snapshot must be a dict"
    missing = SNAPSHOT_REQUIRED_KEYS - set(data.keys())
    assert not missing, f"Snapshot missing keys: {missing}"

    assert isinstance(data["generated_at"], str)
    assert isinstance(data["mode"], str)
    assert isinstance(data["strategy"], str)
    assert isinstance(data["account_id"], str)
    assert isinstance(data["symbols"], list)
    assert isinstance(data["positions"], list)
    assert isinstance(data["historic"], list)
    assert isinstance(data["orders"], list)
    assert isinstance(data["alerts"], list)

    metrics = data["metrics"]
    assert isinstance(metrics, dict), "metrics must be a dict"
    missing_m = METRICS_REQUIRED_KEYS - set(metrics.keys())
    assert not missing_m, f"metrics missing keys: {missing_m}"
    assert isinstance(metrics["net_liq"], (int, float))
    assert isinstance(metrics["buying_power"], (int, float))
    assert isinstance(metrics["portal_ok"], bool)

    for sym in data["symbols"]:
        assert isinstance(sym, dict)
        missing_s = SYMBOL_REQUIRED_KEYS - set(sym.keys())
        assert not missing_s, f"symbol entry missing keys: {missing_s}"
        assert isinstance(sym["symbol"], str)
        assert isinstance(sym["candle"], dict)
        assert CANDLE_KEYS <= set(sym["candle"].keys()), f"candle missing keys: {CANDLE_KEYS - set(sym['candle'].keys())}"


class TestSnapshotContract:
    """Snapshot API contract: required keys and types."""

    def test_snapshot_payload_from_dict_roundtrip(self):
        """SnapshotPayload.from_dict then to_dict preserves required shape."""
        minimal = {
            "generated_at": "2025-01-15T12:00:00Z",
            "mode": "DRY-RUN",
            "strategy": "RUNNING",
            "account_id": "DU123",
            "metrics": {
                "net_liq": 100000.0,
                "buying_power": 50000.0,
                "excess_liquidity": 45000.0,
                "margin_requirement": 5000.0,
                "commissions": 0.0,
                "portal_ok": True,
                "tws_ok": False,
                "questdb_ok": False,
            },
            "symbols": [
                {
                    "symbol": "SPY",
                    "last": 450.0,
                    "bid": 449.5,
                    "ask": 450.5,
                    "spread": 1.0,
                    "roi": 0.0,
                    "maker_count": 0,
                    "taker_count": 0,
                    "volume": 1000000,
                    "candle": {
                        "open": 449.0,
                        "high": 451.0,
                        "low": 448.0,
                        "close": 450.0,
                        "volume": 1000000,
                        "entry": 449.0,
                        "updated": "2025-01-15T12:00:00Z",
                    },
                    "option_chains": [],
                }
            ],
            "positions": [],
            "historic": [],
            "orders": [],
            "alerts": [],
        }
        payload = SnapshotPayload.from_dict(minimal)
        assert payload.mode == "DRY-RUN"
        assert payload.account_id == "DU123"
        assert len(payload.symbols) == 1
        assert payload.symbols[0].symbol == "SPY"
        assert payload.metrics.net_liq == 100000.0

        back = payload.to_dict()
        _assert_snapshot_shape(back)
        assert back["mode"] == minimal["mode"]
        assert back["symbols"][0]["symbol"] == "SPY"

    def test_empty_snapshot_has_required_shape(self):
        """Empty SnapshotPayload().to_dict() has required keys."""
        empty = SnapshotPayload()
        data = empty.to_dict()
        _assert_snapshot_shape(data)
        assert data["mode"] == "DRY-RUN"
        assert data["strategy"] == "STOPPED"
        assert data["symbols"] == []

    def test_ib_service_snapshot_response_shape(self):
        """GET /api/snapshot returns JSON with snapshot contract shape."""
        from unittest.mock import Mock, patch
        from fastapi.testclient import TestClient
        from integration.ib_service import create_app
        from integration.ibkr_portal_client import IBKRPortalClient

        with patch("integration.ib_service.IBKRPortalClient") as mock_cls:
            mock_client = Mock(spec=IBKRPortalClient)
            mock_client.get_snapshot.return_value = {
                "last": 450.0,
                "bid": 449.5,
                "ask": 450.5,
                "close": 449.0,
                "volume": 1000000,
            }
            mock_client.get_account_summary.return_value = {}
            mock_client.get_accounts.return_value = ["DU123"]
            mock_client.get_portfolio_positions.return_value = []
            mock_cls.return_value = mock_client

            app = create_app()
            client = TestClient(app)

        with patch("integration.ib_service._symbols_from_env", return_value=["SPY"]):
            response = client.get("/api/snapshot")
        assert response.status_code == 200
        data = response.json()
        _assert_snapshot_shape(data)
        assert "symbols" in data
        assert len(data["symbols"]) >= 1

    def test_mock_provider_snapshot_shape(self):
        """MockProvider._generate_snapshot() returns payload matching contract."""
        from tui.providers import MockProvider

        provider = MockProvider(update_interval_ms=1000)
        snapshot = provider._generate_snapshot()
        assert isinstance(snapshot, SnapshotPayload)
        data = snapshot.to_dict()
        _assert_snapshot_shape(data)
        assert snapshot.mode == "DRY-RUN"
        assert len(snapshot.symbols) >= 1


class TestSnapshotGolden:
    """Golden snapshot file: regression test for snapshot structure."""

    @pytest.fixture
    def golden_path(self):
        return Path(__file__).parent / "snapshots" / "snapshot_golden.json"

    def test_golden_snapshot_loads_and_roundtrips(self, golden_path):
        """Golden snapshot JSON loads and SnapshotPayload round-trips."""
        if not golden_path.exists():
            pytest.skip("Golden snapshot not found; create python/tests/snapshots/snapshot_golden.json")
        with open(golden_path, encoding="utf-8") as f:
            data = json.load(f)
        _assert_snapshot_shape(data)
        payload = SnapshotPayload.from_dict(data)
        back = payload.to_dict()
        _assert_snapshot_shape(back)
        assert back["mode"] == data["mode"]
        assert back["account_id"] == data["account_id"]
        assert len(back["symbols"]) == len(data["symbols"])

    def test_ib_build_snapshot_matches_golden_shape(self, golden_path):
        """build_snapshot_payload output has at least the same top-level keys as golden."""
        from unittest.mock import Mock, patch
        from integration.ib_service import build_snapshot_payload
        from integration.ibkr_portal_client import IBKRPortalClient

        if not golden_path.exists():
            pytest.skip("Golden snapshot not found")
        with open(golden_path, encoding="utf-8") as f:
            golden = json.load(f)
        golden_keys = set(golden.keys())

        mock_client = Mock(spec=IBKRPortalClient)
        mock_client.get_snapshot.return_value = {
            "last": 450.0,
            "bid": 449.5,
            "ask": 450.5,
            "close": 449.0,
            "volume": 1000000,
        }
        mock_client.get_account_summary.return_value = {}
        mock_client.get_accounts.return_value = ["DU123"]
        mock_client.get_portfolio_positions.return_value = []

        built = build_snapshot_payload(["SPY"], mock_client)
        _assert_snapshot_shape(built)
        assert golden_keys <= set(built.keys()), f"Built snapshot missing keys: {golden_keys - set(built.keys())}"


# Default symbols for live test (SPY is widely available; use MNQ, NQ, etc. for futures)
LIVE_SNAPSHOT_SYMBOLS = os.getenv("LIVE_SNAPSHOT_SYMBOLS", "SPY")
LIVE_SNAPSHOT_URL = os.getenv("LIVE_SNAPSHOT_URL", "http://127.0.0.1:8002/api/snapshot")


@pytest.mark.skipif(
    not os.getenv("RUN_LIVE_SNAPSHOT_TESTS"),
    reason="Set RUN_LIVE_SNAPSHOT_TESTS=1 and run IB service (e.g. ./scripts/service.sh start ib) to run",
)
class TestLiveSnapshotRealContractData:
    """Live snapshot test: hit real IB service for real contract data (e.g. MNQ, NQ, SPY)."""

    def test_live_snapshot_real_contract_data(self):
        """GET snapshot from IB service with real symbols (e.g. MNQ); assert shape and real data."""
        import requests

        try:
            response = requests.get(
                LIVE_SNAPSHOT_URL,
                params={"symbols": LIVE_SNAPSHOT_SYMBOLS},
                timeout=15,
                headers={"Accept": "application/json"},
            )
        except (requests.ConnectionError, requests.Timeout) as e:
            pytest.skip(f"IB service not reachable at {LIVE_SNAPSHOT_URL}: {e}")

        assert response.status_code == 200, response.text
        data = response.json()

        if data.get("error") and not data.get("symbols"):
            pytest.skip(f"IB service returned error payload (gateway/auth?): {data.get('error')}")

        _assert_snapshot_shape(data)
        assert data["symbols"], "Expected at least one symbol in snapshot"

        # At least one symbol should have real-looking data (last or bid/ask)
        has_real = any(
            (float(s.get("last", 0) or 0) > 0
            or (float(s.get("bid", 0) or 0) > 0)
            or (float(s.get("ask", 0) or 0) > 0)
            for s in data["symbols"])
        )
        assert has_real, (
            "Expected at least one symbol with last/bid/ask > 0 (real contract data). "
            f"Symbols requested: {LIVE_SNAPSHOT_SYMBOLS}. "
            "Ensure IB Gateway is logged in and market data is enabled."
        )
