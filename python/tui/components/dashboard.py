"""Dashboard tab: symbols table and metrics."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from textual.containers import Vertical
from textual.widgets import Label, DataTable
from textual.app import ComposeResult

from .base import SnapshotTabBase
from .snapshot_display import format_updated_display
from ..models import SnapshotPayload

# Default watchlist when none provided (must match config default for mock sync)
DEFAULT_WATCHLIST: List[str] = ["SPX", "XSP", "NANOS", "TLT", "DSP"]

# Display names for backends in action items (subset of snapshot_display.BACKEND_DISPLAY_NAMES)
BACKEND_DISPLAY_NAMES: Dict[str, str] = {
    "ib": "TWS/IBKR",
    "tws": "TWS",
    "discount_bank": "Discount Bank",
    "rust": "Rust",
}


class DashboardTab(SnapshotTabBase):
    """Dashboard tab showing symbols and metrics."""

    def __init__(
        self,
        snapshot: Optional[SnapshotPayload] = None,
        watchlist: Optional[List[str]] = None,
        *,
        name: Optional[str] = None,
        id: Optional[str] = None,
        classes: Optional[str] = None,
        disabled: bool = False,
    ) -> None:
        self.watchlist: List[str] = list(watchlist or DEFAULT_WATCHLIST)
        self._backend_health: Optional[Dict[str, Any]] = None
        super().__init__(
            snapshot=snapshot,
            name=name,
            id=id,
            classes=classes,
            disabled=disabled,
        )

    def update_snapshot(self, snapshot: SnapshotPayload, **kwargs: object) -> None:
        """Accept backend_health for action items (e.g. services awaiting authentication)."""
        backend_health = kwargs.pop("backend_health", None)
        self._backend_health = backend_health if isinstance(backend_health, dict) else None
        super().update_snapshot(snapshot, **kwargs)

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Dashboard", classes="tab-title")
            yield DataTable(id="symbols-table")
            yield Label(id="action-items-label")
            yield Label(id="metrics-label")

    def on_mount(self) -> None:
        table = self.query_one("#symbols-table", DataTable)
        table.add_columns("Symbol", "Last", "Bid", "Ask", "Spread", "ROI%", "Updated")
        self._update_data()

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        available_symbols = {s.symbol.upper() for s in self.snapshot.symbols}
        missing_symbols = [
            s for s in self.watchlist
            if s.upper() not in available_symbols
        ]

        # Build action items (notes / things to act on)
        action_items: List[str] = []

        if missing_symbols:
            action_items.append(
                f"The following symbols are in your watchlist but not available in the current snapshot: {', '.join(missing_symbols)}"
            )

        # Services disabled or awaiting authentication (from backend health)
        if self._backend_health:
            awaiting: List[str] = []
            checking: List[str] = []
            unreachable: List[str] = []
            for name, payload in sorted(self._backend_health.items()):
                if not isinstance(payload, dict):
                    continue
                status = payload.get("status", "")
                label = BACKEND_DISPLAY_NAMES.get(name.lower(), name)
                if status == "disabled":
                    awaiting.append(label)
                elif status == "checking":
                    checking.append(label)
                elif status == "error":
                    unreachable.append(label)
            if awaiting:
                action_items.append(
                    "Services awaiting authentication or configuration: " + ", ".join(awaiting)
                )
            if checking:
                action_items.append(
                    "Services still connecting: " + ", ".join(checking)
                )
            if unreachable:
                action_items.append(
                    "Backends unreachable (retrying connection): " + ", ".join(unreachable)
                )

        action_label = self.query_one("#action-items-label", Label)
        if action_items:
            lines = "  • " + "\n  • ".join(action_items)
            action_label.update(f"Note:\n{lines}")
            action_label.add_class("warning")
        else:
            action_label.update("")
            action_label.remove_class("warning")

        table = self.query_one("#symbols-table", DataTable)
        table.clear()

        for symbol in self.snapshot.symbols:
            updated_str = format_updated_display(
                getattr(getattr(symbol, "candle", None), "updated", "") or ""
            )
            table.add_row(
                symbol.symbol,
                f"{symbol.last:.2f}" if symbol.last > 0 else "--",
                f"{symbol.bid:.2f}" if symbol.bid > 0 else "--",
                f"{symbol.ask:.2f}" if symbol.ask > 0 else "--",
                f"{symbol.spread:.2f}" if symbol.spread > 0 else "--",
                f"{symbol.roi:.2f}" if symbol.roi > 0 else "--",
                updated_str,
            )

        metrics = self.snapshot.metrics
        metrics_label = self.query_one("#metrics-label", Label)
        snapshot_updated = format_updated_display(self.snapshot.generated_at)
        metrics_label.update(
            f"Positions: {len(self.snapshot.positions)} | "
            f"Orders: {len(self.snapshot.orders)} | "
            f"Alerts: {len(self.snapshot.alerts)} | "
            f"Net Liq: ${metrics.net_liq:,.0f} | "
            f"Data updated: {snapshot_updated}"
        )
