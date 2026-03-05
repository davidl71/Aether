"""
Main TUI application using Textual

This is the Python replacement for the C++ TUI (native/src/tui_app.cpp).
It provides the same functionality with better performance and easier maintenance.

MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
- UI rendering logic can stay in Python (Textual is Python-only)
- Data processing can be moved to C++ and exposed via pybind11
- Consider keeping Python TUI as reference implementation
- Provider classes can be C++ implementations wrapped in Python
- Keyboard shortcuts and event handling can remain in Python
"""

from __future__ import annotations

import logging
import sys
from pathlib import Path
from typing import Optional, List, Dict, Any

from textual.app import App, ComposeResult
from textual.containers import Container
from textual.widgets import (
    Header,
    Footer,
    TabbedContent,
    TabPane,
)
from textual.binding import Binding

from .models import SnapshotPayload, BoxSpreadPayload
from .providers import Provider, MockProvider, RestProvider, FileProvider, BackendHealthAggregator
from .config import TUIConfig, load_config
from .box_spread_loader import get_box_spread_payload
from .log_handler import install_tui_log_handler, remove_tui_log_handler, drain_log_queue, get_buffered_log_lines
from .components.snapshot_display import StatusBar
from .components.dashboard import DashboardTab
from .components.positions import PositionsTab
from .components.orders import OrdersTab
from .components.alerts import AlertsTab
from .components.scenarios import ScenariosTab
from .components.historic import HistoricTab
from .components.unified_positions import UnifiedPositionsTab
from .components.cash_flow import CashFlowTab
from .components.opportunity_simulation import OpportunitySimulationTab
from .components.relationship_visualization import RelationshipVisualizationTab
from .components.loan_entry import LoanListTab, LoanManager
from .components.setup_screen import SetupScreen
from .components.logs_tab import LogsTab

logger = logging.getLogger(__name__)


