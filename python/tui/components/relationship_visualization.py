"""
Relationship Visualization Tab Component for TUI

Displays relationships between instruments (loan → margin → box spread → fund → cheaper loan).
"""

from __future__ import annotations

from typing import Optional, Dict, List
from textual.widgets import DataTable, Label
from textual.containers import Container, Vertical
from textual.app import ComposeResult

from ..models import SnapshotPayload


class RelationshipVisualizationTab(Container):
    """Relationship visualization tab showing instrument relationships"""

    def __init__(
        self,
        snapshot: Optional[SnapshotPayload] = None,
        bank_accounts: Optional[List[Dict]] = None
    ):
        super().__init__()
        self.snapshot = snapshot
        self.bank_accounts = bank_accounts or []

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Multi-Instrument Relationships", classes="tab-title")
            yield Label(id="summary-label")
            yield DataTable(id="relationships-table")

    def on_mount(self) -> None:
        table = self.query_one("#relationships-table", DataTable)
        table.add_columns("From", "Type", "To", "Description", "Value")
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

    def _update_data(self) -> None:
        """Update the display with current data"""
        if not self.snapshot:
            return

        # Find relationships
        relationships = self._find_relationships()

        # Update summary
        summary_label = self.query_one("#summary-label", Label)
        summary_label.update(
            f"Total Relationships: {len(relationships)} | "
            f"Instruments: {len(self._get_unique_nodes(relationships))}"
        )

        # Update table
        table = self.query_one("#relationships-table", DataTable)
        table.clear()

        for rel in relationships:
            table.add_row(
                rel['from'],
                rel['type'].replace('_', ' ').title(),
                rel['to'],
                rel['description'],
                f"${rel['value']:,.2f}"
            )

    def _find_relationships(self) -> List[Dict]:
        """Find relationships between instruments"""
        relationships = []

        if not self.snapshot:
            return relationships

        # Find loans
        loans = [
            p for p in self.snapshot.positions
            if p.instrument_type in ('bank_loan', 'pension_loan')
        ]

        # Find box spreads
        box_spreads = [
            p for p in self.snapshot.positions
            if p.instrument_type == 'box_spread'
        ]

        # Find bonds/funds
        bonds = [
            p for p in self.snapshot.positions
            if p.instrument_type in ('bond', 't_bill')
        ]

        # Relationship 1: Loan → Box Spread (margin)
        for loan in loans:
            for box_spread in box_spreads:
                relationships.append({
                    'from': loan.name,
                    'to': box_spread.name,
                    'type': 'margin',
                    'description': 'Loan used as margin for box spread',
                    'value': loan.cash_flow or loan.candle.close or 0.0
                })

        # Relationship 2: Loan → Investment (fund/bond)
        for loan in loans:
            for bond in bonds:
                relationships.append({
                    'from': loan.name,
                    'to': bond.name,
                    'type': 'investment',
                    'description': 'Loan proceeds invested in bond',
                    'value': loan.cash_flow or loan.candle.close or 0.0
                })

        # Relationship 3: Investment → Collateral (for cheaper loan)
        for bond in bonds:
            for loan in loans:
                bond_rate = bond.rate or 0.0
                loan_rate = loan.rate or 0.0
                if bond_rate > loan_rate:
                    relationships.append({
                        'from': bond.name,
                        'to': loan.name,
                        'type': 'collateral',
                        'description': 'Bond used as collateral for loan',
                        'value': bond.collateral_value or bond.cash_flow or bond.candle.close or 0.0
                    })

        # Relationship 4: Box Spread → Financing (synthetic financing)
        for box_spread in box_spreads:
            relationships.append({
                'from': box_spread.name,
                'to': 'Synthetic Financing',
                'type': 'financing',
                'description': 'Box spread provides synthetic financing',
                'value': box_spread.cash_flow or box_spread.candle.close or 0.0
            })

        return relationships

    def _get_unique_nodes(self, relationships: List[Dict]) -> List[str]:
        """Get unique node names from relationships"""
        nodes = set()
        for rel in relationships:
            nodes.add(rel['from'])
            nodes.add(rel['to'])
        for pos in self.snapshot.positions:
            nodes.add(pos.name)
        for acc in self.bank_accounts:
            nodes.add(acc.get('account_name', ''))
        return list(nodes)
