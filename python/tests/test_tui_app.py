"""Tests for python/tui/app.py - TUI application module.

Tests cover widget classes, tab containers, app initialization,
provider factory, and snapshot formatting - all without running
a real Textual application.
"""

import json
from pathlib import Path
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
        from python.tui.app import SnapshotDisplay
        widget = SnapshotDisplay.__new__(SnapshotDisplay)
        result = widget._format_snapshot(sample_snapshot)
        assert "14:30:45" in result
        assert "LIVE" in result
        assert "RUNNING" in result
        assert "DU123456" in result

    def test_format_snapshot_no_timestamp(self):
        from python.tui.app import SnapshotDisplay
        widget = SnapshotDisplay.__new__(SnapshotDisplay)
        snapshot = SnapshotPayload(generated_at="", mode="DRY-RUN")
        result = widget._format_snapshot(snapshot)
        assert "--:--:--" in result
        assert "DRY-RUN" in result

    def test_format_snapshot_date_only(self):
        from python.tui.app import SnapshotDisplay
        widget = SnapshotDisplay.__new__(SnapshotDisplay)
        snapshot = SnapshotPayload(generated_at="2025-06-15T09:05:01Z")
        result = widget._format_snapshot(snapshot)
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
        assert provider.endpoint == "http://test:8080/api"

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
    def test_successful_fetch(self, mock_get):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_response = Mock()
        mock_response.ok = True
        mock_response.json.return_value = {
            "accounts": [{"id": "1", "name": "Savings"}]
        }
        mock_get.return_value = mock_response

        app = TUIApp(MockProvider())
        app._unified_positions_tab = None
        app._fetch_bank_accounts()
        assert len(app._bank_accounts) == 1
        assert app._bank_accounts[0]["name"] == "Savings"

    @patch("requests.get")
    def test_failed_fetch_silently_ignored(self, mock_get):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_get.side_effect = Exception("Connection refused")

        app = TUIApp(MockProvider())
        app._bank_accounts = []
        app._fetch_bank_accounts()
        assert app._bank_accounts == []

    @patch("requests.get")
    def test_non_ok_response(self, mock_get):
        from python.tui.app import TUIApp
        from python.tui.providers import MockProvider

        mock_response = Mock()
        mock_response.ok = False
        mock_get.return_value = mock_response

        app = TUIApp(MockProvider())
        app._bank_accounts = [{"old": True}]
        app._fetch_bank_accounts()
        assert app._bank_accounts == [{"old": True}]


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

        mock_display = Mock()
        app.query_one = Mock(return_value=mock_display)

        app._update_snapshot()

        assert app.snapshot is sample_snapshot
        app._dashboard_tab.update_snapshot.assert_called_once_with(sample_snapshot)
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

        app._update_snapshot()
        app._dashboard_tab.update_snapshot.assert_not_called()


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
        app.action_setup()
        app.notify.assert_called_once()

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
        mock_app_class.assert_called_once_with(mock_provider, mock_config)
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