class TUIApp(App):
    """
    Main TUI application

    MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
    - This class structure can be mirrored in C++ using FTXUI
    - Provider can be C++ class exposed via pybind11
    - Consider keeping Python version as reference during migration
    - Event handling and UI updates can remain in Python
    """

    CSS = """
    Screen {
        background: $surface;
    }

    #status-bar {
        dock: bottom;
        height: 1;
        padding: 0 1;
        background: $surface-darken-2;
        color: $text;
    }

    .tab-title {
        text-style: bold;
        color: $accent;
        margin: 1;
    }

    #symbols-table, #positions-table {
        height: 1fr;
    }

    #metrics-label {
        margin: 1;
        text-style: dim;
    }

    .position-group-header {
        text-style: bold;
        color: $accent;
        margin: 1 0;
    }

    #missing-symbols-label.warning {
        color: $warning;
        text-style: bold;
        margin: 1;
        padding: 1;
        background: $surface-darken-1;
    }

    .logs-tab-header {
        color: $text-muted;
        margin: 0 0 1 0;
    }
    """

    TITLE = "IB Box Spread Terminal"
    BINDINGS = [
        Binding("q", "quit", "Quit"),
        Binding("f1", "help", "Help"),
        Binding("f2", "setup", "Setup"),
        Binding("f5", "refresh", "Refresh"),
        Binding("f10", "quit", "Quit"),
    ]

    def __init__(
        self,
        provider: Provider,
        config: Optional[TUIConfig] = None,
        tui_log_handler: Optional[logging.Handler] = None,
        tui_log_handler_on_root: bool = False,
    ):
        super().__init__()
        self.provider = provider
        self.config = config or TUIConfig()
        self._tui_log_handler = tui_log_handler
        self._tui_log_handler_on_root = tui_log_handler_on_root
        self.snapshot: Optional[SnapshotPayload] = None
        self.box_spread_data: Optional[BoxSpreadPayload] = None
        self._dashboard_tab: Optional[DashboardTab] = None
        self._unified_positions_tab: Optional[UnifiedPositionsTab] = None
        self._cash_flow_tab: Optional[CashFlowTab] = None
        self._opportunity_simulation_tab: Optional[OpportunitySimulationTab] = None
        self._relationship_visualization_tab: Optional[RelationshipVisualizationTab] = None
        self._positions_tab: Optional[PositionsTab] = None
        self._orders_tab: Optional[OrdersTab] = None
        self._alerts_tab: Optional[AlertsTab] = None
        self._scenarios_tab: Optional[ScenariosTab] = None
        self._historic_tab: Optional[HistoricTab] = None
        self._loan_tab: Optional[LoanListTab] = None
        self._logs_tab: Optional[LogsTab] = None
        self._logs_buffer_loaded = False
        self._box_spread_file_path = Path("web/public/data/box_spread_sample.json")
        self._last_box_spread_mtime: Optional[float] = None
        self._bank_accounts: List[Dict] = []
        self._loan_manager = LoanManager("config/loans.json")
        self._backend_health_aggregator: Optional[BackendHealthAggregator] = None

    def compose(self) -> ComposeResult:
        """Create child widgets for the app"""
        yield Header(show_clock=True)

        with Container(id="main-container"):
            with TabbedContent(id="tabs"):
                with TabPane("Dashboard", id="dashboard-tab"):
                    self._dashboard_tab = DashboardTab()
                    yield self._dashboard_tab

                with TabPane("Unified Positions", id="unified-positions-tab"):
                    self._unified_positions_tab = UnifiedPositionsTab()
                    yield self._unified_positions_tab

                with TabPane("Cash Flow", id="cash-flow-tab"):
                    self._cash_flow_tab = CashFlowTab()
                    yield self._cash_flow_tab

                with TabPane("Simulation", id="simulation-tab"):
                    self._opportunity_simulation_tab = OpportunitySimulationTab()
                    yield self._opportunity_simulation_tab

                with TabPane("Relationships", id="relationships-tab"):
                    self._relationship_visualization_tab = RelationshipVisualizationTab()
                    yield self._relationship_visualization_tab

                with TabPane("Positions", id="positions-tab"):
                    self._positions_tab = PositionsTab()
                    yield self._positions_tab

                with TabPane("Historic", id="historic-tab"):
                    self._historic_tab = HistoricTab()
                    yield self._historic_tab

                with TabPane("Orders", id="orders-tab"):
                    self._orders_tab = OrdersTab()
                    yield self._orders_tab

                with TabPane("Alerts", id="alerts-tab"):
                    self._alerts_tab = AlertsTab()
                    yield self._alerts_tab

                with TabPane("Scenarios", id="scenarios-tab"):
                    self._scenarios_tab = ScenariosTab()
                    yield self._scenarios_tab

                with TabPane("Loans", id="loans-tab"):
                    self._loan_tab = LoanListTab(self._loan_manager)
                    yield self._loan_tab

                with TabPane("Logs", id="logs-tab"):
                    self._logs_tab = LogsTab(max_lines=500)
                    yield self._logs_tab

        yield StatusBar(id="status-bar")
        yield Footer()

    def on_mount(self) -> None:
        """Called when app starts"""
        if self._tui_log_handler is None:
            self._tui_log_handler = install_tui_log_handler(level=logging.DEBUG)
            self._tui_log_handler_on_root = False
        self.provider.start()
        if self.config.backend_ports:
            self._backend_health_aggregator = BackendHealthAggregator(self.config.backend_ports)
            self._backend_health_aggregator.start()
        self.set_interval(0.5, self._update_snapshot)  # Update every 500ms
        self.set_interval(0.25, self._drain_tui_logs)  # Drain log queue for Logs tab
        self.set_interval(2.0, self._update_box_spread_data)  # Update every 2 seconds
        self.set_interval(30.0, self._fetch_bank_accounts)  # Update bank accounts every 30 seconds
        self._fetch_bank_accounts()  # Initial fetch
        logger.info("TUI application mounted")

    def on_unmount(self) -> None:
        """Called when app exits"""
        if self._backend_health_aggregator:
            self._backend_health_aggregator.stop()
            self._backend_health_aggregator = None
        if self._tui_log_handler:
            remove_tui_log_handler(self._tui_log_handler, from_root=self._tui_log_handler_on_root)
            self._tui_log_handler = None
        self.provider.stop()
        logger.info("TUI application unmounted")

    def _get_provider_label(self) -> str:
        """Return short label for current data provider and endpoint (e.g. 'rest (127.0.0.1:8002)', 'mock')."""
        from urllib.parse import urlparse
        if isinstance(self.provider, RestProvider):
            try:
                parsed = urlparse(self.provider.endpoint)
                netloc = parsed.netloc or parsed.path.split("/")[0]
                return f"rest ({netloc})"
            except Exception:
                return "rest"
        if isinstance(self.provider, FileProvider):
            path = getattr(self.provider, "file_path", None)
            name = path.name if path else "file"
            return f"file ({name})"
        return "mock"

    def _update_snapshot(self) -> None:
        """Update snapshot from provider"""
        new_snapshot = self.provider.get_snapshot()
        status_bar = self.query_one("#status-bar", StatusBar)
        status_bar.provider_label = self._get_provider_label()
        # All backend health: from aggregator if present, else single backend from current provider
        all_health: Dict[str, Dict[str, Any]] = {}
        if self._backend_health_aggregator:
            all_health = self._backend_health_aggregator.get_all_health()
        # Overlay disabled backends (e.g. missing API keys) so TUI shows "disabled" instead of unreachable
        for name, reason in getattr(self.config, "disabled_backends", {}).items():
            all_health[name] = {"status": "disabled", "error": reason}
        if not all_health and hasattr(self.provider, "get_health"):
            h = self.provider.get_health()
            if h is not None:
                all_health["current"] = h
        # Show "checking..." when aggregator is running but no health data yet
        if not all_health and self._backend_health_aggregator:
            all_health = {"connection": {"status": "checking", "error": "polling..."}}
        status_bar.backend_health = all_health if all_health else None
        if new_snapshot != self.snapshot:
            self.snapshot = new_snapshot
            status_bar.snapshot = new_snapshot

            # Update tabs
            if self._dashboard_tab:
                self._dashboard_tab.update_snapshot(new_snapshot)
            if self._unified_positions_tab:
                self._unified_positions_tab.update_snapshot(new_snapshot, self._bank_accounts)
            if self._cash_flow_tab:
                self._cash_flow_tab.update_snapshot(new_snapshot, self._bank_accounts)
            if self._opportunity_simulation_tab:
                self._opportunity_simulation_tab.update_snapshot(new_snapshot, self._bank_accounts)
            if self._relationship_visualization_tab:
                self._relationship_visualization_tab.update_snapshot(new_snapshot, self._bank_accounts)
            if self._positions_tab:
                self._positions_tab.update_snapshot(new_snapshot)
            if self._orders_tab:
                self._orders_tab.update_snapshot(new_snapshot)
            if self._alerts_tab:
                self._alerts_tab.update_snapshot(new_snapshot)
            if self._historic_tab:
                self._historic_tab.update_snapshot(new_snapshot)

    def _drain_tui_logs(self) -> None:
        """Drain TUI log queue and append to Logs tab."""
        if not self._logs_tab:
            return
        if not self._logs_buffer_loaded:
            self._logs_buffer_loaded = True
            buf = get_buffered_log_lines()
            if buf:
                self._logs_tab.load_buffer(buf)
        lines = drain_log_queue()
        if lines:
            self._logs_tab.write_lines(lines)

    def _fetch_bank_accounts(self) -> None:
        """Fetch bank accounts from Discount Bank service"""
        try:
            import requests
            response = requests.get(
                "http://localhost:8003/api/bank-accounts",
                timeout=2.0,
                headers={"cache-control": "no-cache"}
            )
            if response.ok:
                data = response.json()
                self._bank_accounts = data.get("accounts", [])
                # Update unified positions tab if it exists
                if self._unified_positions_tab and self.snapshot:
                    self._unified_positions_tab.update_snapshot(self.snapshot, self._bank_accounts)
        except Exception as e:
            logger.debug(f"Failed to fetch bank accounts: {e}")
            # Silently fail - bank accounts are optional

    def _update_box_spread_data(self) -> None:
        """Update box spread data from REST or file via loader."""
        payload, new_mtime = get_box_spread_payload(
            self.config,
            self._box_spread_file_path,
            self._last_box_spread_mtime,
        )
        if new_mtime is not None:
            self._last_box_spread_mtime = new_mtime
        if payload is None:
            return
        if payload != self.box_spread_data:
            self.box_spread_data = payload
            if self._scenarios_tab:
                self._scenarios_tab.update_data(payload)

    def action_quit(self) -> None:
        """Quit the application"""
        self.exit()

    def action_help(self) -> None:
        """Show help screen"""
        self.notify("Press F1 for help, F2 for setup, Q/F10 to quit", title="Help")

    def action_setup(self) -> None:
        """Show setup screen"""
        self.push_screen(
            SetupScreen(
                self.config,
                self._get_provider_label(),
                str(self._loan_manager.loans_file_path),
            ),
            self._on_setup_closed,
        )

    def _on_setup_closed(self, result: Optional[dict]) -> None:
        """Handle setup screen dismiss: apply backend switch if user clicked Apply."""
        if result and isinstance(result, dict) and result.get("provider_type"):
            self._switch_provider(result)

    def _switch_provider(self, params: Dict[str, Optional[str]]) -> None:
        """Replace current provider with a new one from params."""
        ptype = (params.get("provider_type") or "mock").lower()
        rest_endpoint = params.get("rest_endpoint")
        file_path = params.get("file_path")
        self.provider.stop()
        temp_config = TUIConfig(
            provider_type=ptype,
            rest_endpoint=rest_endpoint or self.config.rest_endpoint,
            file_path=file_path or self.config.file_path,
            update_interval_ms=self.config.update_interval_ms,
            rest_timeout_ms=self.config.rest_timeout_ms,
        )
        self.provider = create_provider_from_config(temp_config)
        self.config.provider_type = ptype
        if rest_endpoint is not None:
            self.config.rest_endpoint = rest_endpoint
        if file_path is not None:
            self.config.file_path = file_path
        self.provider.start()
        label = self._get_provider_label()
        self.notify(f"Backend switched to {label}", title="Switch backend")

    def action_refresh(self) -> None:
        """Force refresh snapshot"""
        self._update_snapshot()
        self.notify("Refreshed", title="Refresh")


