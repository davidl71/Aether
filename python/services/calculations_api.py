"""
Calculations API Service

FastAPI service exposing Python-specific calculation endpoints.
The shared frontend read-model endpoints now live in the Rust API.

Endpoints:
- POST /api/v1/cash-flow/management - Analyze reserves and allocation recommendations
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import List, Dict, Optional, Any

from fastapi import FastAPI, HTTPException, APIRouter
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

# Router for mounting in analytics_api or running standalone
router_calculations = APIRouter()


# Request/Response Models
class PositionInput(BaseModel):
    """Position input for calculations"""
    name: str
    quantity: Optional[int] = None
    roi: Optional[float] = None
    maker_count: Optional[int] = None
    taker_count: Optional[int] = None
    rebate_estimate: Optional[float] = None
    vega: Optional[float] = None
    theta: Optional[float] = None
    fair_diff: Optional[float] = None
    maturity_date: Optional[str] = None
    cash_flow: Optional[float] = None
    candle: Optional[Dict[str, float]] = None
    instrument_type: Optional[str] = None
    rate: Optional[float] = None
    collateral_value: Optional[float] = None
    currency: Optional[str] = None
    market_value: Optional[float] = None
    bid: Optional[float] = None
    ask: Optional[float] = None
    last: Optional[float] = None
    spread: Optional[float] = None
    price: Optional[float] = None
    side: Optional[str] = None
    expected_cash_at_expiry: Optional[float] = None
    dividend: Optional[float] = None
    conid: Optional[int] = None


class BankAccountInput(BaseModel):
    """Bank account input for calculations"""
    account_name: str
    balance: float
    account_path: Optional[str] = None
    bank_name: Optional[str] = None
    account_number: Optional[str] = None
    debit_rate: Optional[float] = None
    credit_rate: Optional[float] = None
    currency: Optional[str] = None
    balances_by_currency: Optional[Dict[str, float]] = None
    is_mixed_currency: bool = False


class CashManagementRequest(BaseModel):
    """Request for cash management analysis"""
    positions: List[PositionInput]
    bank_accounts: List[BankAccountInput] = []
    available_cash: float
    projection_months: int = 12


@router_calculations.post("/api/v1/cash-flow/management")
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
                'quantity': p.quantity,
                'roi': p.roi,
                'maker_count': p.maker_count,
                'taker_count': p.taker_count,
                'rebate_estimate': p.rebate_estimate,
                'vega': p.vega,
                'theta': p.theta,
                'fair_diff': p.fair_diff,
                'maturity_date': p.maturity_date,
                'cash_flow': p.cash_flow,
                'candle': p.candle or {},
                'instrument_type': p.instrument_type,
                'rate': p.rate,
                'collateral_value': p.collateral_value,
                'currency': p.currency,
            }
            for p in request.positions
        ]

        bank_accounts_dict = [
            {
                'account_name': a.account_name,
                'balance': a.balance,
                'account_path': a.account_path,
                'bank_name': a.bank_name,
                'account_number': a.account_number,
                'debit_rate': a.debit_rate,
                'credit_rate': a.credit_rate,
                'currency': a.currency,
                'balances_by_currency': a.balances_by_currency,
                'is_mixed_currency': a.is_mixed_currency,
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


@router_calculations.get("/health")
async def health():
    """Health check endpoint"""
    return {"status": "ok", "service": "calculations-api"}


# Standalone app (same as before)
app = FastAPI(
    title="Calculations API",
    description="REST API for Python-specific calculation endpoints",
    version="1.0.0"
)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
app.include_router(router_calculations)


if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("CALCULATIONS_API_PORT", "8004"))
    uvicorn.run(app, host="0.0.0.0", port=port)
