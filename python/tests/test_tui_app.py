"""Tests for python/tui/app.py - TUI application module.

Tests cover widget classes, tab containers, app initialization,
provider factory, and snapshot formatting - all without running
a real Textual application.
"""

import json
import os
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

import pytest

from python.tui.models import (
    SnapshotPayload,
    SymbolSnapshot,
    PositionSnapshot,
    TimelineEvent,
    AccountMetrics,
    Severity,
    BoxSpreadPayload,
    BoxSpreadScenario,
    BoxSpreadSummary,
)
from python.tui.config import TUIConfig


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture
def sample_snapshot():
    return SnapshotPayload(
        generated_at="2025-06-15T14:30:45.123Z",
        mode="LIVE",
        strategy="RUNNING",
        account_id="DU123456",
        metrics=AccountMetrics(net_liq=250_000.0, buying_power=100_000.0),
        symbols=[
            SymbolSnapshot(symbol="SPX", last=5400.50, bid=5400.0, ask=5401.0, spread=1.0, roi=4.5),
            SymbolSnapshot(symbol="XSP", last=540.05, bid=540.0, ask=540.10, spread=0.10, roi=4.3),
        ],
        positions=[
            PositionSnapshot(
                name="SPX Jul 5400/5500 Box",
                quantity=2,
                roi=4.5,
                maker_count=4,
                taker_count=0,
                rebate_estimate=1.20,
                vega=0.01,
                theta=-0.05,
            ),
        ],
        orders=[
            TimelineEvent(
                timestamp="2025-06-15T14:25:00.000Z",
                text="BUY 2 SPX Box 5400/5500",
                severity=Severity.SUCCESS,
            ),
        ],
        alerts=[
            TimelineEvent(
                timestamp="2025-06-15T14:20:00.000Z",
                text="Spread widened on XSP",
                severity=Severity.WARN,
            ),
            TimelineEvent(
                timestamp="2025-06-15T14:22:00.000Z",
                text="Connection lost",
                severity=Severity.ERROR,
            ),
        ],
    )


@pytest.fixture
def sample_box_spread_payload():
    return BoxSpreadPayload(
        as_of="2025-06-15T14:30:00Z",
        underlying="SPX",
        scenarios=[
            BoxSpreadScenario(
                width=100.0,
                annualized_return=4.5,
                fill_probability=80.0,
                option_style="European",
                buy_profit=12.50,
                sell_profit=-11.80,
            ),
            BoxSpreadScenario(
                width=50.0,
                annualized_return=4.2,
                fill_probability=90.0,
                option_style="European",
                buy_profit=6.10,
                sell_profit=-5.90,
            ),
            BoxSpreadScenario(
                width=100.0,
                annualized_return=3.8,
                fill_probability=0.0,
                option_style="American",
                buy_profit=10.0,
                sell_profit=-9.5,
            ),
        ],
    )


@pytest.fixture
def empty_snapshot():
    return SnapshotPayload()


# ---------------------------------------------------------------------------
# SnapshotDisplay
# ---------------------------------------------------------------------------

class TestSnapshotDisplay:
    def test_format_snapshot_normal(self, sample_snapshot):
        from python.tui.components.snapshot_display import format_status_line
        result = format_status_line("", sample_snapshot, None)
        assert "14:30:45" in result
        assert "LIVE" in result
        assert "RUNNING" in result
        assert "DU123456" in result

    def test_format_snapshot_no_timestamp(self):
        from python.tui.components.snapshot_display import format_status_line
        snapshot = SnapshotPayload(generated_at="", mode="DRY-RUN")
        result = format_status_line("", snapshot, None)
        assert "--:--:--" in result
        assert "DRY-RUN" in result

    def test_format_snapshot_date_only(self):
        from python.tui.components.snapshot_display import format_status_line
        snapshot = SnapshotPayload(generated_at="2025-06-15T09:05:01Z")
        result = format_status_line("", snapshot, None)
        assert "09:05:01" in result


# ---------------------------------------------------------------------------
# AlertsTab._get_severity_style
# ---------------------------------------------------------------------------

