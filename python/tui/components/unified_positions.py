"""
Unified Positions Tab Component for TUI

Displays all positions across all instrument types (box spreads, bank loans, pension loans, bonds, T-bills, futures)
in a unified, grouped view.
"""

from __future__ import annotations

from typing import Optional, Dict, List
from collections import defaultdict
from textual.widgets import DataTable, Label
from textual.containers import Container, Vertical
from textual.app import ComposeResult

from ..models import SnapshotPayload, PositionSnapshot, Candle

INSTRUMENT_TYPE_LABELS: Dict[str, str] = {
    'box_spread': 'Box Spreads',
    'bank_loan': 'Bank Loans',
    'pension_loan': 'Pension Loans',
    'bond': 'Bonds',
    't_bill': 'T-Bills',
    'futures': 'Futures',
    'cash': 'Cash',
    'other': 'Other',
}

INSTRUMENT_TYPE_ORDER: List[str] = [
    'cash',
    'box_spread',
    'bank_loan',
    'pension_loan',
    'bond',
    't_bill',
    'futures',
    'other',
]


class UnifiedPositionsTab(Container):
    """Unified positions tab showing all positions grouped by instrument type"""

    def __init__(self, snapshot: Optional[SnapshotPayload] = None, bank_accounts: Optional[List[Dict]] = None, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.snapshot = snapshot
        self.bank_accounts = bank_accounts or []
        self._expanded_groups: set = set(INSTRUMENT_TYPE_ORDER)  # All groups expanded by default
        self._expanded_groups: set = set(INSTRUMENT_TYPE_ORDER)  # All groups expanded by default

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Unified Positions", classes="tab-title")
            yield Label(id="summary-label")
            yield Container(id="positions-container")

    def on_mount(self) -> None:
        self._update_data()

    def update_snapshot(self, snapshot: SnapshotPayload, bank_accounts: Optional[List[Dict]] = None) -> None:
        """Update with new snapshot data"""
        self.snapshot = snapshot
        if bank_accounts is not None:
            self.bank_accounts = bank_accounts
        self._update_data()

    def _update_data(self) -> None:
        """Update the display with current data"""
        if not self.snapshot:
            return

        # Convert bank accounts to positions
        bank_account_positions = self._convert_bank_accounts_to_positions()

        # Combine all positions
        all_positions = list(self.snapshot.positions) + bank_account_positions

        # Group positions by instrument type
        grouped = self._group_positions(all_positions)

        # Update summary
        summary_label = self.query_one("#summary-label", Label)
        total_count = sum(len(positions) for positions in grouped.values())
        summary_label.update(f"Total Positions: {total_count}")

        # Update positions container
        container = self.query_one("#positions-container", Container)
        container.remove_children()

        # Create sections for each instrument type
        for inst_type in INSTRUMENT_TYPE_ORDER:
            positions = grouped.get(inst_type, [])
            if not positions:
                continue

            # Create a vertical container for this group
            group_container = Vertical()

            # Add header label
            header_label = Label(f"{INSTRUMENT_TYPE_LABELS[inst_type]} ({len(positions)})", classes="position-group-header")
            group_container.compose_add_child(header_label)

            # Create data table for this group
            table = DataTable()
            self._setup_table_columns(table, inst_type)
            self._populate_table(table, positions, inst_type)
            group_container.compose_add_child(table)

            container.compose_add_child(group_container)

    def _convert_bank_accounts_to_positions(self) -> List[PositionSnapshot]:
        """Convert bank accounts to PositionSnapshot objects"""
        positions = []
        for account in self.bank_accounts:
            rate = account.get('credit_rate') or account.get('debit_rate')
            position = PositionSnapshot(
                name=account.get('account_name', ''),
                quantity=1,
                roi=rate * 100 if rate else 0.0,
                maker_count=0,
                taker_count=0,
                rebate_estimate=0.0,
                vega=0.0,
                theta=0.0,
                fair_diff=0.0,
                candle=self.snapshot.positions[0].candle if self.snapshot and self.snapshot.positions else Candle(),
                instrument_type='bank_loan',
                rate=rate,
                currency=account.get('currency', 'USD'),
                cash_flow=account.get('balance', 0.0)
            )
            positions.append(position)
        return positions

    def _group_positions(self, positions: List[PositionSnapshot]) -> Dict[str, List[PositionSnapshot]]:
        """Group positions by instrument type"""
        grouped: Dict[str, List[PositionSnapshot]] = defaultdict(list)
        for position in positions:
            inst_type = position.instrument_type or 'other'
            if inst_type not in INSTRUMENT_TYPE_LABELS:
                inst_type = 'other'
            grouped[inst_type].append(position)
        return grouped

    def _setup_table_columns(self, table: DataTable, inst_type: str) -> None:
        """Setup table columns based on instrument type"""
        if inst_type == 'box_spread':
            table.add_columns(
                "Name", "Qty", "Rate/ROI", "Maturity", "Cash Flow", "Currency",
                "Vega", "Theta", "Fair Δ"
            )
        else:
            table.add_columns(
                "Name", "Qty", "Rate/ROI", "Maturity", "Cash Flow", "Currency"
            )

    def _populate_table(self, table: DataTable, positions: List[PositionSnapshot], inst_type: str) -> None:
        """Populate table with position data"""
        for position in positions:
            rate_display = f"{position.rate * 100:.2f}%" if position.rate is not None else f"{position.roi:.2f}%"
            maturity_display = self._format_maturity_date(position.maturity_date) if position.maturity_date else "—"
            cash_flow_display = f"${position.cash_flow:,.2f}" if position.cash_flow is not None else "—"
            currency_display = position.currency or "USD"

            if inst_type == 'box_spread':
                table.add_row(
                    position.name,
                    str(position.quantity),
                    rate_display,
                    maturity_display,
                    cash_flow_display,
                    currency_display,
                    f"{position.vega:.3f}",
                    f"{position.theta:.3f}",
                    f"{position.fair_diff:.3f}"
                )
            else:
                table.add_row(
                    position.name,
                    str(position.quantity),
                    rate_display,
                    maturity_display,
                    cash_flow_display,
                    currency_display
                )

    def _format_maturity_date(self, date_str: str) -> str:
        """Format maturity date for display"""
        try:
            from datetime import datetime
            date = datetime.fromisoformat(date_str.replace('Z', '+00:00'))
            return date.strftime('%Y-%m-%d')
        except Exception:
            return date_str
