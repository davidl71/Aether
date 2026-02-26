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

import json
import logging
import os
from pathlib import Path
from typing import Optional, List, Dict

from textual.app import App, ComposeResult
from textual.containers import Container, Horizontal, Vertical
from textual.widgets import (
    Header,
    Footer,
    Static,
    DataTable,
    TabbedContent,
    TabPane,
    Label,
    Log,
    Button,
)
from textual.binding import Binding
from textual.reactive import reactive

from .models import SnapshotPayload, Severity, BoxSpreadPayload, BoxSpreadSummary
from .providers import Provider, MockProvider, RestProvider, FileProvider
from .config import TUIConfig, load_config
from .components.unified_positions import UnifiedPositionsTab
from .components.cash_flow import CashFlowTab
from .components.opportunity_simulation import OpportunitySimulationTab
from .components.relationship_visualization import RelationshipVisualizationTab
from .components.loan_entry import LoanListTab, LoanManager

logger = logging.getLogger(__name__)


class SnapshotDisplay(Static):
    """Widget that displays snapshot data reactively"""

    snapshot: reactive[Optional[SnapshotPayload]] = reactive(None)

    def watch_snapshot(self, snapshot: Optional[SnapshotPayload]) -> None:
        """Called when snapshot changes"""
        if snapshot:
            self.update(self._format_snapshot(snapshot))
        else:
            self.update("Waiting for data...")

    def _format_snapshot(self, snapshot: SnapshotPayload) -> str:
        """Format snapshot for display"""
        time_str = (
            snapshot.generated_at.split("T")[1].split(".")[0]
            if snapshot.generated_at
            else "--:--:--"
        )
        return f"Time: {time_str} | Mode: {snapshot.mode} | Strategy: {snapshot.strategy} | Account: {snapshot.account_id}"


class DashboardTab(Container):
    """Dashboard tab showing symbols and metrics"""

    def __init__(self, snapshot: Optional[SnapshotPayload] = None):
        super().__init__()
        self.snapshot = snapshot

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Dashboard", classes="tab-title")
            yield Label(id="missing-symbols-label")
            yield DataTable(id="symbols-table")
            yield Label(id="metrics-label")

    def on_mount(self) -> None:
        table = self.query_one("#symbols-table", DataTable)
        table.add_columns("Symbol", "Last", "Bid", "Ask", "Spread", "ROI%")
        self._update_data()

    def update_snapshot(self, snapshot: SnapshotPayload) -> None:
        """Update with new snapshot data"""
        self.snapshot = snapshot
        self._update_data()

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        # Default watchlist symbols (matching PWA)
        default_watchlist = ['SPX', 'XSP', 'NANOS', 'TLT', 'DSP']

        # Find missing symbols
        available_symbols = {s.symbol.upper() for s in self.snapshot.symbols}
        missing_symbols = [s for s in default_watchlist if s.upper() not in available_symbols]

        # Update missing symbols label
        missing_label = self.query_one("#missing-symbols-label", Label)
        if missing_symbols:
            missing_label.update(
                f"⚠️ Note: The following symbols are in your watchlist but not available in the current snapshot: {', '.join(missing_symbols)}"
            )
            missing_label.add_class("warning")
        else:
            missing_label.update("")
            missing_label.remove_class("warning")

        table = self.query_one("#symbols-table", DataTable)
        table.clear()

        for symbol in self.snapshot.symbols:
            table.add_row(
                symbol.symbol,
                f"{symbol.last:.2f}" if symbol.last > 0 else "--",
                f"{symbol.bid:.2f}" if symbol.bid > 0 else "--",
                f"{symbol.ask:.2f}" if symbol.ask > 0 else "--",
                f"{symbol.spread:.2f}" if symbol.spread > 0 else "--",
                f"{symbol.roi:.2f}" if symbol.roi > 0 else "--",
            )

        metrics = self.snapshot.metrics
        metrics_label = self.query_one("#metrics-label", Label)
        metrics_label.update(
            f"Positions: {len(self.snapshot.positions)} | "
            f"Orders: {len(self.snapshot.orders)} | "
            f"Alerts: {len(self.snapshot.alerts)} | "
            f"Net Liq: ${metrics.net_liq:,.0f}"
        )


class PositionsTab(Container):
    """Positions tab showing current positions"""

    def __init__(self, snapshot: Optional[SnapshotPayload] = None):
        super().__init__()
        self.snapshot = snapshot

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Current Positions", classes="tab-title")
            yield DataTable(id="positions-table")

    def on_mount(self) -> None:
        table = self.query_one("#positions-table", DataTable)
        table.add_columns("Name", "Qty", "ROI%", "Mk/Tk", "Rebate", "Vega", "Theta")
        self._update_data()

    def update_snapshot(self, snapshot: SnapshotPayload) -> None:
        """Update with new snapshot data"""
        self.snapshot = snapshot
        self._update_data()

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        table = self.query_one("#positions-table", DataTable)
        table.clear()

        for pos in self.snapshot.positions:
            table.add_row(
                pos.name,
                str(pos.quantity),
                f"{pos.roi:.2f}",
                f"{pos.maker_count}/{pos.taker_count}",
                f"{pos.rebate_estimate:.2f}",
                f"{pos.vega:.2f}",
                f"{pos.theta:.2f}",
            )


