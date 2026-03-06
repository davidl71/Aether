"""Orders tab: recent orders log."""

from __future__ import annotations

from textual.containers import Vertical
from textual.widgets import Label, Log
from textual.app import ComposeResult

from .base import SnapshotTabBase


class OrdersTab(SnapshotTabBase):
    """Orders tab showing recent orders."""

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Recent Orders", classes="tab-title")
            yield Log(id="orders-log")

    def on_mount(self) -> None:
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
            pass