def create_provider_from_config(config: TUIConfig) -> Provider:
    """
    Create provider based on configuration

    MIGRATION NOTE: This factory function can call C++ provider constructors
    via pybind11 in the future
    """
    provider_type = config.provider_type.lower()

    if provider_type == "mock" or not provider_type:
        return MockProvider(update_interval_ms=config.update_interval_ms)

    elif provider_type == "rest":
        endpoint = config.rest_endpoint or "http://localhost:8002/api/v1/snapshot"
        return RestProvider(
            endpoint=endpoint,
            update_interval_ms=config.update_interval_ms,
            timeout_ms=config.rest_timeout_ms,
            verify_ssl=config.rest_verify_ssl,
        )

    elif provider_type == "file":
        file_path = config.file_path or "web/public/data/snapshot.json"
        return FileProvider(
            file_path=file_path, update_interval_ms=config.update_interval_ms
        )

    else:
        logger.warning(f"Unknown provider type: {provider_type}, using mock")
        return MockProvider(update_interval_ms=config.update_interval_ms)


def main():
    """Main entry point for Python TUI"""

    # Logging to stderr (stdout reserved for GUI/normal output); logs also go to TUI Logs tab
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        stream=sys.stderr,
    )
    tui_handler = install_tui_log_handler(level=logging.INFO, attach_to_root=True)

    # Load configuration
    config = load_config()

    # Create provider
    provider = create_provider_from_config(config)

    # Create and run app (pass handler so it can be removed from root on unmount)
    app = TUIApp(provider, config, tui_log_handler=tui_handler, tui_log_handler_on_root=True)
    app.run()


if __name__ == "__main__":
    main()
