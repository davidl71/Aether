"""
Cash Flow Tab Component for TUI

Displays cash flow projections from all positions (box spreads, loans, bonds, etc.)
with monthly breakdown and totals.
"""

from __future__ import annotations

from typing import Optional, Dict, List
from collections import defaultdict
from datetime import datetime, timedelta
from textual.widgets import Container, DataTable, Label, Select
from textual.containers import Vertical, Horizontal
from textual.app import ComposeResult

from ..models import SnapshotPayload, PositionSnapshot


class CashFlowTab(Container):
    """Cash flow tab showing projected cash flows from all positions"""

    def __init__(
        self,
        snapshot: Optional[SnapshotPayload] = None,
        bank_accounts: Optional[List[Dict]] = None,
        projection_months: int = 12
    ):
        super().__init__()
        self.snapshot = snapshot
        self.bank_accounts = bank_accounts or []
        self.projection_months = projection_months

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Cash Flow Projection", classes="tab-title")

            with Horizontal():
                yield Label("Projection Period:")
                yield Select(
                    [(str(i), str(i)) for i in [6, 12, 24, 36]],
                    value=str(self.projection_months),
                    id="projection-select"
                )

            yield Label(id="summary-label")
            yield DataTable(id="cash-flow-table")

    def on_mount(self) -> None:
        table = self.query_one("#cash-flow-table", DataTable)
        table.add_columns(
            "Month",
            "Inflows",
            "Outflows",
            "Net",
            "Events"
        )
        self._update_data()

    def update_snapshot(
        self,
        snapshot: SnapshotPayload,
        bank_accounts: Optional[List[Dict]] = None
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

        # Calculate cash flows
        cash_flows = self._calculate_cash_flows()

        # Group by month
        monthly_flows = self._group_by_month(cash_flows)

        # Calculate totals
        total_inflows = sum(m['inflows'] for m in monthly_flows.values())
        total_outflows = sum(m['outflows'] for m in monthly_flows.values())
        net_cash_flow = total_inflows - total_outflows

        # Update summary
        summary_label = self.query_one("#summary-label", Label)
        summary_label.update(
            f"Total Inflows: ${total_inflows:,.2f} | "
            f"Total Outflows: ${total_outflows:,.2f} | "
            f"Net Cash Flow: ${net_cash_flow:,.2f}"
        )

        # Update table
        table = self.query_one("#cash-flow-table", DataTable)
        table.clear()

        # Sort months
        sorted_months = sorted(monthly_flows.keys())

        for month in sorted_months:
            monthly = monthly_flows[month]
            month_display = datetime.strptime(month, "%Y-%m").strftime("%b %Y")
            table.add_row(
                month_display,
                f"${monthly['inflows']:,.2f}" if monthly['inflows'] > 0 else "—",
                f"${monthly['outflows']:,.2f}" if monthly['outflows'] > 0 else "—",
                f"${monthly['net']:,.2f}",
                str(len(monthly['events']))
            )

    def _calculate_cash_flows(self) -> List[Dict]:
        """Calculate cash flow events from positions"""
        events = []
        now = datetime.now()

        # Process positions
        for position in self.snapshot.positions:
            if position.maturity_date:
                try:
                    maturity_date = datetime.fromisoformat(
                        position.maturity_date.replace('Z', '+00:00')
                    )
                    months_ahead = (maturity_date.year - now.year) * 12 + (
                        maturity_date.month - now.month
                    )

                    if 0 <= months_ahead <= self.projection_months:
                        # Maturity cash flow
                        cash_flow_amount = position.cash_flow or position.candle.close or 0.0
                        events.append({
                            'date': position.maturity_date,
                            'amount': cash_flow_amount,
                            'description': f"{position.instrument_type or 'Position'} maturity",
                            'position_name': position.name,
                            'type': 'maturity'
                        })

                        # Monthly interest payments for loans
                        if position.instrument_type in ('bank_loan', 'pension_loan'):
                            rate = position.rate or 0.0
                            principal = position.cash_flow or position.candle.close or 0.0
                            monthly_payment = (principal * rate) / 12

                            for month in range(1, min(months_ahead, self.projection_months) + 1):
                                payment_date = now + timedelta(days=30 * month)
                                events.append({
                                    'date': payment_date.isoformat().split('T')[0],
                                    'amount': -monthly_payment,  # Outflow
                                    'description': 'Monthly interest payment',
                                    'position_name': position.name,
                                    'type': 'loan_payment'
                                })
                except (ValueError, AttributeError):
                    pass

            # Current cash flow
            if position.cash_flow is not None and position.cash_flow != 0:
                events.append({
                    'date': now.isoformat().split('T')[0],
                    'amount': position.cash_flow,
                    'description': f"Current {position.instrument_type or 'position'} cash flow",
                    'position_name': position.name,
                    'type': 'other'
                })

        # Process bank accounts (as loans if debit_rate exists)
        for account in self.bank_accounts:
            debit_rate = account.get('debit_rate')
            if debit_rate and debit_rate > 0:
                principal = account.get('balance', 0.0)
                monthly_payment = (principal * debit_rate) / 12

                for month in range(1, self.projection_months + 1):
                    payment_date = now + timedelta(days=30 * month)
                    events.append({
                        'date': payment_date.isoformat().split('T')[0],
                        'amount': -monthly_payment,  # Outflow
                        'description': 'Monthly interest payment',
                        'position_name': account.get('account_name', 'Bank Account'),
                        'type': 'loan_payment'
                    })

        return events

    def _group_by_month(self, events: List[Dict]) -> Dict[str, Dict]:
        """Group cash flow events by month"""
        monthly: Dict[str, Dict] = defaultdict(lambda: {
            'inflows': 0.0,
            'outflows': 0.0,
            'net': 0.0,
            'events': []
        })

        for event in events:
            month = event['date'][:7]  # YYYY-MM
            monthly[month]['events'].append(event)

            if event['amount'] > 0:
                monthly[month]['inflows'] += event['amount']
            else:
                monthly[month]['outflows'] += abs(event['amount'])

            monthly[month]['net'] = (
                monthly[month]['inflows'] - monthly[month]['outflows']
            )

        return dict(monthly)