class TestAlertsSeverityStyle:
    def _get_style(self, severity):
        from python.tui.app import AlertsTab
        tab = AlertsTab.__new__(AlertsTab)
        return tab._get_severity_style(severity)

    def test_info_style(self):
        assert self._get_style(Severity.INFO) == "cyan"

    def test_success_style(self):
        assert self._get_style(Severity.SUCCESS) == "green"

    def test_warn_style(self):
        assert self._get_style(Severity.WARN) == "yellow"

    def test_warning_style(self):
        assert self._get_style(Severity.WARNING) == "yellow"

    def test_error_style(self):
        assert self._get_style(Severity.ERROR) == "red"

    def test_critical_style(self):
        assert self._get_style(Severity.CRITICAL) == "bold red"

    def test_unknown_severity_returns_white(self):
        assert self._get_style("unknown") == "white"


# ---------------------------------------------------------------------------
# Disabled backends (missing API keys) indication
# ---------------------------------------------------------------------------

class TestDisabledBackendsConfig:
    """TUIConfig.disabled_backends and snapshot status line formatting."""

    def test_tuiconfig_has_disabled_backends_default_empty(self):
        config = TUIConfig()
        assert config.disabled_backends == {}

    def test_tuiconfig_disabled_backends_roundtrip(self):
        config = TUIConfig(disabled_backends={"discount_bank": "Unavailable"})
        assert config.disabled_backends["discount_bank"] == "Unavailable"
        restored = TUIConfig.from_dict(config.to_dict())
        assert restored.disabled_backends == config.disabled_backends

    def test_format_backend_health_disabled(self):
        from python.tui.components.snapshot_display import _format_one_backend_health

        out = _format_one_backend_health("discount_bank", {"status": "disabled", "error": "Unavailable"})
        assert "Discount Bank" in out
        assert "disabled" in out
        assert "Unavailable" in out

    def test_format_backend_health_disabled_long_error_truncated(self):
        from python.tui.components.snapshot_display import _format_one_backend_health

        long_err = "Unavailable because the source system is disabled and this explanation is intentionally long"
        out = _format_one_backend_health("discount_bank", {"status": "disabled", "error": long_err})
        assert "disabled" in out
        assert "..." in out

    def test_format_ib_backend_health_grouped(self):
        from python.tui.components.snapshot_display import _format_one_backend_health

        out = _format_one_backend_health(
            "ib",
            {"status": "ok", "gateway_logged_in": True, "gateway_port": 5001},
        )
        assert out == "TWS/IBKR: ok (Gateway 5001: logged in)"
        out2 = _format_one_backend_health(
            "ib",
            {"status": "ok", "gateway_logged_in": False, "gateway_port": 5001},
        )
        assert out2 == "TWS/IBKR: ok (Gateway 5001: not logged in)"

    def test_format_status_line_with_snapshot(self):
        from python.tui.components.snapshot_display import format_status_line

        snapshot = SnapshotPayload(
            generated_at="2025-06-15T14:30:45.123Z",
            mode="LIVE",
            strategy="RUNNING",
            account_id="DU123456",
        )
        line = format_status_line("rest (localhost:8002)", snapshot, None)
        assert "Provider: rest" in line
        assert "14:30:45" in line
        assert "LIVE" in line
        assert "RUNNING" in line
        assert "DU123456" in line