class OrdersTab(Container):
    """Orders tab showing recent orders"""

    def __init__(self, snapshot: Optional[SnapshotPayload] = None):
        super().__init__()
        self.snapshot = snapshot

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Recent Orders", classes="tab-title")
            yield Log(id="orders-log")

    def on_mount(self) -> None:
        """Called when tab is mounted"""
        self._update_data()

    def update_snapshot(self, snapshot: SnapshotPayload) -> None:
        """Update with new snapshot data"""
        self.snapshot = snapshot
        self._update_data()

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        try:
            log = self.query_one("#orders-log", Log)
            log.clear()

            for order in self.snapshot.orders:
                time_str = (
                    order.timestamp.split("T")[1].split(".")[0]
                    if order.timestamp
                    else "--:--:--"
                )
                log.write(f"[{time_str}] {order.text}")
        except Exception:
            # Widget might not be mounted yet
            pass


class AlertsTab(Container):
    """Alerts tab showing system alerts"""

    def __init__(self, snapshot: Optional[SnapshotPayload] = None):
        super().__init__()
        self.snapshot = snapshot

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Alerts", classes="tab-title")
            yield Log(id="alerts-log")

    def on_mount(self) -> None:
        """Called when tab is mounted"""
        self._update_data()

    def update_snapshot(self, snapshot: SnapshotPayload) -> None:
        """Update with new snapshot data"""
        self.snapshot = snapshot
        self._update_data()

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        try:
            log = self.query_one("#alerts-log", Log)
            log.clear()

            for alert in self.snapshot.alerts:
                time_str = (
                    alert.timestamp.split("T")[1].split(".")[0]
                    if alert.timestamp
                    else "--:--:--"
                )
                severity_style = self._get_severity_style(alert.severity)
                log.write(
                    f"[{time_str}] [{severity_style}]{alert.text}[/]", markup=True
                )
        except Exception:
            # Widget might not be mounted yet
            pass

    def _get_severity_style(self, severity: Severity) -> str:
        """Get Textual markup style for severity"""
        styles = {
            Severity.INFO: "cyan",
            Severity.SUCCESS: "green",
            Severity.WARN: "yellow",
            Severity.WARNING: "yellow",
            Severity.ERROR: "red",
            Severity.CRITICAL: "bold red",
        }
        return styles.get(severity, "white")


