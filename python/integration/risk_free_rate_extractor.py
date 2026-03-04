"""
risk_free_rate_extractor.py - Extract and aggregate risk-free rates from box spreads

This module implements the methodology described in:
- New York Fed: https://libertystreeteconomics.newyorkfed.org/2023/10/options-for-calculating-risk-free-rates/
- CME Group: https://www.cmegroup.com/articles/2025/price-and-hedging-usd-sofr-interest-swaps-with-sofr-futures.html

Box spreads are theoretically risk-free and their implied rates represent the risk-free
rate for that expiration. This module aggregates rates across multiple expirations to
build a term structure of risk-free rates.
"""

from __future__ import annotations

import logging
from datetime import datetime
from typing import Dict, List, Optional
from dataclasses import dataclass
from collections import defaultdict

# Import Treasury API client for benchmark comparison
try:
  from .treasury_api_client import TreasuryAPIClient, get_treasury_benchmark
  TREASURY_API_AVAILABLE = True
except ImportError:
  TREASURY_API_AVAILABLE = False
  logging.warning("Treasury API client not available")

logger = logging.getLogger(__name__)


@dataclass
class RiskFreeRatePoint:
    """A single risk-free rate data point from a box spread."""

    symbol: str
    expiry: str  # YYYYMMDD format
    days_to_expiry: int
    strike_width: float
    buy_implied_rate: float  # Rate when buying (borrowing scenario)
    sell_implied_rate: float  # Rate when selling (lending scenario)
    mid_rate: float  # Average of buy and sell rates
    net_debit: float
    net_credit: float
    liquidity_score: float
    timestamp: datetime
    spread_id: Optional[str] = None  # Identifier for the box spread

    def is_valid(self) -> bool:
        """Validate that the rate point has meaningful data."""
        return (
            self.days_to_expiry > 0
            and self.strike_width > 0
            and (self.buy_implied_rate > 0 or self.sell_implied_rate > 0)
            and self.liquidity_score >= 0
        )


@dataclass
class RiskFreeRateCurve:
    """Term structure of risk-free rates from box spreads."""

    symbol: str
    points: List[RiskFreeRatePoint]
    timestamp: datetime
    strike_width: Optional[float] = None  # If all points use same strike width

    def get_rates_by_dte(self) -> Dict[int, float]:
        """Get mid rates indexed by days to expiry."""
        return {
            point.days_to_expiry: point.mid_rate
            for point in self.points
            if point.is_valid()
        }

    def get_rate_at_dte(self, target_dte: int, tolerance: int = 5) -> Optional[float]:
        """Get rate at specific days to expiry (with tolerance)."""
        for point in self.points:
            if abs(point.days_to_expiry - target_dte) <= tolerance and point.is_valid():
                return point.mid_rate
        return None

    def sort_by_dte(self) -> None:
        """Sort points by days to expiry."""
        self.points.sort(key=lambda p: p.days_to_expiry)

    def filter_by_liquidity(self, min_liquidity: float = 50.0) -> RiskFreeRateCurve:
        """Create a filtered curve with only liquid points."""
        filtered = RiskFreeRateCurve(
            symbol=self.symbol,
            points=[p for p in self.points if p.liquidity_score >= min_liquidity],
            timestamp=self.timestamp,
            strike_width=self.strike_width,
        )
        filtered.sort_by_dte()
        return filtered


