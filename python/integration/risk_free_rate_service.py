"""
risk_free_rate_service.py - FastAPI service for risk-free rate extraction and comparison

Exposes endpoints for:
- Extracting risk-free rates from box spreads
- Building yield curves
- Comparing with SOFR/Treasury benchmarks
"""

from __future__ import annotations

import logging
from datetime import datetime
from typing import Dict, List, Optional, Any
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from pathlib import Path
import sys

# Add project root to path for security module
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from python.services.security_integration_helper import (
    add_security_to_app,
    add_security_headers_middleware
)

from .risk_free_rate_extractor import (
    RiskFreeRateExtractor,
    RiskFreeRatePoint
)
from .sofr_treasury_client import (
    SOFRTreasuryClient,
    BenchmarkRate,
    RateComparison
)

logger = logging.getLogger(__name__)

app = FastAPI(title="Risk-Free Rate Service", version="1.0.0")

# Add security components
security_components = add_security_to_app(app, project_root=project_root)
add_security_headers_middleware(app)

# Initialize clients
extractor = RiskFreeRateExtractor(min_liquidity_score=50.0)
benchmark_client = SOFRTreasuryClient()


# Pydantic models for API
class BoxSpreadInput(BaseModel):
    """Input for extracting rate from a single box spread."""
    symbol: str
    expiry: str
    days_to_expiry: int
    strike_width: float
    buy_implied_rate: float
    sell_implied_rate: float
    net_debit: float
    net_credit: float
    liquidity_score: float
    spread_id: Optional[str] = None


class RatePointResponse(BaseModel):
    """Response for a single risk-free rate point."""
    symbol: str
    expiry: str
    days_to_expiry: int
    strike_width: float
    buy_implied_rate: float
    sell_implied_rate: float
    mid_rate: float
    net_debit: float
    net_credit: float
    liquidity_score: float
    timestamp: str
    spread_id: Optional[str] = None


class CurveResponse(BaseModel):
    """Response for a risk-free rate curve."""
    symbol: str
    points: List[RatePointResponse]
    timestamp: str
    strike_width: Optional[float] = None
    point_count: int


class ComparisonResponse(BaseModel):
    """Response for rate comparison with benchmarks."""
    dte: int
    box_spread_rate: float
    benchmark_rate: float
    benchmark_type: str
    spread_bps: float
    liquidity_score: float
    timestamp: str


def _rate_point_to_dict(point: RiskFreeRatePoint) -> Dict[str, Any]:
    """Convert RiskFreeRatePoint to dictionary."""
    return {
        "symbol": point.symbol,
        "expiry": point.expiry,
        "days_to_expiry": point.days_to_expiry,
        "strike_width": point.strike_width,
        "buy_implied_rate": point.buy_implied_rate,
        "sell_implied_rate": point.sell_implied_rate,
        "mid_rate": point.mid_rate,
        "net_debit": point.net_debit,
        "net_credit": point.net_credit,
        "liquidity_score": point.liquidity_score,
        "timestamp": point.timestamp.isoformat(),
        "spread_id": point.spread_id
    }


@app.get("/api/health")
def health_check() -> Dict[str, Any]:
    """Health check endpoint."""
    return {
        "status": "ok",
        "service": "risk-free-rate",
        "timestamp": datetime.now().isoformat()
    }


@app.post("/api/extract-rate", response_model=RatePointResponse)
def extract_rate(box_spread: BoxSpreadInput) -> RatePointResponse:
    """
    Extract a risk-free rate point from a single box spread.

    This endpoint implements the methodology from:
    - New York Fed: Options for Calculating Risk-Free Rates
    - CME Group: Pricing SOFR Swaps with SOFR Futures
    """
    try:
        point = extractor.extract_from_box_spread(
            symbol=box_spread.symbol,
            expiry=box_spread.expiry,
            days_to_expiry=box_spread.days_to_expiry,
            strike_width=box_spread.strike_width,
            buy_implied_rate=box_spread.buy_implied_rate,
            sell_implied_rate=box_spread.sell_implied_rate,
            net_debit=box_spread.net_debit,
            net_credit=box_spread.net_credit,
            liquidity_score=box_spread.liquidity_score,
            spread_id=box_spread.spread_id
        )

        if not point:
            raise HTTPException(
                status_code=400,
                detail="Invalid box spread data or below minimum liquidity threshold"
            )

        return RatePointResponse(**_rate_point_to_dict(point))

    except Exception as e:
        logger.error(f"Error extracting rate: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/build-curve", response_model=CurveResponse)
