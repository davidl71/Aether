#!/usr/bin/env python3
"""
Swiftness API Service - FastAPI wrapper for Swiftness integration
Exposes Swiftness positions, cash flows, and validation via REST API
"""
import logging
import sys
from datetime import datetime, timedelta
from pathlib import Path

from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.integration.swiftness_storage import SwiftnessStorage
from python.integration.swiftness_integration import SwiftnessIntegration, PositionSnapshot, CashFlowEvent

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='[%(asctime)s] [%(levelname)s] [%(name)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="Swiftness API",
    description="REST API for Swiftness pension fund position integration",
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

# Initialize Swiftness integration
storage_path = Path.home() / ".config" / "ib_box_spread" / "swiftness"
storage = SwiftnessStorage(storage_path)
integration = SwiftnessIntegration(storage, ils_to_usd_rate=0.27)


# Pydantic models for API requests/responses
class ExchangeRateUpdate(BaseModel):
    rate: float = Field(..., gt=0, description="ILS to USD exchange rate")


class CashFlowRequest(BaseModel):
    start_date: datetime
    end_date: datetime
    check_validity: bool = True
    max_age_days: int = 90


class PositionSnapshotResponse(BaseModel):
    id: str
    symbol: str
    quantity: int
    cost_basis: float
    mark: float
    unrealized_pnl: float

    @classmethod
    def from_snapshot(cls, snapshot: PositionSnapshot) -> "PositionSnapshotResponse":
        return cls(
            id=snapshot.id,
            symbol=snapshot.symbol,
            quantity=snapshot.quantity,
            cost_basis=snapshot.cost_basis,
            mark=snapshot.mark,
            unrealized_pnl=snapshot.unrealized_pnl,
        )


class CashFlowEventResponse(BaseModel):
    date: datetime
    amount: float
    currency: str
    description: str
    source: str

    @classmethod
    def from_event(cls, event: CashFlowEvent) -> "CashFlowEventResponse":
        return cls(
            date=event.date,
            amount=event.amount,
            currency=event.currency,
            description=event.description,
            source=event.source,
        )


class ValidationReportResponse(BaseModel):
    total_products: int
    valid_products: list[str]
    stale_products: list[str]
    last_updated: datetime | None


class PortfolioValueResponse(BaseModel):
    total_value_usd: float
    currency: str = "USD"


# API Endpoints

@app.get("/health")
async def health():
    """Health check endpoint"""
    return {"status": "ok", "service": "swiftness-api"}


@app.get("/positions", response_model=list[PositionSnapshotResponse])
async def get_positions(
    check_validity: bool = Query(True, description="Only return positions with valid data"),
    max_age_days: int = Query(90, ge=1, le=365, description="Maximum age of position data in days"),
):
    """
    Get Swiftness positions as PositionSnapshot format.

    Returns positions compatible with backend SystemSnapshot.positions structure.
    """
    try:
        positions = integration.get_positions(
            check_validity=check_validity,
            max_age_days=max_age_days
        )
        return [PositionSnapshotResponse.from_snapshot(pos) for pos in positions]
    except Exception as e:
        logger.error(f"Error fetching positions: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to fetch positions: {str(e)}")


@app.get("/portfolio-value", response_model=PortfolioValueResponse)
async def get_portfolio_value(
    check_validity: bool = Query(True, description="Only include valid positions"),
    max_age_days: int = Query(90, ge=1, le=365, description="Maximum age of position data in days"),
):
    """Get total Swiftness portfolio value in USD"""
    try:
        total_value = integration.get_portfolio_value(
            check_validity=check_validity,
            max_age_days=max_age_days
        )
        return PortfolioValueResponse(total_value_usd=total_value)
    except Exception as e:
        logger.error(f"Error calculating portfolio value: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to calculate portfolio value: {str(e)}")


@app.get("/cash-flows", response_model=list[CashFlowEventResponse])
async def get_cash_flows(
    start_date: datetime = Query(..., description="Start date for cash flow forecast"),
    end_date: datetime = Query(..., description="End date for cash flow forecast"),
    check_validity: bool = Query(True, description="Only include valid positions"),
    max_age_days: int = Query(90, ge=1, le=365, description="Maximum age of position data in days"),
):
    """Get cash flow forecast for Swiftness positions"""
    try:
        if end_date <= start_date:
            raise HTTPException(status_code=400, detail="end_date must be after start_date")

        cash_flows = integration.get_cash_flows(
            start_date=start_date,
            end_date=end_date,
            check_validity=check_validity,
            max_age_days=max_age_days
        )
        return [CashFlowEventResponse.from_event(cf) for cf in cash_flows]
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error fetching cash flows: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to fetch cash flows: {str(e)}")


@app.get("/validate", response_model=ValidationReportResponse)
async def validate_positions():
    """Validate Swiftness positions and return validation report"""
    try:
        report = integration.validate_positions()
        return ValidationReportResponse(
            total_products=report["total_products"],
            valid_products=report["valid_products"],
            stale_products=report["stale_products"],
            last_updated=report["last_updated"],
        )
    except Exception as e:
        logger.error(f"Error validating positions: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to validate positions: {str(e)}")


@app.get("/exchange-rate")
async def get_exchange_rate():
    """Get current ILS to USD exchange rate"""
    return {"rate": integration.ils_to_usd_rate, "currency": "ILS/USD"}


@app.put("/exchange-rate")
async def update_exchange_rate(update: ExchangeRateUpdate):
    """Update ILS to USD exchange rate"""
    try:
        integration.update_exchange_rate(update.rate)
        return {"status": "ok", "rate": integration.ils_to_usd_rate, "message": "Exchange rate updated"}
    except Exception as e:
        logger.error(f"Error updating exchange rate: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to update exchange rate: {str(e)}")


@app.get("/greeks")
async def get_greeks(
    check_validity: bool = Query(True, description="Only include valid positions"),
    max_age_days: int = Query(90, ge=1, le=365, description="Maximum age of position data in days"),
):
    """Get Greeks for Swiftness positions"""
    try:
        greeks = integration.get_greeks(
            check_validity=check_validity,
            max_age_days=max_age_days
        )
        return {"greeks": greeks}
    except Exception as e:
        logger.error(f"Error calculating Greeks: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to calculate Greeks: {str(e)}")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=8081, log_level="info")