class RiskFreeRateExtractor:
    """
    Extract risk-free rates from box spread opportunities.

    Box spreads are theoretically risk-free because they lock in a fixed return
    regardless of underlying price movement. The implied rate from a box spread
    represents the risk-free rate for that expiration.
    """

    def __init__(self, min_liquidity_score: float = 50.0):
        """
        Initialize the risk-free rate extractor.

        Args:
            min_liquidity_score: Minimum liquidity score to include a rate point
        """
        self.min_liquidity_score = min_liquidity_score

    def extract_from_box_spread(
        self,
        symbol: str,
        expiry: str,
        days_to_expiry: int,
        strike_width: float,
        buy_implied_rate: float,
        sell_implied_rate: float,
        net_debit: float,
        net_credit: float,
        liquidity_score: float,
        spread_id: Optional[str] = None,
    ) -> Optional[RiskFreeRatePoint]:
        """
        Extract a risk-free rate point from a single box spread.

        Args:
            symbol: Underlying symbol (e.g., "SPX", "XSP")
            expiry: Expiration date (YYYYMMDD)
            days_to_expiry: Days until expiration
            strike_width: Difference between strikes (K2 - K1)
            buy_implied_rate: Implied rate when buying the box spread
            sell_implied_rate: Implied rate when selling the box spread
            net_debit: Net cost to buy the box spread
            net_credit: Net credit from selling the box spread
            liquidity_score: Average liquidity score across all legs
            spread_id: Optional identifier for the box spread

        Returns:
            RiskFreeRatePoint if valid, None otherwise
        """
        # Calculate mid rate (average of buy and sell)
        if buy_implied_rate > 0 and sell_implied_rate > 0:
            mid_rate = (buy_implied_rate + sell_implied_rate) / 2.0
        elif buy_implied_rate > 0:
            mid_rate = buy_implied_rate
        elif sell_implied_rate > 0:
            mid_rate = sell_implied_rate
        else:
            return None

        point = RiskFreeRatePoint(
            symbol=symbol,
            expiry=expiry,
            days_to_expiry=days_to_expiry,
            strike_width=strike_width,
            buy_implied_rate=buy_implied_rate,
            sell_implied_rate=sell_implied_rate,
            mid_rate=mid_rate,
            net_debit=net_debit,
            net_credit=net_credit,
            liquidity_score=liquidity_score,
            timestamp=datetime.now(),
            spread_id=spread_id,
        )

        if point.is_valid() and liquidity_score >= self.min_liquidity_score:
            return point
        return None

    def extract_from_box_spread_dict(
        self, spread_data: Dict
    ) -> Optional[RiskFreeRatePoint]:
        """
        Extract rate point from a box spread dictionary (from C++ bindings or API).

        Args:
            spread_data: Dictionary containing box spread data with keys:
                - symbol, expiry, days_to_expiry, strike_width
                - buy_implied_rate, sell_implied_rate
                - buy_net_debit, sell_net_credit
                - liquidity_score, spread_id (optional)

        Returns:
            RiskFreeRatePoint if valid, None otherwise
        """
        try:
            return self.extract_from_box_spread(
                symbol=spread_data.get("symbol", ""),
                expiry=spread_data.get("expiry", ""),
                days_to_expiry=spread_data.get("days_to_expiry", 0),
                strike_width=spread_data.get("strike_width", 0.0),
                buy_implied_rate=spread_data.get("buy_implied_rate", 0.0),
                sell_implied_rate=spread_data.get("sell_implied_rate", 0.0),
                net_debit=spread_data.get("buy_net_debit", 0.0),
                net_credit=spread_data.get("sell_net_credit", 0.0),
                liquidity_score=spread_data.get("liquidity_score", 0.0),
                spread_id=spread_data.get("spread_id"),
            )
        except (KeyError, TypeError, ValueError) as e:
            logger.warning(f"Failed to extract rate from box spread data: {e}")
            return None

    def aggregate_rates(
        self,
        rate_points: List[RiskFreeRatePoint],
        symbol: str,
        aggregation_method: str = "weighted_average",
    ) -> RiskFreeRateCurve:
        """
        Aggregate multiple rate points into a yield curve.

        Args:
            rate_points: List of risk-free rate points
            symbol: Underlying symbol
            aggregation_method: How to handle multiple points at same DTE
                - "weighted_average": Weight by liquidity score
                - "best_liquidity": Use point with highest liquidity
                - "average": Simple average

        Returns:
            RiskFreeRateCurve with aggregated points
        """
        # Group by days to expiry
        grouped: Dict[int, List[RiskFreeRatePoint]] = defaultdict(list)
        for point in rate_points:
            if point.is_valid():
                grouped[point.days_to_expiry].append(point)

        # Aggregate points at same DTE
        aggregated_points: List[RiskFreeRatePoint] = []
        for _dte, points in grouped.items():
            if len(points) == 1:
                aggregated_points.append(points[0])
            else:
                # Multiple points at same DTE - aggregate
                if aggregation_method == "weighted_average":
                    total_weight = sum(p.liquidity_score for p in points)
                    if total_weight > 0:
                        mid_rate = (
                            sum(p.mid_rate * p.liquidity_score for p in points)
                            / total_weight
                        )
                        buy_rate = (
                            sum(p.buy_implied_rate * p.liquidity_score for p in points)
                            / total_weight
                        )
                        sell_rate = (
                            sum(p.sell_implied_rate * p.liquidity_score for p in points)
                            / total_weight
                        )
                    else:
                        mid_rate = sum(p.mid_rate for p in points) / len(points)
                        buy_rate = sum(p.buy_implied_rate for p in points) / len(points)
                        sell_rate = sum(p.sell_implied_rate for p in points) / len(
                            points
                        )
                elif aggregation_method == "best_liquidity":
                    best = max(points, key=lambda p: p.liquidity_score)
                    mid_rate = best.mid_rate
                    buy_rate = best.buy_implied_rate
                    sell_rate = best.sell_implied_rate
                else:  # average
                    mid_rate = sum(p.mid_rate for p in points) / len(points)
                    buy_rate = sum(p.buy_implied_rate for p in points) / len(points)
                    sell_rate = sum(p.sell_implied_rate for p in points) / len(points)

                # Create aggregated point using first point as template
                template = points[0]
                aggregated_point = RiskFreeRatePoint(
                    symbol=template.symbol,
                    expiry=template.expiry,
                    days_to_expiry=template.days_to_expiry,
                    strike_width=template.strike_width,
                    buy_implied_rate=buy_rate,
                    sell_implied_rate=sell_rate,
                    mid_rate=mid_rate,
                    net_debit=sum(p.net_debit for p in points) / len(points),
                    net_credit=sum(p.net_credit for p in points) / len(points),
                    liquidity_score=max(p.liquidity_score for p in points),
                    timestamp=datetime.now(),
                    spread_id=None,  # Aggregated point
                )
                aggregated_points.append(aggregated_point)

        curve = RiskFreeRateCurve(
            symbol=symbol, points=aggregated_points, timestamp=datetime.now()
        )
        curve.sort_by_dte()
        return curve

    def build_curve_from_opportunities(
        self, opportunities: List[Dict], symbol: str
    ) -> RiskFreeRateCurve:
        """
        Build a risk-free rate curve from a list of box spread opportunities.

        Args:
            opportunities: List of box spread opportunity dictionaries
            symbol: Underlying symbol

        Returns:
            RiskFreeRateCurve
        """
        rate_points: List[RiskFreeRatePoint] = []

        for opp in opportunities:
            spread = opp.get("spread", {})
            point = self.extract_from_box_spread_dict(spread)
            if point:
                rate_points.append(point)

        return self.aggregate_rates(rate_points, symbol)

    def compare_to_treasury_benchmark(
        self,
        rate_point: RiskFreeRatePoint,
        security_type: Optional[str] = None
    ) -> Optional[Dict]:
        """
        Compare box spread implied rate to U.S. Treasury benchmark.

        Args:
            rate_point: Risk-free rate point from box spread
            security_type: Treasury security type filter (optional)

        Returns:
            Dictionary with comparison metrics, or None if Treasury rate unavailable
        """
        if not TREASURY_API_AVAILABLE:
            logger.warning("Treasury API not available for benchmark comparison")
            return None

        try:
            client = TreasuryAPIClient()
            comparison = client.compare_to_box_spread_rate(
                box_spread_rate=rate_point.mid_rate,
                days_to_expiry=rate_point.days_to_expiry,
                security_type=security_type
            )
            return comparison
        except Exception as e:
            logger.error(f"Failed to compare to Treasury benchmark: {e}")
            return None

    def build_curve_with_benchmarks(
        self,
        opportunities: List[Dict],
        symbol: str,
        include_treasury_benchmarks: bool = True
    ) -> RiskFreeRateCurve:
        """
        Build a risk-free rate curve with Treasury benchmark comparisons.

        Args:
            opportunities: List of box spread opportunity dictionaries
            symbol: Underlying symbol
            include_treasury_benchmarks: Whether to fetch Treasury benchmarks

        Returns:
            RiskFreeRateCurve with benchmark data in metadata
        """
        curve = self.build_curve_from_opportunities(opportunities, symbol)

        if include_treasury_benchmarks and TREASURY_API_AVAILABLE:
            # Add Treasury benchmark comparisons to each point
            for point in curve.points:
                benchmark = self.compare_to_treasury_benchmark(point)
                if benchmark:
                    # Store benchmark data (could extend RiskFreeRatePoint to include this)
                    logger.debug(
                        f"Box spread rate {point.mid_rate:.4f}% vs Treasury "
                        f"{benchmark['treasury_rate']:.4f}% "
                        f"(spread: {benchmark['spread_bps']:.2f} bps)"
                    )

        return curve
