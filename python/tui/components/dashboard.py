"""Dashboard tab: symbols table and metrics."""

from __future__ import annotations


from textual.containers import Vertical
from textual.widgets import Label, DataTable
from textual.app import ComposeResult

from .base import SnapshotTabBase


class DashboardTab(SnapshotTabBase):
    """Dashboard tab showing symbols and metrics."""

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

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        default_watchlist = ["SPX", "XSP", "NANOS", "TLT", "DSP"]
        available_symbols = {s.symbol.upper() for s in self.snapshot.symbols}
        missing_symbols = [s for s in default_watchlist if s.upper() not in available_symbols]

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
