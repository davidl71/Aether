"""
Opportunity Simulation Calculator

Shared module for calculating opportunity simulation scenarios.
Used by both TUI and PWA (via REST API).

This module identifies optimization opportunities and calculates their benefits.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import List, Dict, Optional


@dataclass
class SimulationScenario:
    """A simulation scenario"""

    id: str
    name: str
    type: str  # 'loan_consolidation', 'margin_for_box_spread', 'investment_fund'
    description: str
    parameters: Dict[str, float]


@dataclass
class ScenarioResult:
    """Result of scenario calculation"""

    net_benefit: float
    cash_flow_impact: float  # Monthly benefit
    risk_reduction: float  # Percentage
    capital_efficiency: Optional[float] = None


def find_available_scenarios(
    positions: List[Dict], bank_accounts: List[Dict]
) -> List[SimulationScenario]:
    """
    Find available simulation scenarios based on current positions.

    Args:
        positions: List of position dictionaries
        bank_accounts: List of bank account dictionaries

    Returns:
        List of available scenarios
    """
    scenarios: List[SimulationScenario] = []

    # Find loans
    loans = [
        p
        for p in positions
        if p.get("instrument_type") in ("bank_loan", "pension_loan")
    ]
    bank_loans = [
        a for a in bank_accounts if a.get("debit_rate") and a.get("debit_rate", 0) > 0
    ]

    # Scenario 1: Loan Consolidation
    if len(loans) > 1 or len(bank_loans) > 0:
        all_loans = [
            {
                "rate": l.get("rate") or 0,
                "balance": (
                    l.get("cash_flow") or l.get("candle", {}).get("close") or 0
                ),
            }
            for l in loans
        ] + [
            {"rate": a.get("debit_rate", 0), "balance": a.get("balance", 0)}
            for a in bank_loans
        ]
        highest_rate_loan = max(all_loans, key=lambda x: x["rate"], default=None)

        if highest_rate_loan and highest_rate_loan["rate"] > 0.03:
            scenarios.append(
                SimulationScenario(
                    id="loan_consolidation",
                    name="Loan Consolidation",
                    type="loan_consolidation",
                    description="Consolidate high-rate loans using lower-rate financing",
                    parameters={
                        "loan_amount": highest_rate_loan["balance"],
                        "loan_rate": highest_rate_loan["rate"],
                        "target_rate": 0.04,
                    },
                )
            )

    # Scenario 2: Margin for Box Spreads
    box_spreads = [p for p in positions if p.get("instrument_type") == "box_spread"]
    if loans and box_spreads:
        loan = loans[0]
        scenarios.append(
            SimulationScenario(
                id="margin_for_box_spread",
                name="Use Loan as Margin for Box Spreads",
                type="margin_for_box_spread",
                description="Use loan proceeds as margin collateral for box spread positions",
                parameters={
                    "loan_amount": (
                        loan.get("cash_flow")
                        or loan.get("candle", {}).get("close")
                        or 0
                    ),
                    "loan_rate": loan.get("rate") or 0,
                    "box_spread_rate": box_spreads[0].get("rate") or 0.05,
                },
            )
        )

    # Scenario 3: Investment Fund Strategy
    if loans:
        loan = loans[0]
        scenarios.append(
            SimulationScenario(
                id="investment_fund",
                name="Investment Fund Strategy",
                type="investment_fund",
                description="Use loan to invest in fund, use fund as collateral for cheaper loan",
                parameters={
                    "loan_amount": (
                        loan.get("cash_flow")
                        or loan.get("candle", {}).get("close")
                        or 0
                    ),
                    "loan_rate": loan.get("rate") or 0,
                    "fund_return": 0.06,
                },
            )
        )

    return scenarios


def calculate_net_benefit(scenario: SimulationScenario) -> float:
    """
    Calculate net benefit for a scenario.

    Args:
        scenario: Simulation scenario

    Returns:
        Net benefit (annual)
    """
    params = scenario.parameters

    if scenario.type == "loan_consolidation":
        current_cost = params.get("loan_amount", 0) * params.get("loan_rate", 0)
        new_cost = params.get("loan_amount", 0) * params.get("target_rate", 0)
        return current_cost - new_cost

    elif scenario.type == "margin_for_box_spread":
        loan_cost = params.get("loan_amount", 0) * params.get("loan_rate", 0)
        box_spread_return = params.get("loan_amount", 0) * params.get(
            "box_spread_rate", 0
        )
        return box_spread_return - loan_cost

    elif scenario.type == "investment_fund":
        loan_cost = params.get("loan_amount", 0) * params.get("loan_rate", 0)
        fund_return = params.get("loan_amount", 0) * params.get("fund_return", 0)
        return fund_return - loan_cost

    return 0.0


def calculate_scenario_results(scenario: SimulationScenario) -> ScenarioResult:
    """
    Calculate detailed results for a scenario.

    Args:
        scenario: Simulation scenario

    Returns:
        ScenarioResult with net benefit, cash flow impact, and risk metrics
    """
    net_benefit = calculate_net_benefit(scenario)

    # Risk reduction varies by scenario type
    risk_reduction = 0.15 if scenario.type == "loan_consolidation" else 0.05

    # Capital efficiency varies by scenario type
    capital_efficiency = None
    if scenario.type == "margin_for_box_spread":
        capital_efficiency = 1.2
    elif scenario.type == "investment_fund":
        capital_efficiency = 1.5
    elif scenario.type == "loan_consolidation":
        capital_efficiency = 1.0

    return ScenarioResult(
        net_benefit=net_benefit,
        cash_flow_impact=net_benefit / 12,  # Monthly benefit
        risk_reduction=risk_reduction,
        capital_efficiency=capital_efficiency,
    )
