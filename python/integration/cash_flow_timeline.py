"""
Cash Flow Timeline Calculator

Shared module for calculating cash flow timelines from positions and bank accounts.
Used by both TUI and PWA (via REST API).

This module provides a simplified interface compared to cash_flow_calculator.py,
focusing on the specific needs of the frontend components.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import List, Dict
from collections import defaultdict


@dataclass
class CashFlowEvent:
    """Single cash flow event for timeline display"""

    date: str  # ISO date string (YYYY-MM-DD)
    amount: float  # Positive for inflows, negative for outflows
    description: str
    position_name: str
    type: str  # 'maturity', 'loan_payment', 'other'


@dataclass
class MonthlyCashFlow:
    """Monthly aggregated cash flow"""

    month: str  # YYYY-MM
    inflows: float
    outflows: float
    net: float
    events: List[CashFlowEvent]


@dataclass
class CashFlowTimelineResult:
    """Result of cash flow timeline calculation"""

    events: List[CashFlowEvent]
    monthly_flows: Dict[str, MonthlyCashFlow]  # Key: YYYY-MM
    total_inflows: float
    total_outflows: float
    net_cash_flow: float


def calculate_cash_flow_timeline(
    positions: List[Dict], bank_accounts: List[Dict], projection_months: int = 12
) -> CashFlowTimelineResult:
    """
    Calculate cash flow timeline from positions and bank accounts.

    Args:
        positions: List of position dictionaries with fields:
            - maturity_date (optional): ISO date string
            - cash_flow (optional): float
            - candle.close (optional): float
            - instrument_type (optional): str
            - rate (optional): float (for loans)
            - name: str
        bank_accounts: List of bank account dictionaries with fields:
            - debit_rate (optional): float
            - balance: float
            - account_name: str
        projection_months: Number of months to project (default: 12)

    Returns:
        CashFlowTimelineResult with events, monthly flows, and totals
    """
    events: List[CashFlowEvent] = []
    now = datetime.now()

    # Process positions
    for position in positions:
        maturity_date_str = position.get("maturity_date")
        if maturity_date_str:
            try:
                maturity_date = datetime.fromisoformat(
                    maturity_date_str.replace("Z", "+00:00")
                )
                months_ahead = (maturity_date.year - now.year) * 12 + (
                    maturity_date.month - now.month
                )

                if 0 <= months_ahead <= projection_months:
                    # Maturity cash flow
                    cash_flow_amount = (
                        position.get("cash_flow")
                        or position.get("candle", {}).get("close")
                        or 0.0
                    )
                    events.append(
                        CashFlowEvent(
                            date=maturity_date_str.split("T")[0],
                            amount=cash_flow_amount,
                            description=f"{position.get('instrument_type', 'Position')} maturity",
                            position_name=position.get("name", "Unknown"),
                            type="maturity",
                        )
                    )

                    # Monthly interest payments for loans
                    instrument_type = position.get("instrument_type")
                    if instrument_type in ("bank_loan", "pension_loan"):
                        rate = position.get("rate") or 0.0
                        principal = (
                            position.get("cash_flow")
                            or position.get("candle", {}).get("close")
                            or 0.0
                        )
                        monthly_payment = (principal * rate) / 12

                        for month in range(1, min(months_ahead, projection_months) + 1):
                            payment_date = now + timedelta(days=30 * month)
                            events.append(
                                CashFlowEvent(
                                    date=payment_date.isoformat().split("T")[0],
                                    amount=-monthly_payment,  # Outflow
                                    description="Monthly interest payment",
                                    position_name=position.get("name", "Unknown"),
                                    type="loan_payment",
                                )
                            )
            except (ValueError, AttributeError, KeyError):
                pass

        # Current cash flow
        cash_flow = position.get("cash_flow")
        if cash_flow is not None and cash_flow != 0:
            events.append(
                CashFlowEvent(
                    date=now.isoformat().split("T")[0],
                    amount=cash_flow,
                    description=f"Current {position.get('instrument_type', 'position')} cash flow",
                    position_name=position.get("name", "Unknown"),
                    type="other",
                )
            )

    # Process bank accounts (as loans if debit_rate exists)
    for account in bank_accounts:
        debit_rate = account.get("debit_rate")
        if debit_rate and debit_rate > 0:
            principal = account.get("balance", 0.0)
            monthly_payment = (principal * debit_rate) / 12

            for month in range(1, projection_months + 1):
                payment_date = now + timedelta(days=30 * month)
                events.append(
                    CashFlowEvent(
                        date=payment_date.isoformat().split("T")[0],
                        amount=-monthly_payment,  # Outflow
                        description="Monthly interest payment",
                        position_name=account.get("account_name", "Bank Account"),
                        type="loan_payment",
                    )
                )

    # Group by month
    monthly_flows: Dict[str, MonthlyCashFlow] = defaultdict(
        lambda: MonthlyCashFlow(month="", inflows=0.0, outflows=0.0, net=0.0, events=[])
    )

    for event in events:
        month = event.date[:7]  # YYYY-MM
        if monthly_flows[month].month == "":
            monthly_flows[month].month = month

        monthly_flows[month].events.append(event)

        if event.amount > 0:
            monthly_flows[month].inflows += event.amount
        else:
            monthly_flows[month].outflows += abs(event.amount)

        monthly_flows[month].net = (
            monthly_flows[month].inflows - monthly_flows[month].outflows
        )

    # Calculate totals
    total_inflows = sum(m.inflows for m in monthly_flows.values())
    total_outflows = sum(m.outflows for m in monthly_flows.values())
    net_cash_flow = total_inflows - total_outflows

    return CashFlowTimelineResult(
        events=events,
        monthly_flows=dict(monthly_flows),
        total_inflows=total_inflows,
        total_outflows=total_outflows,
        net_cash_flow=net_cash_flow,
    )
