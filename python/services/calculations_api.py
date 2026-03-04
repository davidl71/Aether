"""
Calculations API Service

FastAPI service exposing shared calculation endpoints for cash flow and opportunity simulation.
Used by both PWA and TUI (via REST API).

Endpoints:
- POST /api/v1/cash-flow/timeline - Calculate cash flow timeline
- POST /api/v1/opportunity-simulation/scenarios - Find available scenarios
- POST /api/v1/opportunity-simulation/calculate - Calculate scenario results
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import List, Dict, Optional, Any

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.integration.cash_flow_timeline import (
    calculate_cash_flow_timeline
)
from python.integration.opportunity_simulation_calculator import (
    find_available_scenarios,
    calculate_net_benefit,
    calculate_scenario_results,
    SimulationScenario
)

# Initialize FastAPI app
app = FastAPI(
    title="Calculations API",
    description="REST API for shared calculation endpoints (cash flow, opportunity simulation)",
    version="1.0.0"
)

# CORS middleware for web frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, restrict to specific origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Request/Response Models
class PositionInput(BaseModel):
    """Position input for calculations"""
    name: str
    maturity_date: Optional[str] = None
    cash_flow: Optional[float] = None
    candle: Optional[Dict[str, float]] = None
    instrument_type: Optional[str] = None
    rate: Optional[float] = None


class BankAccountInput(BaseModel):
    """Bank account input for calculations"""
    account_name: str
    balance: float
    debit_rate: Optional[float] = None
    credit_rate: Optional[float] = None
    currency: Optional[str] = None


class CashFlowTimelineRequest(BaseModel):
    """Request for cash flow timeline calculation"""
    positions: List[PositionInput]
    bank_accounts: List[BankAccountInput] = []
    projection_months: int = 12


class CashFlowTimelineResponse(BaseModel):
    """Response for cash flow timeline calculation"""
    events: List[Dict[str, Any]]
    monthly_flows: Dict[str, Dict[str, Any]]
    total_inflows: float
    total_outflows: float
    net_cash_flow: float


class OpportunitySimulationRequest(BaseModel):
    """Request for opportunity simulation"""
    positions: List[PositionInput]
    bank_accounts: List[BankAccountInput] = []


class ScenarioResponse(BaseModel):
    """Response for scenario"""
    id: str
    name: str
    type: str
    description: str
    parameters: Dict[str, float]
    net_benefit: float


class ScenarioCalculationRequest(BaseModel):
    """Request for scenario calculation"""
    scenario: Dict[str, any]


class ScenarioCalculationResponse(BaseModel):
    """Response for scenario calculation"""
    net_benefit: float
    cash_flow_impact: float
    risk_reduction: float
    capital_efficiency: Optional[float] = None


@app.post("/api/v1/cash-flow/timeline", response_model=CashFlowTimelineResponse)
async def calculate_cash_flow_timeline_endpoint(
    request: CashFlowTimelineRequest
) -> CashFlowTimelineResponse:
    """
    Calculate cash flow timeline from positions and bank accounts.
    """
    try:
        # Convert Pydantic models to dicts
        positions_dict = [
            {
                'name': p.name,
                'maturity_date': p.maturity_date,
                'cash_flow': p.cash_flow,
                'candle': p.candle or {},
                'instrument_type': p.instrument_type,
                'rate': p.rate
            }
            for p in request.positions
        ]

        bank_accounts_dict = [
            {
                'account_name': a.account_name,
                'balance': a.balance,
                'debit_rate': a.debit_rate,
                'credit_rate': a.credit_rate,
                'currency': a.currency
            }
            for a in request.bank_accounts
        ]

        # Calculate timeline
        result = calculate_cash_flow_timeline(
            positions=positions_dict,
            bank_accounts=bank_accounts_dict,
            projection_months=request.projection_months
        )

        # Convert to response format
        events_dict = [
            {
                'date': e.date,
                'amount': e.amount,
                'description': e.description,
                'position_name': e.position_name,
                'type': e.type
            }
            for e in result.events
        ]

        monthly_flows_dict = {
            month: {
                'month': m.month,
                'inflows': m.inflows,
                'outflows': m.outflows,
                'net': m.net,
                'events': [
                    {
                        'date': e.date,
                        'amount': e.amount,
                        'description': e.description,
                        'position_name': e.position_name,
                        'type': e.type
                    }
                    for e in m.events
                ]
            }
            for month, m in result.monthly_flows.items()
        }

        return CashFlowTimelineResponse(
            events=events_dict,
            monthly_flows=monthly_flows_dict,
            total_inflows=result.total_inflows,
            total_outflows=result.total_outflows,
            net_cash_flow=result.net_cash_flow
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/v1/opportunity-simulation/scenarios", response_model=List[ScenarioResponse])
async def find_scenarios_endpoint(
    request: OpportunitySimulationRequest
) -> List[ScenarioResponse]:
    """
    Find available opportunity simulation scenarios.
    """
    try:
        # Convert Pydantic models to dicts
        positions_dict = [
            {
                'name': p.name,
                'instrument_type': p.instrument_type,
                'cash_flow': p.cash_flow,
                'candle': p.candle or {},
                'rate': p.rate
            }
            for p in request.positions
        ]

        bank_accounts_dict = [
            {
                'account_name': a.account_name,
                'balance': a.balance,
                'debit_rate': a.debit_rate,
                'credit_rate': a.credit_rate
            }
            for a in request.bank_accounts
        ]

        # Find scenarios
        scenarios = find_available_scenarios(
            positions=positions_dict,
            bank_accounts=bank_accounts_dict
        )

        # Convert to response format
        return [
            ScenarioResponse(
                id=s.id,
                name=s.name,
                type=s.type,
                description=s.description,
                parameters=s.parameters,
                net_benefit=calculate_net_benefit(s)
            )
            for s in scenarios
        ]
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/v1/opportunity-simulation/calculate", response_model=ScenarioCalculationResponse)
async def calculate_scenario_endpoint(
    request: ScenarioCalculationRequest
) -> ScenarioCalculationResponse:
    """
    Calculate detailed results for a scenario.
    """
    try:
        # Convert dict to SimulationScenario
        scenario = SimulationScenario(
            id=request.scenario['id'],
            name=request.scenario['name'],
            type=request.scenario['type'],
            description=request.scenario['description'],
            parameters=request.scenario['parameters']
        )

        # Calculate results
        result = calculate_scenario_results(scenario)

        return ScenarioCalculationResponse(
            net_benefit=result.net_benefit,
            cash_flow_impact=result.cash_flow_impact,
            risk_reduction=result.risk_reduction,
            capital_efficiency=result.capital_efficiency
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


class CashManagementRequest(BaseModel):
    """Request for cash management analysis"""
    positions: List[PositionInput]
    bank_accounts: List[BankAccountInput] = []
    available_cash: float
    projection_months: int = 12


@app.post("/api/v1/cash-flow/management")
async def cash_management_endpoint(request: CashManagementRequest) -> Dict[str, Any]:
    """
    Analyze portfolio cash position against upcoming obligations.

    Returns reserves analysis, alerts, and allocation recommendations.
    """
    try:
        from python.integration.cash_flow_portfolio_manager import CashFlowPortfolioManager

        positions_dict = [
            {
                'name': p.name,
                'maturity_date': p.maturity_date,
                'cash_flow': p.cash_flow,
                'candle': p.candle or {},
                'instrument_type': p.instrument_type,
                'rate': p.rate,
            }
            for p in request.positions
        ]

        bank_accounts_dict = [
            {
                'account_name': a.account_name,
                'balance': a.balance,
                'debit_rate': a.debit_rate,
                'credit_rate': a.credit_rate,
                'currency': a.currency,
            }
            for a in request.bank_accounts
        ]

        manager = CashFlowPortfolioManager()
        snapshot = manager.analyze(
            positions=positions_dict,
            bank_accounts=bank_accounts_dict,
            available_cash=request.available_cash,
            projection_months=request.projection_months,
        )

        return snapshot.to_dict()
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/health")
async def health():
    """Health check endpoint"""
    return {"status": "ok", "service": "calculations-api"}


if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("CALCULATIONS_API_PORT", "8004"))
    uvicorn.run(app, host="0.0.0.0", port=port)