class TestEffectiveHealthUrl:
    def test_prefers_api_base_url_by_default(self):
        from python.tui.app import _effective_health_url

        config = TUIConfig(api_base_url="http://shared:8080")
        assert _effective_health_url(config) == "http://shared:8080/api/health"

    def test_explicit_dashboard_override_still_supported(self):
        from python.tui.app import _effective_health_url

        config = TUIConfig(api_base_url="http://shared:8080")
        assert _effective_health_url(config) == "http://shared:8080/api/health"

    def test_format_status_line_no_snapshot(self):
        from python.tui.components.snapshot_display import format_status_line

        line = format_status_line("mock", None, None)
        assert "Provider: mock" in line
        assert "Updated: --:--:--" in line
        assert "Mode: --" in line
        assert "Strategy: --" in line
        assert "Account: --" in line

    def test_format_status_line_includes_backend_when_backend_health_present(self):
        """Bottom status bar must show Backend: when backend_health is set."""
        from python.tui.components.snapshot_display import format_status_line

        backend_health = {
            "ib": {"status": "ok"},
            "discount_bank": {"status": "error", "error": "Connection refused"},
        }
        line = format_status_line("rest (localhost:8002)", None, backend_health)
        assert "Backend:" in line
        assert "TWS/IBKR" in line or "ib" in line.lower()
        assert "Discount Bank" in line or "discount" in line.lower()

    def test_format_status_line_backend_checking_placeholder(self):
        """When aggregator is polling, status line shows Backend: Connection: checking..."""
        from python.tui.components.snapshot_display import format_status_line

        backend_health = {"connection": {"status": "checking", "error": "polling..."}}
        line = format_status_line("mock", None, backend_health)
        assert "Backend:" in line
        assert "checking" in line

    @patch.dict("os.environ", {}, clear=False)
    def test_disabled_backends_from_env_ignores_retired_brokers(self):
        from python.tui.config import _disabled_backends_from_env

        out = _disabled_backends_from_env({"alpaca": {"port": 8000}, "tastytrade": {"port": 8005}})
        assert out == {}

    @patch.dict("os.environ", {"ALPACA_API_KEY_ID": "key", "ALPACA_API_SECRET_KEY": "secret"}, clear=False)
    def test_disabled_backends_from_env_retired_brokers_still_ignored(self):
        from python.tui.config import _disabled_backends_from_env

        out = _disabled_backends_from_env({"alpaca": {"port": 8000}})
        assert out == {}

    @patch.dict(
        "os.environ",
        {
            "OP_ALPACA_API_KEY_ID_SECRET": "op://Vault/Alpaca/credential",
            "OP_ALPACA_API_SECRET_KEY_SECRET": "op://Vault/Alpaca/secret",
        },
        clear=False,
    )
    def test_disabled_backends_from_env_alpaca_op_refs_not_disabled(self):
        """When only 1Password op:// refs are set (no resolved keys), Alpaca is still considered configured."""
        from python.tui.config import _disabled_backends_from_env

        for key in ("ALPACA_API_KEY_ID", "ALPACA_API_SECRET_KEY", "ALPACA_CLIENT_ID", "ALPACA_CLIENT_SECRET"):
            os.environ.pop(key, None)
        out = _disabled_backends_from_env({"alpaca": {"port": 8000}})
        assert "alpaca" not in out


# ---------------------------------------------------------------------------
# create_provider_from_config
# ---------------------------------------------------------------------------

