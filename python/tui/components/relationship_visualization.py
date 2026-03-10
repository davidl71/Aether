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
from ...integration.frontend_views import infer_relationships, relationship_nodes


class RelationshipVisualizationTab(Container):
    """Relationship visualization tab showing instrument relationships"""

    def __init__(
        self,
        snapshot: Optional[SnapshotPayload] = None,
        bank_accounts: Optional[List[Dict]] = None,
        *args,
        **kwargs,
    ):
        super().__init__(*args, **kwargs)
        self.snapshot = snapshot
        self.bank_accounts = bank_accounts or []
        self._precomputed_relationships: Optional[List[Dict]] = None
        self._precomputed_nodes: Optional[List[str]] = None

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
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
        bank_accounts: Optional[List[Dict]] = None,
        relationships: Optional[List[Dict]] = None,
        nodes: Optional[List[str]] = None,
    ) -> None:
        """Update with new snapshot data"""
        self.snapshot = snapshot
        if bank_accounts is not None:
            self.bank_accounts = bank_accounts
        self._precomputed_relationships = relationships
        self._precomputed_nodes = nodes
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
        if not self.snapshot:
            return []
        if self._precomputed_relationships is not None:
            return self._precomputed_relationships
        return infer_relationships(
            [position.to_dict() for position in self.snapshot.positions],
            self.bank_accounts,
        )

    def _get_unique_nodes(self, relationships: List[Dict]) -> List[str]:
        """Get unique node names from relationships"""
        if self._precomputed_nodes is not None:
            return self._precomputed_nodes
        return relationship_nodes(
            relationships,
            [position.to_dict() for position in self.snapshot.positions] if self.snapshot else [],
            self.bank_accounts,
        )
