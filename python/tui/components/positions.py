"""Positions tab: current positions table."""

from __future__ import annotations

from typing import Optional

from textual.containers import Vertical
from textual.widgets import Label, DataTable
from textual.app import ComposeResult

from .base import SnapshotTabBase


def _fmt(v: Optional[float], decimals: int = 2, prefix: str = "") -> str:
    """Format number or return '—' if None."""
    if v is None:
        return "—"
    if decimals == 0:
        return f"{prefix}{int(v):,}"
    return f"{prefix}{v:,.{decimals}f}"


class PositionsTab(SnapshotTabBase):
    """Positions tab showing current positions with price metrics, side, and expiry cash."""

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Current Positions", classes="tab-title")
            yield DataTable(id="positions-table")

    def on_mount(self) -> None:
        table = self.query_one("#positions-table", DataTable)
        table.add_columns(
            "Side",
            "Name",
            "Qty",
            "Bid",
            "Ask",
            "Last",
            "Spread",
            "Price",
            "Value",
            "Currency",
            "Dividend",
            "Expiry cash",
        )
        self._update_data()

    def _update_data(self) -> None:
        if not self.snapshot:
            return

        table = self.query_one("#positions-table", DataTable)
        table.clear()

        for pos in self.snapshot.positions:
            side_str = (pos.side or "—").capitalize()
            value_str = _fmt(pos.market_value, 2, "$") if pos.market_value is not None else "—"
            curr_str = pos.currency or "USD"
            div_str = _fmt(pos.dividend, 2, "$") if pos.dividend is not None else "—"
            exp_cash_str = _fmt(pos.expected_cash_at_expiry, 2, "$") if pos.expected_cash_at_expiry is not None else "—"
            price_str = _fmt(pos.price or pos.last, 2) if (pos.price is not None or pos.last is not None) else "—"
            table.add_row(
                side_str,
                pos.name,
                str(pos.quantity),
                _fmt(pos.bid),
                _fmt(pos.ask),
                _fmt(pos.last),
                _fmt(pos.spread),
                price_str,
                value_str,
                curr_str,
                div_str,
                exp_cash_str,
            )