class ScenariosTab(Container):
    """Scenarios tab showing box spread scenarios"""

    def __init__(self, box_spread_data: Optional[BoxSpreadPayload] = None):
        super().__init__()
        self.box_spread_data = box_spread_data

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Box Spread Scenarios", classes="tab-title")
            yield Static(id="scenario-summary")
            yield DataTable(id="scenarios-table")

    def on_mount(self) -> None:
        """Called when tab is mounted"""
        table = self.query_one("#scenarios-table", DataTable)
        table.add_columns(
            "Width", "Style", "Buy Profit", "Sell Profit", "APR %", "Fill Prob"
        )
        self._update_data()

    def update_data(self, box_spread_data: BoxSpreadPayload) -> None:
        """Update with new box spread data"""
        self.box_spread_data = box_spread_data
        self._update_data()

    def _update_data(self) -> None:
        if not self.box_spread_data:
            try:
                summary = self.query_one("#scenario-summary", Static)
                summary.update("Loading scenarios...")
            except Exception:
                pass
            return

        try:
            # Calculate summary
            summary_stats = BoxSpreadSummary.calculate(self.box_spread_data)

            # Update summary
            summary = self.query_one("#scenario-summary", Static)
            summary_text = (
                f"Total Scenarios: {summary_stats.total_scenarios} | "
                f"Average APR: {summary_stats.avg_apr:.2f}% | "
                f"Probable (fill_prob > 0): {summary_stats.probable_count}"
            )
            if summary_stats.max_apr_scenario:
                summary_text += (
                    f" | Max APR: {summary_stats.max_apr_scenario.annualized_return:.2f}% "
                    f"({summary_stats.max_apr_scenario.width:.2f} pts)"
                )
            summary.update(summary_text)

            # Update table
            table = self.query_one("#scenarios-table", DataTable)
            table.clear()

            # Filter to European-style scenarios (default behavior, matching web app)
            european_scenarios = [
                s for s in self.box_spread_data.scenarios
                if s.option_style == "European"
            ]
            scenarios_to_show = european_scenarios if european_scenarios else self.box_spread_data.scenarios

            for scenario in scenarios_to_show:
                buy_profit = scenario.buy_profit if scenario.buy_profit is not None else 0.0
                sell_profit = scenario.sell_profit if scenario.sell_profit is not None else 0.0

                table.add_row(
                    f"{scenario.width:.2f}",
                    scenario.option_style,
                    f"${buy_profit:.2f}" if buy_profit != 0.0 else "—",
                    f"${sell_profit:.2f}" if sell_profit != 0.0 else "—",
                    f"{scenario.annualized_return:.2f}%",
                    f"{scenario.fill_probability:.0f}%"
                )
        except Exception as e:
            logger.error(f"Error updating scenarios: {e}")
            try:
                summary = self.query_one("#scenario-summary", Static)
                summary.update(f"Error loading scenarios: {e}")
            except Exception:
                pass


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
    """

    TITLE = "IB Box Spread Terminal"
    BINDINGS = [
        Binding("q", "quit", "Quit"),
        Binding("f1", "help", "Help"),
        Binding("f2", "setup", "Setup"),
        Binding("f5", "refresh", "Refresh"),
        Binding("f10", "quit", "Quit"),
        Binding("tab", "next_tab", "Next Tab"),
        Binding("shift+tab", "prev_tab", "Previous Tab"),
    ]

    def __init__(self, provider: Provider, config: Optional[TUIConfig] = None):
        super().__init__()
        self.provider = provider
        self.config = config or TUIConfig()
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
        self._loan_tab: Optional[LoanListTab] = None
        self._box_spread_file_path = Path("web/public/data/box_spread_sample.json")
        self._bank_accounts: List[Dict] = []
        self._loan_manager = LoanManager("config/loans.json")

    def compose(self) -> ComposeResult:
        """Create child widgets for the app"""
        yield Header(show_clock=True)

        with Container(id="main-container"):
            yield SnapshotDisplay(id="snapshot-display")

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
                    yield Label("Historic Positions (coming soon)")

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

        yield Footer()

    def on_mount(self) -> None:
        """Called when app starts"""
        self.provider.start()
        self.set_interval(0.5, self._update_snapshot)  # Update every 500ms
        self.set_interval(2.0, self._update_box_spread_data)  # Update every 2 seconds
        self.set_interval(30.0, self._fetch_bank_accounts)  # Update bank accounts every 30 seconds
        self._fetch_bank_accounts()  # Initial fetch
        logger.info("TUI application mounted")

    def on_unmount(self) -> None:
        """Called when app exits"""
        self.provider.stop()
        logger.info("TUI application unmounted")

    def _update_snapshot(self) -> None:
        """Update snapshot from provider"""
        new_snapshot = self.provider.get_snapshot()
        if new_snapshot != self.snapshot:
            self.snapshot = new_snapshot

            # Update snapshot display
            display = self.query_one("#snapshot-display", SnapshotDisplay)
            display.snapshot = new_snapshot

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
        """Update box spread data from REST API, falling back to local file"""
        data = None

        api_url = self.config.rest_endpoint.rsplit("/", 1)[0] + "/scenarios"
        try:
            import requests as _req
            resp = _req.get(api_url, timeout=2.0, headers={"Accept": "application/json"})
            if resp.ok:
                data = resp.json()
        except Exception:
            pass

        if data is None:
            try:
                if not self._box_spread_file_path.exists():
                    return
                current_mtime = self._box_spread_file_path.stat().st_mtime
                if hasattr(self, '_last_box_spread_mtime'):
                    if current_mtime <= self._last_box_spread_mtime:
                        return
                self._last_box_spread_mtime = current_mtime
                with open(self._box_spread_file_path, 'r') as f:
                    data = json.load(f)
            except Exception as e:
                logger.error(f"Error reading box spread file: {e}")
                return

        if data is None:
            return

        try:
            new_data = BoxSpreadPayload.from_dict(data)
            if new_data != self.box_spread_data:
                self.box_spread_data = new_data
                if self._scenarios_tab:
                    self._scenarios_tab.update_data(new_data)
        except Exception as e:
            logger.error(f"Error updating box spread data: {e}")

    def action_quit(self) -> None:
        """Quit the application"""
        self.exit()

    def action_help(self) -> None:
        """Show help screen"""
        self.notify("Press F1 for help, F2 for setup, Q/F10 to quit", title="Help")

    def action_setup(self) -> None:
        """Show setup screen"""
        self.notify("Setup screen (coming soon)", title="Setup")

    def action_refresh(self) -> None:
        """Force refresh snapshot"""
        self._update_snapshot()
        self.notify("Refreshed", title="Refresh")

    def action_next_tab(self) -> None:
        """Switch to next tab"""
        tabs = self.query_one("#tabs", TabbedContent)
        # Textual handles tab switching automatically

    def action_prev_tab(self) -> None:
        """Switch to previous tab"""
        tabs = self.query_one("#tabs", TabbedContent)
        # Textual handles tab switching automatically


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
        endpoint = config.rest_endpoint or "http://localhost:8080/api/v1/snapshot"
        return RestProvider(
            endpoint=endpoint,
            update_interval_ms=config.update_interval_ms,
            timeout_ms=config.rest_timeout_ms,
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
    import sys

    # Setup logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )

    # Load configuration
    config = load_config()

    # Create provider
    provider = create_provider_from_config(config)

    # Create and run app
    app = TUIApp(provider, config)
    app.run()


if __name__ == "__main__":
    main()
