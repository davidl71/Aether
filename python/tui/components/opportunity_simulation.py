"""
Opportunity Simulation Tab Component for TUI

Displays what-if scenarios for loan usage and optimization.
"""

from __future__ import annotations

from typing import Optional, Dict, List
from textual.widgets import DataTable, Label
from textual.containers import Container, Vertical
from textual.app import ComposeResult

from ..models import SnapshotPayload
from ...integration.opportunity_simulation_calculator import (
    find_available_scenarios,
    calculate_net_benefit,
    calculate_scenario_results,
)


class OpportunitySimulationTab(Container):
    """Opportunity simulation tab showing what-if scenarios"""

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
        self.selected_scenario: Optional[str] = None

    def compose(self) -> ComposeResult:
        with Vertical(classes="fill"):
            yield Label("Opportunity Simulation", classes="tab-title")
            yield Label(id="scenarios-label")
            yield DataTable(id="opportunity-scenarios-table")
            yield Label(id="results-label")

    def on_mount(self) -> None:
        table = self.query_one("#opportunity-scenarios-table", DataTable)
        table.add_columns("Scenario", "Type", "Description", "Net Benefit")
        self._update_data()

    def update_snapshot(
        self, snapshot: SnapshotPayload, bank_accounts: Optional[List[Dict]] = None
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

        # Convert positions to dict format for shared calculator
        positions_dict = [
            {
                "name": p.name,
                "instrument_type": p.instrument_type,
                "cash_flow": p.cash_flow,
                "candle": {"close": p.candle.close if p.candle else None},
                "rate": p.rate,
            }
            for p in self.snapshot.positions
        ]

        # Find available scenarios using shared module
        scenarios = find_available_scenarios(
            positions=positions_dict, bank_accounts=self.bank_accounts
        )

        # Update scenarios label
        scenarios_label = self.query_one("#scenarios-label", Label)
        scenarios_label.update(f"Available Scenarios: {len(scenarios)}")

        # Update table
        table = self.query_one("#opportunity-scenarios-table", DataTable)
        table.clear()

        for scenario in scenarios:
            net_benefit = calculate_net_benefit(scenario)
            table.add_row(
                scenario.name,
                scenario.type.replace("_", " ").title(),
                scenario.description,
                f"${net_benefit:,.2f}" if net_benefit else "—",
            )

        # Update results if scenario selected
        if self.selected_scenario:
            scenario = next(
                (s for s in scenarios if s.id == self.selected_scenario), None
            )
            if scenario:
                results = calculate_scenario_results(scenario)
                results_label = self.query_one("#results-label", Label)
                results_label.update(
                    f"Net Benefit: ${results.net_benefit:,.2f} | "
                    f"Cash Flow Impact: ${results.cash_flow_impact:,.2f}/mo | "
                    f"Risk Reduction: {results.risk_reduction:.1%}"
                )
