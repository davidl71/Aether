"""
Opportunity Simulation Tab Component for TUI

Displays what-if scenarios for loan usage and optimization.
"""

from __future__ import annotations

from typing import Optional, Dict, List
from textual.widgets import Container, DataTable, Label, Button
from textual.containers import Vertical, Horizontal
from textual.app import ComposeResult

from ..models import SnapshotPayload, PositionSnapshot


class OpportunitySimulationTab(Container):
    """Opportunity simulation tab showing what-if scenarios"""

    def __init__(
        self,
        snapshot: Optional[SnapshotPayload] = None,
        bank_accounts: Optional[List[Dict]] = None
    ):
        super().__init__()
        self.snapshot = snapshot
        self.bank_accounts = bank_accounts or []
        self.selected_scenario: Optional[str] = None

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Opportunity Simulation", classes="tab-title")
            yield Label(id="scenarios-label")
            yield DataTable(id="scenarios-table")
            yield Label(id="results-label")

    def on_mount(self) -> None:
        table = self.query_one("#scenarios-table", DataTable)
        table.add_columns("Scenario", "Type", "Description", "Net Benefit")
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

        # Find available scenarios
        scenarios = self._find_scenarios()

        # Update scenarios label
        scenarios_label = self.query_one("#scenarios-label", Label)
        scenarios_label.update(f"Available Scenarios: {len(scenarios)}")

        # Update table
        table = self.query_one("#scenarios-table", DataTable)
        table.clear()

        for scenario in scenarios:
            net_benefit = self._calculate_net_benefit(scenario)
            table.add_row(
                scenario['name'],
                scenario['type'].replace('_', ' ').title(),
                scenario['description'],
                f"${net_benefit:,.2f}" if net_benefit else "—"
            )

        # Update results if scenario selected
        if self.selected_scenario:
            scenario = next((s for s in scenarios if s['id'] == self.selected_scenario), None)
            if scenario:
                results = self._calculate_results(scenario)
                results_label = self.query_one("#results-label", Label)
                results_label.update(
                    f"Net Benefit: ${results['net_benefit']:,.2f} | "
                    f"Cash Flow Impact: ${results['cash_flow_impact']:,.2f}/mo | "
                    f"Risk Reduction: {results['risk_reduction']:.1%}"
                )

    def _find_scenarios(self) -> List[Dict]:
        """Find available scenarios based on positions"""
        scenarios = []

        if not self.snapshot:
            return scenarios

        # Find loans
        loans = [
            p for p in self.snapshot.positions
            if p.instrument_type in ('bank_loan', 'pension_loan')
        ]
        bank_loans = [
            a for a in self.bank_accounts
            if a.get('debit_rate') and a.get('debit_rate', 0) > 0
        ]

        # Scenario 1: Loan Consolidation
        if len(loans) > 1 or len(bank_loans) > 0:
            all_loans = [
                {'rate': l.rate or 0, 'balance': l.cash_flow or l.candle.close or 0}
                for l in loans
            ] + [
                {'rate': a.get('debit_rate', 0), 'balance': a.get('balance', 0)}
                for a in bank_loans
            ]
            highest_rate_loan = max(all_loans, key=lambda x: x['rate'], default=None)

            if highest_rate_loan and highest_rate_loan['rate'] > 0.03:
                scenarios.append({
                    'id': 'loan_consolidation',
                    'name': 'Loan Consolidation',
                    'type': 'loan_consolidation',
                    'description': 'Consolidate high-rate loans using lower-rate financing',
                    'parameters': {
                        'loan_amount': highest_rate_loan['balance'],
                        'loan_rate': highest_rate_loan['rate'],
                        'target_rate': 0.04
                    }
                })

        # Scenario 2: Margin for Box Spreads
        box_spreads = [
            p for p in self.snapshot.positions
            if p.instrument_type == 'box_spread'
        ]
        if loans and box_spreads:
            loan = loans[0]
            scenarios.append({
                'id': 'margin_for_box_spread',
                'name': 'Use Loan as Margin for Box Spreads',
                'type': 'margin_for_box_spread',
                'description': 'Use loan proceeds as margin collateral for box spread positions',
                'parameters': {
                    'loan_amount': loan.cash_flow or loan.candle.close or 0,
                    'loan_rate': loan.rate or 0,
                    'box_spread_rate': box_spreads[0].rate or 0.05
                }
            })

        # Scenario 3: Investment Fund Strategy
        if loans:
            loan = loans[0]
            scenarios.append({
                'id': 'investment_fund',
                'name': 'Investment Fund Strategy',
                'type': 'investment_fund',
                'description': 'Use loan to invest in fund, use fund as collateral for cheaper loan',
                'parameters': {
                    'loan_amount': loan.cash_flow or loan.candle.close or 0,
                    'loan_rate': loan.rate or 0,
                    'fund_return': 0.06
                }
            })

        return scenarios

    def _calculate_net_benefit(self, scenario: Dict) -> float:
        """Calculate net benefit for a scenario"""
        params = scenario.get('parameters', {})

        if scenario['type'] == 'loan_consolidation':
            current_cost = params.get('loan_amount', 0) * params.get('loan_rate', 0)
            new_cost = params.get('loan_amount', 0) * params.get('target_rate', 0)
            return current_cost - new_cost

        elif scenario['type'] == 'margin_for_box_spread':
            loan_cost = params.get('loan_amount', 0) * params.get('loan_rate', 0)
            box_spread_return = params.get('loan_amount', 0) * params.get('box_spread_rate', 0)
            return box_spread_return - loan_cost

        elif scenario['type'] == 'investment_fund':
            loan_cost = params.get('loan_amount', 0) * params.get('loan_rate', 0)
            fund_return = params.get('loan_amount', 0) * params.get('fund_return', 0)
            return fund_return - loan_cost

        return 0.0

    def _calculate_results(self, scenario: Dict) -> Dict:
        """Calculate detailed results for a scenario"""
        net_benefit = self._calculate_net_benefit(scenario)
        return {
            'net_benefit': net_benefit,
            'cash_flow_impact': net_benefit / 12,
            'risk_reduction': 0.15 if scenario['type'] == 'loan_consolidation' else 0.05
        }
