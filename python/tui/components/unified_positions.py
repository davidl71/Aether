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
from .snapshot_display import ENVIRONMENT_LABELS, ENVIRONMENT_MARKUP
from ...integration.frontend_views import normalize_bank_accounts_to_positions

try:
    from ...integration.dte_utils import days_to_maturity_from_date
except ImportError:
    days_to_maturity_from_date = None  # type: ignore[misc, assignment]

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
        self._precomputed_positions: Optional[List[Dict]] = None
        self._expanded_groups: set = set(INSTRUMENT_TYPE_ORDER)
        self._environment = "mock"
        self._provider_label = ""

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Unified Positions", classes="tab-title")
            yield Label(id="data-source-label", classes="unified-data-source")
            yield Label(id="summary-label")
            yield Container(id="positions-container")

    def on_mount(self) -> None:
        self._update_data()

    def update_snapshot(
        self,
        snapshot: SnapshotPayload,
        bank_accounts: Optional[List[Dict]] = None,
        environment: Optional[str] = None,
        provider_label: Optional[str] = None,
        precomputed_positions: Optional[List[Dict]] = None,
    ) -> None:
        """Update with new snapshot data. environment: 'mock'|'paper'|'live'; provider_label for display."""
        self.snapshot = snapshot
        if bank_accounts is not None:
            self.bank_accounts = bank_accounts
        self._precomputed_positions = precomputed_positions
        self._environment = environment or "mock"
        self._provider_label = provider_label or ""
        self._update_data()

    def _update_data(self) -> None:
        """Update the display with current data"""
        if not self.snapshot:
            return

        if self._precomputed_positions is not None:
            all_positions = [
                PositionSnapshot.from_dict(position)
                for position in self._precomputed_positions
            ]
        else:
            bank_account_positions = self._convert_bank_accounts_to_positions()
            all_positions = list(self.snapshot.positions) + bank_account_positions

        # Group positions by instrument type
        grouped = self._group_positions(all_positions)

        # Data source: real / mock / paper
        env = self._environment or "mock"
        env_label = ENVIRONMENT_LABELS.get(env, env.upper())
        env_markup = ENVIRONMENT_MARKUP.get(env, f"[bold]{env_label}[/]")
        source_parts = [f"Data: {env_markup}"]
        if self._provider_label:
            source_parts.append(f"  ·  {self._provider_label}")
        try:
            source_label = self.query_one("#data-source-label", Label)
            source_label.update("  ".join(source_parts))
        except Exception:
            pass

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
        reference_candle = (
            self.snapshot.positions[0].candle.to_dict()
            if self.snapshot and self.snapshot.positions
            else Candle().to_dict()
        )
        return [
            PositionSnapshot.from_dict(position)
            for position in normalize_bank_accounts_to_positions(
                self.bank_accounts, reference_candle=reference_candle
            )
        ]

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
                "Name", "Qty", "Conid", "DTE", "Rate/ROI", "Maturity", "Cash Flow", "Currency",
                "Vega", "Theta", "Fair Δ"
            )
        else:
            table.add_columns(
                "Name", "Qty", "Conid", "DTE", "Rate/ROI", "Maturity", "Cash Flow", "Currency"
            )

    def _populate_table(self, table: DataTable, positions: List[PositionSnapshot], inst_type: str) -> None:
        """Populate table with position data"""
        for position in positions:
            rate_display = f"{position.rate * 100:.2f}%" if position.rate is not None else f"{position.roi:.2f}%"
            maturity_display = self._format_maturity_date(position.maturity_date) if position.maturity_date else "—"
            dte_val = days_to_maturity_from_date(position.maturity_date) if (days_to_maturity_from_date and position.maturity_date) else None
            dte_display = str(dte_val) if dte_val is not None else "—"
            conid_display = str(position.conid) if position.conid is not None else "—"
            cash_flow_display = "—"
            v = position.cash_flow if position.cash_flow is not None else position.expected_cash_at_expiry
            if v is not None:
                cash_flow_display = f"-${-v:,.2f}" if v < 0 else f"${v:,.2f}"
            currency_display = position.currency or "USD"

            if inst_type == 'box_spread':
                table.add_row(
                    position.name,
                    str(position.quantity),
                    conid_display,
                    dte_display,
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
                    conid_display,
                    dte_display,
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
