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
from typing import Optional

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

from .models import SnapshotPayload, Severity
from .providers import Provider, MockProvider, RestProvider, FileProvider
from .config import TUIConfig, load_config

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
        self._dashboard_tab: Optional[DashboardTab] = None
        self._positions_tab: Optional[PositionsTab] = None
        self._orders_tab: Optional[OrdersTab] = None
        self._alerts_tab: Optional[AlertsTab] = None

    def compose(self) -> ComposeResult:
        """Create child widgets for the app"""
        yield Header(show_clock=True)

        with Container(id="main-container"):
            yield SnapshotDisplay(id="snapshot-display")

            with TabbedContent(id="tabs"):
                with TabPane("Dashboard", id="dashboard-tab"):
                    self._dashboard_tab = DashboardTab()
                    yield self._dashboard_tab

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

        yield Footer()

    def on_mount(self) -> None:
        """Called when app starts"""
        self.provider.start()
        self.set_interval(0.5, self._update_snapshot)  # Update every 500ms
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
            if self._positions_tab:
                self._positions_tab.update_snapshot(new_snapshot)
            if self._orders_tab:
                self._orders_tab.update_snapshot(new_snapshot)
            if self._alerts_tab:
                self._alerts_tab.update_snapshot(new_snapshot)

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
        endpoint = config.rest_endpoint or "http://localhost:8080/api/snapshot"
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