def build_curve(opportunities: List[Dict[str, Any]], symbol: str) -> CurveResponse:
    """
    Build a risk-free rate curve from multiple box spread opportunities.

    Aggregates rates across different expirations to create a term structure.
    """
    try:
        curve = extractor.build_curve_from_opportunities(opportunities, symbol)

        points = [RatePointResponse(**_rate_point_to_dict(p)) for p in curve.points]

        return CurveResponse(
            symbol=curve.symbol,
            points=points,
            timestamp=curve.timestamp.isoformat(),
            strike_width=curve.strike_width,
            point_count=len(points)
        )

    except Exception as e:
        logger.error(f"Error building curve: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/benchmarks/sofr")
def get_sofr_rates() -> Dict[str, Any]:
    """Get current SOFR rates."""
    try:
        overnight = benchmark_client.get_sofr_overnight()
        term_rates = benchmark_client.get_sofr_term_rates()

        result = {
            "overnight": {
                "rate": overnight.rate if overnight else None,
                "timestamp": overnight.timestamp.isoformat() if overnight else None
            },
            "term_rates": [
                {
                    "tenor": r.tenor,
                    "rate": r.rate,
                    "days_to_expiry": r.days_to_expiry,
                    "timestamp": r.timestamp.isoformat()
                }
                for r in term_rates
            ],
            "timestamp": datetime.now().isoformat()
        }

        return result

    except Exception as e:
        logger.error(f"Error fetching SOFR rates: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/benchmarks/treasury")
def get_treasury_rates() -> Dict[str, Any]:
    """Get current Treasury rates."""
    try:
        rates = benchmark_client.get_treasury_rates()

        return {
            "rates": [
                {
                    "tenor": r.tenor,
                    "rate": r.rate,
                    "days_to_expiry": r.days_to_expiry,
                    "timestamp": r.timestamp.isoformat()
                }
                for r in rates
            ],
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Error fetching Treasury rates: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/compare", response_model=List[ComparisonResponse])
def compare_rates(
    opportunities: List[Dict[str, Any]],
    symbol: str
) -> List[ComparisonResponse]:
    """
    Compare box spread rates with SOFR/Treasury benchmarks.

    Returns spread analysis showing how box spread rates compare to
    traditional risk-free rate benchmarks.
    """
    try:
        # Build curve from opportunities
        curve = extractor.build_curve_from_opportunities(opportunities, symbol)

        # Get benchmark rates
        sofr_overnight = benchmark_client.get_sofr_overnight()
        sofr_term = benchmark_client.get_sofr_term_rates()
        treasury = benchmark_client.get_treasury_rates()

        all_benchmarks: List[BenchmarkRate] = []
        if sofr_overnight:
            all_benchmarks.append(sofr_overnight)
        all_benchmarks.extend(sofr_term)
        all_benchmarks.extend(treasury)

        # Compare
        comparison = RateComparison.compare_curves(curve, all_benchmarks)

        # Convert to response format
        results = [
            ComparisonResponse(
                dte=comp["dte"],
                box_spread_rate=comp["box_spread_rate"],
                benchmark_rate=comp["benchmark_rate"],
                benchmark_type=comp["benchmark_type"],
                spread_bps=comp["spread_bps"],
                liquidity_score=comp["liquidity_score"],
                timestamp=comp["timestamp"]
            )
            for comp in comparison.values()
        ]

        return results

    except Exception as e:
        logger.error(f"Error comparing rates: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/yield-curve/comparison")
def yield_curve_comparison(
    box_spread_rates: Dict[str, Dict[str, float]],
    treasury_rates: Optional[Dict[str, float]] = None,
    sofr_rates: Optional[Dict[str, float]] = None,
    fetch_live: bool = False,
) -> Dict[str, Any]:
    """
    Overlay Treasury/SOFR yield curves on box spread implied rate curves.

    Request body:
        box_spread_rates: {symbol: {dte_str: rate_pct}}
        treasury_rates:   optional {dte_str: rate_pct}
        sofr_rates:       optional {dte_str: rate_pct}
        fetch_live:       if true, fetch live rates when manual not provided

    Returns YieldCurveComparison with spread analysis at each tenor point.
    """
    try:
        from .yield_curve_comparison import YieldCurveComparer

        comparer = YieldCurveComparer(
            treasury_client=None,
            sofr_client=benchmark_client if fetch_live else None,
        )

        box_rates_int = {
            sym: {int(dte): rate for dte, rate in rates.items()}
            for sym, rates in box_spread_rates.items()
        }
        treas_int = {int(k): v for k, v in treasury_rates.items()} if treasury_rates else None
        sofr_int = {int(k): v for k, v in sofr_rates.items()} if sofr_rates else None

        comparison = comparer.compare(
            box_spread_rates=box_rates_int,
            treasury_rates=treas_int,
            sofr_rates=sofr_int,
            fetch_live=fetch_live,
        )

        return comparison.to_dict()

    except Exception as e:
        logger.error(f"Error comparing yield curves: {e}")
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=8004)
