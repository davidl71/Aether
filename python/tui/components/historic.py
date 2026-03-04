"""Historic positions tab (stub)."""

from __future__ import annotations


from textual.containers import Vertical
from textual.widgets import Label
from textual.app import ComposeResult

from .base import SnapshotTabBase


class HistoricTab(SnapshotTabBase):
    """Historic positions tab. Placeholder until historic data is available."""

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Historic Positions", classes="tab-title")
            yield Label("No historic data available yet.", id="historic-message")

    def on_mount(self) -> None:
        self._update_data()

    def _update_data(self) -> None:
        msg = self.query_one("#historic-message", Label)
        if self.snapshot:
            msg.update("Historic view not yet implemented. Snapshot data is available.")
        else:
            msg.update("No historic data available yet.")
