"""
yield_curve_comparison.py - Overlay Treasury/SOFR yield curves on box spread implied rate curves

Provides side-by-side comparison of:
- Box spread implied rates across multiple symbols (SPX, XSP, ES)
- U.S. Treasury constant-maturity rates (1M through 30Y)
- SOFR overnight and term rates

Produces a unified YieldCurveComparison with spread analysis at each tenor point.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from datetime import datetime
from typing import Dict, List, Optional, Tuple

logger = logging.getLogger(__name__)

try:
    from .treasury_api_client import TreasuryAPIClient
    TREASURY_API_AVAILABLE = True
except ImportError:
    TREASURY_API_AVAILABLE = False

try:
    from .sofr_treasury_client import SOFRTreasuryClient, BenchmarkRate
    SOFR_AVAILABLE = True
except ImportError:
    SOFR_AVAILABLE = False

try:
    from .risk_free_rate_extractor import RiskFreeRateExtractor, RiskFreeRateCurve
    EXTRACTOR_AVAILABLE = True
except ImportError:
    EXTRACTOR_AVAILABLE = False

try:
    from .financing_comparator import TaxConfig
    TAX_CONFIG_AVAILABLE = True
except ImportError:
    TAX_CONFIG_AVAILABLE = False


@dataclass
class CurvePoint:
    """A single point on any yield curve."""
    days_to_expiry: int
    rate_pct: float
    source: str
    tenor_label: str = ""
    liquidity_score: float = 0.0
    timestamp: Optional[datetime] = None


@dataclass
class SpreadPoint:
    """Spread between box spread and benchmark at a single tenor."""
    days_to_expiry: int
    tenor_label: str
    box_rate_pct: float
    benchmark_rate_pct: float
    benchmark_source: str
    spread_bps: float
    box_symbol: str = ""
    box_liquidity: float = 0.0

    @property
    def box_wins(self) -> bool:
        return self.spread_bps > 5.0

    @property
    def benchmark_wins(self) -> bool:
        return self.spread_bps < -5.0


@dataclass
class YieldCurveComparison:
    """Complete yield curve comparison across instruments."""
    box_curves: Dict[str, List[CurvePoint]] = field(default_factory=dict)
    treasury_curve: List[CurvePoint] = field(default_factory=list)
    sofr_curve: List[CurvePoint] = field(default_factory=list)
    spreads: List[SpreadPoint] = field(default_factory=list)
    generated_time: datetime = field(default_factory=datetime.now)

    @property
    def all_tenors(self) -> List[int]:
        tenors = set()
        for pts in self.box_curves.values():
            tenors.update(p.days_to_expiry for p in pts)
        tenors.update(p.days_to_expiry for p in self.treasury_curve)
        tenors.update(p.days_to_expiry for p in self.sofr_curve)
        return sorted(tenors)

    @property
    def symbols(self) -> List[str]:
        return sorted(self.box_curves.keys())

    def get_benchmark_at_dte(self, dte: int, tolerance: int = 15) -> Optional[CurvePoint]:
        """Find closest Treasury or SOFR point to a given DTE."""
        all_bench = self.treasury_curve + self.sofr_curve
        best: Optional[CurvePoint] = None
        best_diff = float("inf")
        for pt in all_bench:
            diff = abs(pt.days_to_expiry - dte)
            if diff <= tolerance and diff < best_diff:
                best_diff = diff
                best = pt
        return best

    def summary(self) -> Dict:
        box_wins = sum(1 for s in self.spreads if s.box_wins)
        bench_wins = sum(1 for s in self.spreads if s.benchmark_wins)
        return {
            "symbols": self.symbols,
            "tenor_count": len(self.all_tenors),
            "treasury_points": len(self.treasury_curve),
            "sofr_points": len(self.sofr_curve),
            "spread_points": len(self.spreads),
            "box_spread_wins": box_wins,
            "benchmark_wins": bench_wins,
            "ties": len(self.spreads) - box_wins - bench_wins,
            "generated": self.generated_time.isoformat(),
        }

    def to_dict(self) -> Dict:
        result = self.summary()
        result["box_curves"] = {
            sym: [
                {"dte": p.days_to_expiry, "rate": p.rate_pct, "liquidity": p.liquidity_score}
                for p in pts
            ]
            for sym, pts in self.box_curves.items()
        }
        result["treasury_curve"] = [
            {"dte": p.days_to_expiry, "rate": p.rate_pct, "tenor": p.tenor_label, "source": p.source}
            for p in self.treasury_curve
        ]
        result["sofr_curve"] = [
            {"dte": p.days_to_expiry, "rate": p.rate_pct, "tenor": p.tenor_label, "source": p.source}
            for p in self.sofr_curve
        ]
        result["spreads"] = [
            {
                "dte": s.days_to_expiry,
                "tenor": s.tenor_label,
                "box_rate": s.box_rate_pct,
                "benchmark_rate": s.benchmark_rate_pct,
                "benchmark_source": s.benchmark_source,
                "spread_bps": s.spread_bps,
                "symbol": s.box_symbol,
            }
            for s in self.spreads
        ]
        return result


# Standard tenor labels
TENOR_LABELS = {
    1: "O/N", 30: "1M", 60: "2M", 90: "3M", 120: "4M",
    180: "6M", 270: "9M", 365: "1Y", 730: "2Y", 1095: "3Y",
    1825: "5Y", 2555: "7Y", 3650: "10Y", 7300: "20Y", 10950: "30Y",
}


def _tenor_label(dte: int) -> str:
    if dte in TENOR_LABELS:
        return TENOR_LABELS[dte]
    for key_dte, label in sorted(TENOR_LABELS.items()):
        if abs(dte - key_dte) <= 5:
            return label
    if dte < 30:
        return f"{dte}d"
    months = round(dte / 30)
    if months <= 12:
        return f"{months}M"
    years = round(dte / 365, 1)
    return f"{years}Y"


class YieldCurveComparer:
    """
    Build overlay comparisons of box spread yield curves with Treasury/SOFR benchmarks.

    Usage:
        comparer = YieldCurveComparer()
        comparison = comparer.compare(
            box_spread_rates={"SPX": {30: 4.25, 90: 4.50, 180: 4.75}},
            treasury_rates={30: 4.50, 90: 4.75, 180: 5.00},
        )
        print(comparer.format_text(comparison))
    """

    def __init__(
        self,
        treasury_client: Optional["TreasuryAPIClient"] = None,
        sofr_client: Optional["SOFRTreasuryClient"] = None,
    ):
        self._treasury_client = treasury_client
        self._sofr_client = sofr_client

    @property
    def treasury_client(self) -> Optional["TreasuryAPIClient"]:
        if self._treasury_client is None and TREASURY_API_AVAILABLE:
            self._treasury_client = TreasuryAPIClient()
        return self._treasury_client

    @property
    def sofr_client(self) -> Optional["SOFRTreasuryClient"]:
        if self._sofr_client is None and SOFR_AVAILABLE:
            self._sofr_client = SOFRTreasuryClient()
        return self._sofr_client

    def fetch_treasury_curve(self) -> List[CurvePoint]:
        """Fetch live Treasury constant-maturity rates."""
        client = self.treasury_client
        if client is None:
            return []

        points: List[CurvePoint] = []
        tenor_map = {
            30: "1-Month", 90: "3-Month", 180: "6-Month",
            365: "1-Year", 730: "2-Year", 1095: "3-Year",
            1825: "5-Year", 3650: "10-Year",
        }

        for dte, term in tenor_map.items():
            try:
                rate = client.get_latest_rate(term)
                if rate and rate.avg_interest_rate > 0:
                    points.append(CurvePoint(
                        days_to_expiry=dte,
                        rate_pct=rate.avg_interest_rate,
                        source=f"Treasury {term} ({rate.record_date})",
                        tenor_label=_tenor_label(dte),
                        timestamp=datetime.now(),
                    ))
            except Exception as e:
                logger.warning(f"Failed to fetch Treasury {term}: {e}")

        points.sort(key=lambda p: p.days_to_expiry)
        return points

    def fetch_sofr_curve(self) -> List[CurvePoint]:
        """Fetch live SOFR rates."""
        client = self.sofr_client
        if client is None:
            return []

        points: List[CurvePoint] = []

        try:
            overnight = client.get_sofr_overnight()
            if overnight:
                points.append(CurvePoint(
                    days_to_expiry=1,
                    rate_pct=overnight.rate,
                    source=overnight.source,
                    tenor_label="O/N",
                    timestamp=datetime.now(),
                ))
        except Exception as e:
            logger.warning(f"Failed to fetch SOFR overnight: {e}")

        try:
            term_rates = client.get_sofr_term_rates()
            for rate in term_rates:
                if rate.days_to_expiry:
                    points.append(CurvePoint(
                        days_to_expiry=rate.days_to_expiry,
                        rate_pct=rate.rate,
                        source=rate.source,
                        tenor_label=rate.tenor,
                        timestamp=datetime.now(),
                    ))
        except Exception as e:
            logger.warning(f"Failed to fetch SOFR term rates: {e}")

        points.sort(key=lambda p: p.days_to_expiry)
        return points

    def compare(
        self,
        box_spread_rates: Dict[str, Dict[int, float]],
        treasury_rates: Optional[Dict[int, float]] = None,
        sofr_rates: Optional[Dict[int, float]] = None,
        liquidity_scores: Optional[Dict[str, Dict[int, float]]] = None,
        fetch_live: bool = False,
    ) -> YieldCurveComparison:
        """
        Build a yield curve comparison.

        Args:
            box_spread_rates: {symbol: {dte: rate_pct}} for each symbol
            treasury_rates: Optional manual {dte: rate_pct}. If None and fetch_live,
                            fetches from Treasury API.
            sofr_rates: Optional manual {dte: rate_pct}. If None and fetch_live,
                        fetches from FRED API.
            liquidity_scores: Optional {symbol: {dte: liquidity}} matching box_spread_rates
            fetch_live: If True, fetch live Treasury/SOFR rates when manual rates not provided

        Returns:
            YieldCurveComparison with all curves and spread analysis
        """
        comparison = YieldCurveComparison(generated_time=datetime.now())

        # Build box spread curves
        for symbol, rates in box_spread_rates.items():
            pts: List[CurvePoint] = []
            for dte, rate in sorted(rates.items()):
                liq = 0.0
                if liquidity_scores and symbol in liquidity_scores:
                    liq = liquidity_scores[symbol].get(dte, 0.0)
                pts.append(CurvePoint(
                    days_to_expiry=dte,
                    rate_pct=rate,
                    source=f"box_spread:{symbol}",
                    tenor_label=_tenor_label(dte),
                    liquidity_score=liq,
                    timestamp=datetime.now(),
                ))
            comparison.box_curves[symbol] = pts

        # Build Treasury curve
        if treasury_rates is not None:
            comparison.treasury_curve = [
                CurvePoint(
                    days_to_expiry=dte, rate_pct=rate,
                    source="manual", tenor_label=_tenor_label(dte),
                )
                for dte, rate in sorted(treasury_rates.items())
            ]
        elif fetch_live:
            comparison.treasury_curve = self.fetch_treasury_curve()

        # Build SOFR curve
        if sofr_rates is not None:
            comparison.sofr_curve = [
                CurvePoint(
                    days_to_expiry=dte, rate_pct=rate,
                    source="manual", tenor_label=_tenor_label(dte),
                )
                for dte, rate in sorted(sofr_rates.items())
            ]
        elif fetch_live:
            comparison.sofr_curve = self.fetch_sofr_curve()

        # Compute spreads at each box spread tenor
        self._compute_spreads(comparison)

        return comparison

    def compare_from_extractor_curves(
        self,
        curves: Dict[str, "RiskFreeRateCurve"],
        treasury_rates: Optional[Dict[int, float]] = None,
        fetch_live: bool = False,
    ) -> YieldCurveComparison:
        """
        Build comparison from RiskFreeRateExtractor curves.

        Args:
            curves: {symbol: RiskFreeRateCurve} from the extractor module
            treasury_rates: Optional manual Treasury rates
            fetch_live: Fetch live rates if manual not provided
        """
        box_rates: Dict[str, Dict[int, float]] = {}
        liq_scores: Dict[str, Dict[int, float]] = {}

        for symbol, curve in curves.items():
            box_rates[symbol] = {}
            liq_scores[symbol] = {}
            for pt in curve.points:
                if pt.is_valid():
                    box_rates[symbol][pt.days_to_expiry] = pt.mid_rate
                    liq_scores[symbol][pt.days_to_expiry] = pt.liquidity_score

        return self.compare(
            box_spread_rates=box_rates,
            treasury_rates=treasury_rates,
            liquidity_scores=liq_scores,
            fetch_live=fetch_live,
        )

    def _compute_spreads(self, comparison: YieldCurveComparison) -> None:
        """Compute spread at each box spread point vs closest benchmark."""
        for symbol, box_pts in comparison.box_curves.items():
            for bp in box_pts:
                bench = comparison.get_benchmark_at_dte(bp.days_to_expiry, tolerance=20)
                if bench is None:
                    continue

                spread_bps = (bp.rate_pct - bench.rate_pct) * 100.0
                comparison.spreads.append(SpreadPoint(
                    days_to_expiry=bp.days_to_expiry,
                    tenor_label=bp.tenor_label,
                    box_rate_pct=bp.rate_pct,
                    benchmark_rate_pct=bench.rate_pct,
                    benchmark_source=bench.source,
                    spread_bps=spread_bps,
                    box_symbol=symbol,
                    box_liquidity=bp.liquidity_score,
                ))

        comparison.spreads.sort(key=lambda s: (s.box_symbol, s.days_to_expiry))

    def format_text(self, comparison: YieldCurveComparison) -> str:
        """Format comparison as a human-readable text report."""
        lines: List[str] = []
        lines.append("=" * 100)
        lines.append("  YIELD CURVE COMPARISON: Box Spread vs Treasury/SOFR Benchmarks")
        lines.append("=" * 100)

        # Treasury curve
        if comparison.treasury_curve:
            lines.append("")
            lines.append("  TREASURY YIELD CURVE")
            lines.append("  " + "-" * 50)
            for pt in comparison.treasury_curve:
                lines.append(f"    {pt.tenor_label:>6}  ({pt.days_to_expiry:>5}d)  {pt.rate_pct:>6.2f}%")

        # SOFR curve
        if comparison.sofr_curve:
            lines.append("")
            lines.append("  SOFR CURVE")
            lines.append("  " + "-" * 50)
            for pt in comparison.sofr_curve:
                lines.append(f"    {pt.tenor_label:>6}  ({pt.days_to_expiry:>5}d)  {pt.rate_pct:>6.2f}%")

        # Box spread curves
        for symbol in comparison.symbols:
            pts = comparison.box_curves[symbol]
            lines.append("")
            lines.append(f"  BOX SPREAD: {symbol}")
            lines.append("  " + "-" * 50)
            for pt in pts:
                liq = f"  liq={pt.liquidity_score:.0f}" if pt.liquidity_score > 0 else ""
                lines.append(f"    {pt.tenor_label:>6}  ({pt.days_to_expiry:>5}d)  {pt.rate_pct:>6.2f}%{liq}")

        # Spread analysis
        if comparison.spreads:
            lines.append("")
            lines.append("  SPREAD ANALYSIS (Box Spread vs Benchmark)")
            lines.append("  " + "-" * 90)
            lines.append(
                f"  {'Symbol':>6}  {'Tenor':>6}  {'DTE':>5}  "
                f"{'BS Rate':>8}  {'Bench':>8}  {'Spread':>8}  {'Winner':>12}"
            )
            lines.append("  " + "-" * 90)
            for s in comparison.spreads:
                winner = "box_spread" if s.box_wins else ("benchmark" if s.benchmark_wins else "tie")
                lines.append(
                    f"  {s.box_symbol:>6}  {s.tenor_label:>6}  {s.days_to_expiry:>5}  "
                    f"{s.box_rate_pct:>7.2f}%  {s.benchmark_rate_pct:>7.2f}%  "
                    f"{s.spread_bps:>+7.0f}bp  {winner:>12}"
                )

        # Summary
        s = comparison.summary()
        lines.append("")
        lines.append("  " + "-" * 90)
        lines.append(
            f"  Box spread wins: {s['box_spread_wins']}  |  "
            f"Benchmark wins: {s['benchmark_wins']}  |  "
            f"Ties: {s['ties']}"
        )
        lines.append("=" * 100)

        return "\n".join(lines)
