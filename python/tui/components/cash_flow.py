"""
Cash Flow Tab Component for TUI

Displays cash flow projections from all positions (box spreads, loans, bonds, etc.)
with monthly breakdown and totals.
"""

from __future__ import annotations

from typing import Optional, Dict, List
from datetime import datetime
from textual.widgets import DataTable, Label, Select
from textual.containers import Container, Vertical, Horizontal
from textual.app import ComposeResult

from ..models import SnapshotPayload
from ...integration.cash_flow_timeline import calculate_cash_flow_timeline


class CashFlowTab(Container):
    """Cash flow tab showing projected cash flows from all positions"""

    def __init__(
        self,
        snapshot: Optional[SnapshotPayload] = None,
        bank_accounts: Optional[List[Dict]] = None,
        projection_months: int = 12,
        *args,
        **kwargs,
    ):
        super().__init__(*args, **kwargs)
        self.snapshot = snapshot
        self.bank_accounts = bank_accounts or []
        self.projection_months = projection_months

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Cash Flow Projection", classes="tab-title")

            with Horizontal():
                yield Label("Projection Period:")
                yield Select(
                    [(str(i), str(i)) for i in [6, 12, 24, 36]],
                    value=str(self.projection_months),
                    id="projection-select",
                )

            yield Label(id="summary-label")
            yield DataTable(id="cash-flow-table")

    def on_mount(self) -> None:
        table = self.query_one("#cash-flow-table", DataTable)
        table.add_columns("Month", "Inflows", "Outflows", "Net", "Events")
        self._update_data()

    def update_snapshot(
        self, snapshot: SnapshotPayload, bank_accounts: Optional[List[Dict]] = None
    ) -> None:
        """Update with new snapshot data"""
        self.snapshot = snapshot
        if bank_accounts is not None:
            self.bank_accounts = bank_accounts
        self._update_data()

    def on_select_changed(self, event: Select.Changed) -> None:
        """Handle projection period change"""
        if event.control.id == "projection-select":
            self.projection_months = int(event.value)
            self._update_data()

    def _update_data(self) -> None:
        """Update the display with current data"""
        if not self.snapshot:
            return

        # Convert positions to dict format for shared calculator
        positions_dict = [
            {
                "name": p.name,
                "maturity_date": p.maturity_date,
                "cash_flow": p.cash_flow,
                "candle": {"close": p.candle.close if p.candle else None},
                "instrument_type": p.instrument_type,
                "rate": p.rate,
            }
            for p in self.snapshot.positions
        ]

        # Calculate cash flows using shared module (includes reported future_events from snapshot)
        reported = [e.to_dict() for e in self.snapshot.future_events] if self.snapshot.future_events else []
        result = calculate_cash_flow_timeline(
            positions=positions_dict,
            bank_accounts=self.bank_accounts,
            projection_months=self.projection_months,
            reported_future_events=reported,
        )

        # Update summary
        summary_label = self.query_one("#summary-label", Label)
        summary_label.update(
            f"Total Inflows: ${result.total_inflows:,.2f} | "
            f"Total Outflows: ${result.total_outflows:,.2f} | "
            f"Net Cash Flow: ${result.net_cash_flow:,.2f}"
        )

        # Update table
        table = self.query_one("#cash-flow-table", DataTable)
        table.clear()

        # Sort months
        sorted_months = sorted(result.monthly_flows.keys())

        for month in sorted_months:
            monthly = result.monthly_flows[month]
            month_display = datetime.strptime(month, "%Y-%m").strftime("%b %Y")
            table.add_row(
                month_display,
                f"${monthly.inflows:,.2f}" if monthly.inflows > 0 else "—",
                f"${monthly.outflows:,.2f}" if monthly.outflows > 0 else "—",
                f"${monthly.net:,.2f}",
                str(len(monthly.events)),
            )