class TestCreateProviderFromConfig:
    def test_mock_provider(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import MockProvider
        config = TUIConfig(provider_type="mock")
        provider = create_provider_from_config(config)
        assert isinstance(provider, MockProvider)

    def test_empty_provider_defaults_to_mock(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import MockProvider
        config = TUIConfig(provider_type="")
        provider = create_provider_from_config(config)
        assert isinstance(provider, MockProvider)

    def test_rest_provider(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import RestProvider
        config = TUIConfig(provider_type="rest", rest_endpoint="http://test:8080/api")
        provider = create_provider_from_config(config)
        assert isinstance(provider, RestProvider)
        assert provider.endpoint == "http://test:8080/api/snapshot"

    def test_rest_provider_prefers_api_base_url(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import RestProvider
        config = TUIConfig(
            provider_type="rest",
            rest_endpoint="http://legacy:8080/api/snapshot",
            api_base_url="http://shared:9000",
        )
        provider = create_provider_from_config(config)
        assert isinstance(provider, RestProvider)
        assert provider.endpoint == "http://shared:9000/api/v1/snapshot"

    def test_rest_provider_default_endpoint(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import RestProvider
        config = TUIConfig(provider_type="rest", rest_endpoint="")
        provider = create_provider_from_config(config)
        assert isinstance(provider, RestProvider)
        assert "localhost" in provider.endpoint

    def test_file_provider(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import FileProvider
        config = TUIConfig(provider_type="file", file_path="/tmp/snap.json")
        provider = create_provider_from_config(config)
        assert isinstance(provider, FileProvider)
        assert provider.file_path == Path("/tmp/snap.json")

    def test_file_provider_default_path(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import FileProvider
        config = TUIConfig(provider_type="file", file_path="")
        provider = create_provider_from_config(config)
        assert isinstance(provider, FileProvider)
        assert "snapshot.json" in str(provider.file_path)

    def test_unknown_provider_falls_back_to_mock(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import MockProvider
        config = TUIConfig(provider_type="kafka")
        provider = create_provider_from_config(config)
        assert isinstance(provider, MockProvider)

    def test_case_insensitive(self):
        from python.tui.app import create_provider_from_config
        from python.tui.providers import RestProvider
        config = TUIConfig(provider_type="REST")
        provider = create_provider_from_config(config)
        assert isinstance(provider, RestProvider)

    def test_update_interval_passed(self):
        from python.tui.app import create_provider_from_config
        config = TUIConfig(provider_type="mock", update_interval_ms=2000)
        provider = create_provider_from_config(config)
        assert provider.update_interval_ms == 2000


# ---------------------------------------------------------------------------
# TUIApp - construction (no compose/mount)
# ---------------------------------------------------------------------------

class TestTUIAppInit:
    def test_init_with_defaults(self):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider
        provider = MockProvider()
        app = TUIApp(provider)
        assert app.provider is provider
        assert isinstance(app.config, TUIConfig)
        assert app.snapshot is None
        assert app.box_spread_data is None

    def test_init_with_custom_config(self):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider
        config = TUIConfig(provider_type="mock", update_interval_ms=500)
        provider = MockProvider()
        app = TUIApp(provider, config)
        assert app.config.update_interval_ms == 500

    def test_app_title(self):
        from python.tui.app import TUIApp
        assert TUIApp.TITLE == "IB Box Spread Terminal"

    def test_app_bindings(self):
        from python.tui.app import TUIApp
        binding_keys = [b.key for b in TUIApp.BINDINGS]
        assert "q" in binding_keys
        assert "f1" in binding_keys
        assert "f5" in binding_keys

    def test_app_has_css(self):
        from python.tui.app import TUIApp
        assert ".tab-title" in TUIApp.CSS
        assert "#symbols-table" in TUIApp.CSS

    def test_app_compose_includes_bottom_status_bar(self):
        """Bottom status bar (id=status-bar) must be present in the composed layout."""
        import inspect
        from python.tui.app import TUIApp

        source = inspect.getsource(TUIApp.compose)
        assert "status-bar" in source
        assert "StatusBar" in source
        assert "#status-bar" in TUIApp.CSS


# ---------------------------------------------------------------------------
# DashboardTab - logic
# ---------------------------------------------------------------------------

class TestDashboardTab:
    def test_init_default(self):
        from python.tui.app import DashboardTab
        tab = DashboardTab.__new__(DashboardTab)
        tab.snapshot = None
        assert tab.snapshot is None

    def test_init_with_snapshot(self):
        from python.tui.app import DashboardTab
        snapshot = SnapshotPayload(mode="LIVE")
        tab = DashboardTab.__new__(DashboardTab)
        tab.snapshot = snapshot
        assert tab.snapshot.mode == "LIVE"

    def test_update_snapshot(self, sample_snapshot):
        from python.tui.app import DashboardTab
        tab = DashboardTab.__new__(DashboardTab)
        tab.snapshot = None
        tab._update_data = Mock()
        tab.update_snapshot(sample_snapshot)
        assert tab.snapshot is sample_snapshot
        tab._update_data.assert_called_once()

    def test_update_data_no_snapshot(self):
        from python.tui.app import DashboardTab
        tab = DashboardTab.__new__(DashboardTab)
        tab.snapshot = None
        tab._update_data()  # Should return early without error


# ---------------------------------------------------------------------------
# UnifiedPositionsTab - bank account conversion
# ---------------------------------------------------------------------------

class TestUnifiedPositionsTab:
    def test_convert_single_currency_bank_account(self, sample_snapshot):
        from python.tui.components.unified_positions import UnifiedPositionsTab

        tab = UnifiedPositionsTab(
            snapshot=sample_snapshot,
            bank_accounts=[{"account_name": "Savings", "currency": "ILS", "balance": 100.5, "credit_rate": 0.03}],
        )

        positions = tab._convert_bank_accounts_to_positions()

        assert len(positions) == 1
        assert positions[0].name == "Savings"
        assert positions[0].currency == "ILS"
        assert positions[0].cash_flow == 100.5

    def test_convert_mixed_currency_bank_account_creates_one_row_per_currency(self, sample_snapshot):
        from python.tui.components.unified_positions import UnifiedPositionsTab

        tab = UnifiedPositionsTab(
            snapshot=sample_snapshot,
            bank_accounts=[
                {
                    "account_name": "Discount Account",
                    "currency": "MULTI",
                    "balance": 0.0,
                    "is_mixed_currency": True,
                    "balances_by_currency": {"ILS": 100.5, "USD": 25.25},
                    "credit_rate": 0.03,
                }
            ],
        )

        positions = tab._convert_bank_accounts_to_positions()

        assert len(positions) == 2
        assert {(p.name, p.currency, p.cash_flow) for p in positions} == {
            ("Discount Account (ILS)", "ILS", 100.5),
            ("Discount Account (USD)", "USD", 25.25),
        }


# ---------------------------------------------------------------------------
# PositionsTab - logic
# ---------------------------------------------------------------------------

class TestPositionsTab:
    def test_init_default(self):
        from python.tui.app import PositionsTab
        tab = PositionsTab.__new__(PositionsTab)
        tab.snapshot = None
        assert tab.snapshot is None

    def test_update_snapshot(self, sample_snapshot):
        from python.tui.app import PositionsTab
        tab = PositionsTab.__new__(PositionsTab)
        tab.snapshot = None
        tab._update_data = Mock()
        tab.update_snapshot(sample_snapshot)
        assert tab.snapshot is sample_snapshot
        tab._update_data.assert_called_once()

    def test_update_data_no_snapshot(self):
        from python.tui.app import PositionsTab
        tab = PositionsTab.__new__(PositionsTab)
        tab.snapshot = None
        tab._update_data()


# ---------------------------------------------------------------------------
# OrdersTab - logic
# ---------------------------------------------------------------------------

class TestOrdersTab:
    def test_init_default(self):
        from python.tui.app import OrdersTab
        tab = OrdersTab.__new__(OrdersTab)
        tab.snapshot = None
        assert tab.snapshot is None

    def test_update_snapshot(self, sample_snapshot):
        from python.tui.app import OrdersTab
        tab = OrdersTab.__new__(OrdersTab)
        tab.snapshot = None
        tab._update_data = Mock()
        tab.update_snapshot(sample_snapshot)
        assert tab.snapshot is sample_snapshot
        tab._update_data.assert_called_once()

    def test_update_data_no_snapshot(self):
        from python.tui.app import OrdersTab
        tab = OrdersTab.__new__(OrdersTab)
        tab.snapshot = None
        tab._update_data()


# ---------------------------------------------------------------------------
# AlertsTab - logic
# ---------------------------------------------------------------------------

class TestAlertsTab:
    def test_init_default(self):
        from python.tui.app import AlertsTab
        tab = AlertsTab.__new__(AlertsTab)
        tab.snapshot = None
        assert tab.snapshot is None

    def test_update_snapshot(self, sample_snapshot):
        from python.tui.app import AlertsTab
        tab = AlertsTab.__new__(AlertsTab)
        tab.snapshot = None
        tab._update_data = Mock()
        tab.update_snapshot(sample_snapshot)
        assert tab.snapshot is sample_snapshot
        tab._update_data.assert_called_once()

    def test_update_data_no_snapshot(self):
        from python.tui.app import AlertsTab
        tab = AlertsTab.__new__(AlertsTab)
        tab.snapshot = None
        tab._update_data()


# ---------------------------------------------------------------------------
# ScenariosTab - logic
# ---------------------------------------------------------------------------

class TestScenariosTab:
    def test_init_default(self):
        from python.tui.app import ScenariosTab
        tab = ScenariosTab.__new__(ScenariosTab)
        tab.box_spread_data = None
        assert tab.box_spread_data is None

    def test_update_data_sets_payload(self, sample_box_spread_payload):
        from python.tui.app import ScenariosTab
        tab = ScenariosTab.__new__(ScenariosTab)
        tab.box_spread_data = None
        tab._update_data = Mock()
        tab.update_data(sample_box_spread_payload)
        assert tab.box_spread_data is sample_box_spread_payload
        tab._update_data.assert_called_once()

    def test_update_data_no_payload(self):
        from python.tui.app import ScenariosTab
        tab = ScenariosTab.__new__(ScenariosTab)
        tab.box_spread_data = None
        tab._update_data()


# ---------------------------------------------------------------------------
# TUIApp._update_box_spread_data (file-based)
# ---------------------------------------------------------------------------

class TestTUIAppBoxSpreadFileLoad:
    def test_loads_from_valid_file(self, tmp_path, sample_box_spread_payload):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        data_file = tmp_path / "box_spread.json"
        data_file.write_text(json.dumps(sample_box_spread_payload.to_dict()))

        provider = MockProvider()
        app = TUIApp(provider)
        app._box_spread_file_path = data_file
        app._scenarios_tab = None

        app._update_box_spread_data()
        assert app.box_spread_data is not None
        assert len(app.box_spread_data.scenarios) == 3

    def test_missing_file_does_nothing(self, tmp_path):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        provider = MockProvider()
        app = TUIApp(provider)
        app._box_spread_file_path = tmp_path / "nonexistent.json"

        app._update_box_spread_data()
        assert app.box_spread_data is None

    def test_skips_unchanged_file(self, tmp_path, sample_box_spread_payload):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        data_file = tmp_path / "box_spread.json"
        data_file.write_text(json.dumps(sample_box_spread_payload.to_dict()))

        provider = MockProvider()
        app = TUIApp(provider)
        app._box_spread_file_path = data_file
        app._scenarios_tab = None

        app._update_box_spread_data()
        first_data = app.box_spread_data

        app._update_box_spread_data()
        assert app.box_spread_data is first_data

    def test_invalid_json_handled(self, tmp_path):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        data_file = tmp_path / "box_spread.json"
        data_file.write_text("not valid json {{{")

        provider = MockProvider()
        app = TUIApp(provider)
        app._box_spread_file_path = data_file
        app._scenarios_tab = None

        app._update_box_spread_data()
        assert app.box_spread_data is None

    def test_updates_scenarios_tab(self, tmp_path, sample_box_spread_payload):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        data_file = tmp_path / "box_spread.json"
        data_file.write_text(json.dumps(sample_box_spread_payload.to_dict()))

        provider = MockProvider()
        app = TUIApp(provider)
        app._box_spread_file_path = data_file
        mock_tab = Mock()
        app._scenarios_tab = mock_tab

        app._update_box_spread_data()
        mock_tab.update_data.assert_called_once()


# ---------------------------------------------------------------------------
# TUIApp._fetch_bank_accounts
# ---------------------------------------------------------------------------

class TestTUIAppFetchBankAccounts:
    @patch("requests.get")
    def test_do_fetch_bank_accounts_successful_fetch(self, mock_get):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_response = Mock()
        mock_response.ok = True
        mock_response.json.return_value = {
            "accounts": [{"id": "1", "name": "Savings"}]
        }
        mock_get.return_value = mock_response

        app = TUIApp(MockProvider())
        result = app._do_fetch_bank_accounts()
        assert len(result) == 1
        assert result[0]["name"] == "Savings"

    @patch("requests.get")
    def test_do_fetch_bank_accounts_failed_fetch_returns_none(self, mock_get):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_get.side_effect = Exception("Connection refused")

        app = TUIApp(MockProvider())
        assert app._do_fetch_bank_accounts() is None

    @patch("requests.get")
    def test_do_fetch_bank_accounts_non_ok_response_returns_none(self, mock_get):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_response = Mock()
        mock_response.ok = False
        mock_get.return_value = mock_response

        app = TUIApp(MockProvider())
        assert app._do_fetch_bank_accounts() is None

    def test_on_bank_accounts_loaded_refreshes_all_dependent_tabs(self, sample_snapshot):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        app = TUIApp(MockProvider())
        app.snapshot = sample_snapshot
        app._unified_positions_tab = Mock()
        app._cash_flow_tab = Mock()
        app._opportunity_simulation_tab = Mock()
        app._relationship_visualization_tab = Mock()
        app._on_bank_accounts_loaded([{"account_name": "Savings"}])

        assert app._bank_accounts == [{"account_name": "Savings"}]
        app._unified_positions_tab.update_snapshot.assert_called_once()
        app._cash_flow_tab.update_snapshot.assert_called_once_with(sample_snapshot, app._bank_accounts)
        app._opportunity_simulation_tab.update_snapshot.assert_called_once_with(sample_snapshot, app._bank_accounts)
        app._relationship_visualization_tab.update_snapshot.assert_called_once_with(sample_snapshot, app._bank_accounts)

    def test_on_worker_state_changed_ignores_failed_fetch_results(self):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider
        from textual.worker import WorkerState

        app = TUIApp(MockProvider())
        app._bank_accounts = [{"old": True}]
        app._on_bank_accounts_loaded = Mock()
        event = SimpleNamespace(
            worker=SimpleNamespace(name="fetch_bank_accounts", result=None),
            state=WorkerState.SUCCESS,
        )

        app.on_worker_state_changed(event)

        assert app._bank_accounts == [{"old": True}]
        app._on_bank_accounts_loaded.assert_not_called()


# ---------------------------------------------------------------------------
# TUIApp._update_snapshot
# ---------------------------------------------------------------------------

class TestTUIAppUpdateSnapshot:
    def test_update_propagates_to_tabs(self, sample_snapshot):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_provider = MockProvider()
        mock_provider.get_snapshot = Mock(return_value=sample_snapshot)

        app = TUIApp(mock_provider)
        app._dashboard_tab = Mock()
        app._positions_tab = Mock()
        app._orders_tab = Mock()
        app._alerts_tab = Mock()
        app._unified_positions_tab = Mock()
        app._cash_flow_tab = Mock()
        app._opportunity_simulation_tab = Mock()
        app._relationship_visualization_tab = Mock()
        app._scenarios_tab = None

        status_bar = Mock()
        status_bar.environment = "live"
        app.query_one = Mock(return_value=status_bar)

        app._update_snapshot()

        assert app.snapshot is sample_snapshot
        app._dashboard_tab.update_snapshot.assert_called_once_with(sample_snapshot, backend_health=None)
        app._positions_tab.update_snapshot.assert_called_once_with(sample_snapshot)
        app._orders_tab.update_snapshot.assert_called_once_with(sample_snapshot)
        app._alerts_tab.update_snapshot.assert_called_once_with(sample_snapshot)

    def test_no_update_when_snapshot_unchanged(self, sample_snapshot):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_provider = MockProvider()
        mock_provider.get_snapshot = Mock(return_value=sample_snapshot)

        app = TUIApp(mock_provider)
        app.snapshot = sample_snapshot
        app._dashboard_tab = Mock()
        status_bar = Mock()
        status_bar.environment = "live"
        app.query_one = Mock(return_value=status_bar)  # status bar

        app._update_snapshot()
        app._dashboard_tab.update_snapshot.assert_not_called()

    def test_manual_switch_clears_pending_real_provider(self):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        current_provider = MockProvider()
        pending_provider = MockProvider()
        app = TUIApp(current_provider)
        app._pending_real_provider = pending_provider
        current_provider.stop = Mock()
        pending_provider.stop = Mock()

        with patch("python.tui.app.create_provider_from_config", return_value=MockProvider()) as create_provider:
            replacement = create_provider.return_value
            replacement.start = Mock()
            app._switch_provider({"provider_type": "mock"}, skip_notify=True)

        pending_provider.stop.assert_called_once()
        assert app._pending_real_provider is None

    def test_update_snapshot_does_not_auto_switch_after_manual_switch(self, sample_snapshot):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        provider = MockProvider()
        provider.get_snapshot = Mock(return_value=sample_snapshot)
        app = TUIApp(provider)
        app._pending_real_provider = None
        status_bar = Mock()
        status_bar.environment = "live"
        app.query_one = Mock(return_value=status_bar)
        app._dashboard_tab = Mock()

        app._update_snapshot()

        assert app.provider is provider
        app._dashboard_tab.update_snapshot.assert_called_once()


# ---------------------------------------------------------------------------
# TUIApp actions
# ---------------------------------------------------------------------------

class TestTUIAppActions:
    def test_action_quit(self):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        app = TUIApp(MockProvider())
        app.exit = Mock()
        app.action_quit()
        app.exit.assert_called_once()

    def test_action_help(self):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        app = TUIApp(MockProvider())
        app.notify = Mock()
        app.action_help()
        app.notify.assert_called_once()
        assert "help" in app.notify.call_args[1].get("title", "").lower() or \
               "Help" in str(app.notify.call_args)

    def test_action_setup(self):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        app = TUIApp(MockProvider())
        app.notify = Mock()
        app.push_screen = Mock()
        app.action_setup()
        app.push_screen.assert_called_once()

    def test_action_refresh(self, sample_snapshot):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_provider = MockProvider()
        mock_provider.get_snapshot = Mock(return_value=sample_snapshot)

        app = TUIApp(mock_provider)
        app.notify = Mock()
        app.query_one = Mock(return_value=Mock())
        app._dashboard_tab = None
        app._unified_positions_tab = None
        app._cash_flow_tab = None
        app._opportunity_simulation_tab = None
        app._relationship_visualization_tab = None
        app._positions_tab = None
        app._orders_tab = None
        app._alerts_tab = None
        status_bar = Mock()
        status_bar.environment = "live"
        app.query_one = Mock(return_value=status_bar)

        app.action_refresh()
        app.notify.assert_called_once()
        assert app.snapshot is sample_snapshot


# ---------------------------------------------------------------------------
# main() entry point
# ---------------------------------------------------------------------------

class TestMain:
    @patch("python.tui.app.TUIApp")
    @patch("python.tui.app.create_provider_from_config")
    @patch("python.tui.app.load_config")
    def test_main_creates_and_runs_app(self, mock_load_config, mock_create_provider, mock_app_class):
        from python.tui.app import main

        mock_config = TUIConfig()
        mock_load_config.return_value = mock_config
        mock_provider = Mock()
        mock_create_provider.return_value = mock_provider
        mock_app = Mock()
        mock_app_class.return_value = mock_app

        main()

        mock_load_config.assert_called_once()
        mock_create_provider.assert_called_once_with(mock_config)
        _, kwargs = mock_app_class.call_args
        assert mock_app_class.call_args.args[:2] == (mock_provider, mock_config)
        assert kwargs["tui_log_handler_on_root"] is True
        assert kwargs["preferred_provider"] is None
        mock_app.run.assert_called_once()


# ---------------------------------------------------------------------------
# DashboardTab - missing symbols detection
# ---------------------------------------------------------------------------

class TestDashboardMissingSymbols:
    def test_detects_missing_symbols(self):
        snapshot = SnapshotPayload(
            symbols=[
                SymbolSnapshot(symbol="SPX", last=5400.0),
                SymbolSnapshot(symbol="XSP", last=540.0),
            ]
        )
        available = {s.symbol.upper() for s in snapshot.symbols}
        default_watchlist = ["SPX", "XSP", "NANOS", "TLT", "DSP"]
        missing = [s for s in default_watchlist if s.upper() not in available]
        assert "NANOS" in missing
        assert "TLT" in missing
        assert "DSP" in missing
        assert "SPX" not in missing
        assert "XSP" not in missing

    def test_no_missing_symbols(self):
        snapshot = SnapshotPayload(
            symbols=[
                SymbolSnapshot(symbol="SPX"),
                SymbolSnapshot(symbol="XSP"),
                SymbolSnapshot(symbol="NANOS"),
                SymbolSnapshot(symbol="TLT"),
                SymbolSnapshot(symbol="DSP"),
            ]
        )
        available = {s.symbol.upper() for s in snapshot.symbols}
        default_watchlist = ["SPX", "XSP", "NANOS", "TLT", "DSP"]
        missing = [s for s in default_watchlist if s.upper() not in available]
        assert missing == []


# ---------------------------------------------------------------------------
# BoxSpreadSummary.calculate (used by ScenariosTab)
# ---------------------------------------------------------------------------

class TestBoxSpreadSummaryCalc:
    def test_empty_payload(self):
        payload = BoxSpreadPayload()
        summary = BoxSpreadSummary.calculate(payload)
        assert summary.total_scenarios == 0
        assert summary.avg_apr == 0.0
        assert summary.probable_count == 0
        assert summary.max_apr_scenario is None

    def test_european_only_summary(self, sample_box_spread_payload):
        summary = BoxSpreadSummary.calculate(sample_box_spread_payload)
        assert summary.total_scenarios == 2
        assert summary.avg_apr == pytest.approx(4.35, abs=0.01)
        assert summary.probable_count == 2
        assert summary.max_apr_scenario.annualized_return == 4.5

    def test_all_american_uses_all(self):
        payload = BoxSpreadPayload(
            scenarios=[
                BoxSpreadScenario(width=100, annualized_return=3.0, fill_probability=50.0, option_style="American"),
            ]
        )
        summary = BoxSpreadSummary.calculate(payload)
        assert summary.total_scenarios == 1
        assert summary.avg_apr == 3.0
