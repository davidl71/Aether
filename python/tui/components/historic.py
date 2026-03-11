"""Historic positions tab."""

from __future__ import annotations

from textual.containers import Vertical
from textual.widgets import DataTable, Label
from textual.app import ComposeResult

from .base import SnapshotTabBase


def _fmt(value: float | None, decimals: int = 2, prefix: str = "") -> str:
    """Format numeric values consistently for the historic positions table."""
    if value is None:
        return "—"
    if decimals == 0:
        if value < 0 and prefix:
            return f"-{prefix}{int(-value):,}"
        return f"{prefix}{int(value):,}"
    if value < 0 and prefix:
        return f"-{prefix}{-value:,.{decimals}f}"
    return f"{prefix}{value:,.{decimals}f}"


class HistoricTab(SnapshotTabBase):
    """Historic positions tab showing closed or archived positions when present."""

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Historic Positions", classes="tab-title")
            yield Label("", id="historic-message")
            yield DataTable(id="historic-table")

    def on_mount(self) -> None:
        table = self.query_one("#historic-table", DataTable)
        table.add_columns(
            "Name",
            "Qty",
            "ROI",
            "Market value",
            "Rate",
            "Maturity",
            "Cash flow",
            "Currency",
        )
        self._update_data()

    def _update_data(self) -> None:
        msg = self.query_one("#historic-message", Label)
        table = self.query_one("#historic-table", DataTable)
        table.clear()

        if not self.snapshot or not self.snapshot.historic:
            msg.update("No historic positions available.")
            table.display = False
            return

        table.display = True
        msg.update(f"{len(self.snapshot.historic)} historic position(s)")

        for pos in self.snapshot.historic:
            table.add_row(
                pos.name or "—",
                str(pos.quantity),
                _fmt(pos.roi, 2, "") + "%",
                _fmt(pos.market_value, 2, "$"),
                _fmt(pos.rate, 2, "") + "%" if pos.rate is not None else "—",
                pos.maturity_date or "—",
                _fmt(pos.cash_flow, 2, "$"),
                pos.currency or "USD",
            )
