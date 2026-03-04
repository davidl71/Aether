"""Positions tab: current positions table."""

from __future__ import annotations

from textual.containers import Vertical
from textual.widgets import Label, DataTable
from textual.app import ComposeResult

from .base import SnapshotTabBase


class PositionsTab(SnapshotTabBase):
    """Positions tab showing current positions."""

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Current Positions", classes="tab-title")
            yield DataTable(id="positions-table")

    def on_mount(self) -> None:
        table = self.query_one("#positions-table", DataTable)
        table.add_columns("Name", "Qty", "ROI%", "Mk/Tk", "Rebate", "Vega", "Theta")
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
