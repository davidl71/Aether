"""
financing_comparator.py - Box spread vs Treasury side-by-side decision matrix

Provides a structured comparison of box spread synthetic financing against
U.S. Treasury securities, incorporating pre-tax rates, after-tax rates
(Section 1256 vs ordinary income), margin/capital efficiency, and liquidity.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Dict, List, Optional

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


class FinancingDirection(Enum):
    LENDING = "lending"
    BORROWING = "borrowing"


class TaxBracket(Enum):
    BRACKET_10 = 0.10
    BRACKET_12 = 0.12
    BRACKET_22 = 0.22
    BRACKET_24 = 0.24
    BRACKET_32 = 0.32
    BRACKET_35 = 0.35
    BRACKET_37 = 0.37


@dataclass
class TaxConfig:
    """Tax parameters for after-tax return comparison."""
    federal_rate: float = 0.37
    state_rate: float = 0.05
    ltcg_rate: float = 0.20
    stcg_rate: float = 0.37
    state_exempt_treasuries: bool = True

    @property
    def section_1256_blended_rate(self) -> float:
        """Section 1256 contracts: 60% LTCG + 40% STCG regardless of holding period."""
        return 0.60 * self.ltcg_rate + 0.40 * self.stcg_rate

    @property
    def ordinary_income_rate(self) -> float:
        return self.federal_rate + self.state_rate

    @property
    def treasury_tax_rate(self) -> float:
        """Treasuries are state-exempt but federally taxed as ordinary income."""
        if self.state_exempt_treasuries:
            return self.federal_rate
        return self.federal_rate + self.state_rate


@dataclass
class InstrumentMetrics:
    """Metrics for a single financing instrument at a specific tenor."""
    instrument_type: str
    tenor_days: int
    gross_rate_pct: float
    after_tax_rate_pct: float
    tax_rate_applied: float
    commission_cost: float = 0.0
    effective_rate_pct: float = 0.0
    margin_requirement: float = 0.0
    notional_exposure: float = 0.0
    capital_outlay: float = 0.0
    capital_efficiency_pct: float = 0.0
    liquidity_score: float = 0.0
    available: bool = True
    source: str = ""
    timestamp: Optional[datetime] = None

    @property
    def after_tax_return_on_capital(self) -> float:
        """After-tax return per dollar of capital actually tied up."""
        if self.capital_outlay > 0:
            notional = self.notional_exposure or self.capital_outlay
            return self.after_tax_rate_pct * (notional / self.capital_outlay)
        return self.after_tax_rate_pct

    @property
    def margin_leverage(self) -> float:
        """Ratio of notional exposure to capital outlay. >1 means leveraged."""
        if self.capital_outlay > 0 and self.notional_exposure > 0:
            return self.notional_exposure / self.capital_outlay
        return 1.0

    @property
    def freed_capital(self) -> float:
        """Capital available for other uses vs full notional deployment."""
        return max(0.0, self.notional_exposure - self.capital_outlay)


@dataclass
class ComparisonRow:
    """A single row in the decision matrix, comparing instruments at one tenor."""
    tenor_days: int
    box_spread: Optional[InstrumentMetrics] = None
    treasury: Optional[InstrumentMetrics] = None

    @property
    def spread_bps_pretax(self) -> Optional[float]:
        if self.box_spread and self.treasury and self.box_spread.available and self.treasury.available:
            return (self.box_spread.gross_rate_pct - self.treasury.gross_rate_pct) * 100.0
        return None

    @property
    def spread_bps_aftertax(self) -> Optional[float]:
        if self.box_spread and self.treasury and self.box_spread.available and self.treasury.available:
            return (self.box_spread.after_tax_rate_pct - self.treasury.after_tax_rate_pct) * 100.0
        return None

    @property
    def winner(self) -> Optional[str]:
        spread = self.spread_bps_aftertax
        if spread is None:
            return None
        if spread > 5.0:
            return "box_spread"
        elif spread < -5.0:
            return "treasury"
        return "tie"


@dataclass
class DecisionMatrix:
    """Complete side-by-side comparison across tenors."""
    rows: List[ComparisonRow] = field(default_factory=list)
    direction: FinancingDirection = FinancingDirection.LENDING
    tax_config: TaxConfig = field(default_factory=TaxConfig)
    generated_time: datetime = field(default_factory=datetime.now)
    principal: float = 100000.0

    def add_row(self, row: ComparisonRow) -> None:
        self.rows.append(row)
        self.rows.sort(key=lambda r: r.tenor_days)

    @property
    def box_spread_wins(self) -> int:
        return sum(1 for r in self.rows if r.winner == "box_spread")

    @property
    def treasury_wins(self) -> int:
        return sum(1 for r in self.rows if r.winner == "treasury")

    @property
    def summary(self) -> Dict:
        return {
            "direction": self.direction.value,
            "tenors_compared": len(self.rows),
            "box_spread_wins": self.box_spread_wins,
            "treasury_wins": self.treasury_wins,
            "ties": len(self.rows) - self.box_spread_wins - self.treasury_wins,
            "principal": self.principal,
            "tax_bracket_federal": self.tax_config.federal_rate,
            "section_1256_blended": self.tax_config.section_1256_blended_rate,
            "generated": self.generated_time.isoformat(),
        }

    def to_dict(self) -> Dict:
        result = self.summary
        result["rows"] = []
        for row in self.rows:
            row_dict: Dict = {"tenor_days": row.tenor_days}
            if row.box_spread and row.box_spread.available:
                row_dict["box_spread"] = {
                    "gross_rate": row.box_spread.gross_rate_pct,
                    "after_tax_rate": row.box_spread.after_tax_rate_pct,
                    "effective_rate": row.box_spread.effective_rate_pct,
                    "tax_rate": row.box_spread.tax_rate_applied,
                    "commission": row.box_spread.commission_cost,
                    "liquidity_score": row.box_spread.liquidity_score,
                    "margin_requirement": row.box_spread.margin_requirement,
                    "notional_exposure": row.box_spread.notional_exposure,
                    "capital_outlay": row.box_spread.capital_outlay,
                    "capital_efficiency": row.box_spread.capital_efficiency_pct,
                    "return_on_capital": row.box_spread.after_tax_return_on_capital,
                    "margin_leverage": row.box_spread.margin_leverage,
                    "freed_capital": row.box_spread.freed_capital,
                }
            if row.treasury and row.treasury.available:
                row_dict["treasury"] = {
                    "gross_rate": row.treasury.gross_rate_pct,
                    "after_tax_rate": row.treasury.after_tax_rate_pct,
                    "tax_rate": row.treasury.tax_rate_applied,
                    "source": row.treasury.source,
                    "capital_outlay": row.treasury.capital_outlay,
                    "capital_efficiency": row.treasury.capital_efficiency_pct,
                    "return_on_capital": row.treasury.after_tax_return_on_capital,
                }
            row_dict["spread_bps_pretax"] = row.spread_bps_pretax
            row_dict["spread_bps_aftertax"] = row.spread_bps_aftertax
            row_dict["winner"] = row.winner
            result["rows"].append(row_dict)
        return result


class FinancingComparator:
    """
    Compare box spread synthetic financing against U.S. Treasuries.

    Builds a decision matrix across multiple tenors showing pre-tax and after-tax
    returns, accounting for Section 1256 tax treatment on qualified index options.
    """

    def __init__(
        self,
        tax_config: Optional[TaxConfig] = None,
        per_contract_fee: float = 0.65,
        treasury_client: Optional[TreasuryAPIClient] = None,
        sofr_client: Optional["SOFRTreasuryClient"] = None,
    ):
        self.tax_config = tax_config or TaxConfig()
        self.per_contract_fee = per_contract_fee
        self._treasury_client = treasury_client
        self._sofr_client = sofr_client

    @property
    def treasury_client(self) -> Optional[TreasuryAPIClient]:
        if self._treasury_client is None and TREASURY_API_AVAILABLE:
            self._treasury_client = TreasuryAPIClient()
        return self._treasury_client

    @property
    def sofr_client(self) -> Optional["SOFRTreasuryClient"]:
        if self._sofr_client is None and SOFR_AVAILABLE:
            self._sofr_client = SOFRTreasuryClient()
        return self._sofr_client

    def calculate_box_spread_after_tax_rate(
        self, gross_rate_pct: float, is_qualified_index: bool = True
    ) -> float:
        """
        Calculate after-tax rate for box spread.

        Qualified index options (SPX, XSP, etc.) get Section 1256 treatment:
        60% long-term capital gains + 40% short-term capital gains.
        """
        if is_qualified_index:
            tax_rate = self.tax_config.section_1256_blended_rate
        else:
            tax_rate = self.tax_config.ordinary_income_rate
        return gross_rate_pct * (1.0 - tax_rate)

    def calculate_treasury_after_tax_rate(self, gross_rate_pct: float) -> float:
        """Calculate after-tax rate for Treasury securities (state-exempt)."""
        return gross_rate_pct * (1.0 - self.tax_config.treasury_tax_rate)

    def build_box_spread_metrics(
        self,
        tenor_days: int,
        gross_rate_pct: float,
        strike_width: float = 50.0,
        liquidity_score: float = 75.0,
        is_qualified_index: bool = True,
        margin_type: str = "reg_t",
    ) -> InstrumentMetrics:
        """Build metrics for a box spread at a specific tenor."""
        commission = 4.0 * self.per_contract_fee
        notional = strike_width * 100.0
        commission_drag_pct = (commission / notional) * (365.0 / tenor_days) * 100.0

        effective_rate = gross_rate_pct - commission_drag_pct
        tax_rate = (
            self.tax_config.section_1256_blended_rate
            if is_qualified_index
            else self.tax_config.ordinary_income_rate
        )
        after_tax = effective_rate * (1.0 - tax_rate)

        # Box spread margin: max loss = notional under Reg-T,
        # but portfolio margin typically ~20-30% of notional
        if margin_type == "portfolio":
            margin = notional * 0.25
        else:
            margin = notional

        capital_eff = (notional / margin * 100.0) if margin > 0 else 100.0

        return InstrumentMetrics(
            instrument_type="box_spread",
            tenor_days=tenor_days,
            gross_rate_pct=gross_rate_pct,
            after_tax_rate_pct=after_tax,
            tax_rate_applied=tax_rate,
            commission_cost=commission,
            effective_rate_pct=effective_rate,
            margin_requirement=margin,
            notional_exposure=notional,
            capital_outlay=margin,
            capital_efficiency_pct=capital_eff,
            liquidity_score=liquidity_score,
            available=True,
            source="box_spread",
            timestamp=datetime.now(),
        )

    def build_treasury_metrics(
        self, tenor_days: int, gross_rate_pct: float, source: str = "Treasury API",
        principal: float = 0.0,
    ) -> InstrumentMetrics:
        """Build metrics for a Treasury security at a specific tenor."""
        tax_rate = self.tax_config.treasury_tax_rate
        after_tax = gross_rate_pct * (1.0 - tax_rate)

        # Treasuries require full capital outlay but can serve as margin collateral
        # (~95% of T-bill face value accepted as margin at IBKR)
        notional = principal if principal > 0 else 0.0

        return InstrumentMetrics(
            instrument_type="treasury",
            tenor_days=tenor_days,
            gross_rate_pct=gross_rate_pct,
            after_tax_rate_pct=after_tax,
            tax_rate_applied=tax_rate,
            effective_rate_pct=gross_rate_pct,
            margin_requirement=notional,
            notional_exposure=notional,
            capital_outlay=notional,
            capital_efficiency_pct=100.0,
            liquidity_score=100.0,
            available=True,
            source=source,
            timestamp=datetime.now(),
        )

    def fetch_treasury_rate(self, tenor_days: int) -> Optional[InstrumentMetrics]:
        """Fetch live Treasury rate for a given tenor from API."""
        client = self.treasury_client
        if client is None:
            return None

        try:
            rate = client.get_rate_for_days(tenor_days)
            if rate and rate.avg_interest_rate > 0:
                return self.build_treasury_metrics(
                    tenor_days=tenor_days,
                    gross_rate_pct=rate.avg_interest_rate,
                    source=f"Treasury {rate.security_term} ({rate.record_date})",
                )
        except Exception as e:
            logger.warning(f"Failed to fetch Treasury rate for {tenor_days}d: {e}")

        return None

    def compare_single_tenor(
        self,
        tenor_days: int,
        box_spread_rate_pct: float,
        treasury_rate_pct: Optional[float] = None,
        strike_width: float = 50.0,
        liquidity_score: float = 75.0,
        is_qualified_index: bool = True,
    ) -> ComparisonRow:
        """Compare box spread vs Treasury at a single tenor point."""
        box_metrics = self.build_box_spread_metrics(
            tenor_days=tenor_days,
            gross_rate_pct=box_spread_rate_pct,
            strike_width=strike_width,
            liquidity_score=liquidity_score,
            is_qualified_index=is_qualified_index,
        )

        treasury_metrics: Optional[InstrumentMetrics] = None
        if treasury_rate_pct is not None:
            treasury_metrics = self.build_treasury_metrics(
                tenor_days=tenor_days,
                gross_rate_pct=treasury_rate_pct,
                source="manual",
            )
        else:
            treasury_metrics = self.fetch_treasury_rate(tenor_days)

        return ComparisonRow(
            tenor_days=tenor_days,
            box_spread=box_metrics,
            treasury=treasury_metrics,
        )

    def build_decision_matrix(
        self,
        box_spread_rates: Dict[int, float],
        treasury_rates: Optional[Dict[int, float]] = None,
        principal: float = 100000.0,
        direction: FinancingDirection = FinancingDirection.LENDING,
        strike_width: float = 50.0,
        is_qualified_index: bool = True,
    ) -> DecisionMatrix:
        """
        Build a complete decision matrix across multiple tenors.

        Args:
            box_spread_rates: Dict mapping tenor_days -> gross rate (%) from box spreads
            treasury_rates: Optional dict mapping tenor_days -> gross rate (%).
                            If None, fetches from Treasury API.
            principal: Notional principal for absolute return calculation
            direction: Whether comparing for lending or borrowing
            strike_width: Box spread strike width (for commission drag calc)
            is_qualified_index: Whether the underlying qualifies for Section 1256

        Returns:
            DecisionMatrix with side-by-side comparison
        """
        matrix = DecisionMatrix(
            direction=direction,
            tax_config=self.tax_config,
            generated_time=datetime.now(),
            principal=principal,
        )

        all_tenors = sorted(box_spread_rates.keys())

        for tenor in all_tenors:
            bs_rate = box_spread_rates[tenor]
            tr_rate = treasury_rates.get(tenor) if treasury_rates else None

            row = self.compare_single_tenor(
                tenor_days=tenor,
                box_spread_rate_pct=bs_rate,
                treasury_rate_pct=tr_rate,
                strike_width=strike_width,
                is_qualified_index=is_qualified_index,
            )
            matrix.add_row(row)

        return matrix

    def compare_from_yield_curve(
        self,
        curve_rates: List[Dict],
        principal: float = 100000.0,
        direction: FinancingDirection = FinancingDirection.LENDING,
        is_qualified_index: bool = True,
    ) -> DecisionMatrix:
        """
        Build decision matrix from a yield curve (list of rate points).

        Each dict in curve_rates should have:
            - days_to_expiry: int
            - mid_rate or implied_rate: float (%)
            - strike_width: float (optional, default 50.0)
            - liquidity_score: float (optional, default 75.0)
        """
        box_rates: Dict[int, float] = {}
        for point in curve_rates:
            dte = point.get("days_to_expiry", 0)
            rate = point.get("mid_rate") or point.get("implied_rate", 0.0)
            if dte > 0 and rate != 0.0:
                box_rates[dte] = rate

        return self.build_decision_matrix(
            box_spread_rates=box_rates,
            principal=principal,
            direction=direction,
            is_qualified_index=is_qualified_index,
        )

    def format_text_report(self, matrix: DecisionMatrix) -> str:
        """Format the decision matrix as a human-readable text table."""
        lines: List[str] = []
        lines.append("=" * 115)
        lines.append(f"  FINANCING COMPARISON: Box Spread vs U.S. Treasury  ({matrix.direction.value.upper()})")
        lines.append(f"  Principal: ${matrix.principal:,.0f}  |  "
                     f"Federal Tax: {matrix.tax_config.federal_rate*100:.0f}%  |  "
                     f"Sec. 1256 Blended: {matrix.tax_config.section_1256_blended_rate*100:.1f}%")
        lines.append("=" * 115)
        lines.append(
            f"{'Tenor':>8}  "
            f"{'BS Gross':>9}  {'BS After':>9}  {'BS RoC':>8}  "
            f"{'Tsy Gross':>9}  {'Tsy After':>9}  "
            f"{'Spread':>8}  {'Leverage':>8}  {'Winner':>12}"
        )
        lines.append("-" * 115)

        for row in matrix.rows:
            tenor_str = f"{row.tenor_days}d"
            bs_gross = f"{row.box_spread.gross_rate_pct:.2f}%" if row.box_spread and row.box_spread.available else "N/A"
            bs_after = f"{row.box_spread.after_tax_rate_pct:.2f}%" if row.box_spread and row.box_spread.available else "N/A"
            bs_roc = f"{row.box_spread.after_tax_return_on_capital:.2f}%" if row.box_spread and row.box_spread.available else "N/A"
            tr_gross = f"{row.treasury.gross_rate_pct:.2f}%" if row.treasury and row.treasury.available else "N/A"
            tr_after = f"{row.treasury.after_tax_rate_pct:.2f}%" if row.treasury and row.treasury.available else "N/A"
            spread = f"{row.spread_bps_aftertax:+.0f}bp" if row.spread_bps_aftertax is not None else "N/A"
            leverage = f"{row.box_spread.margin_leverage:.1f}x" if row.box_spread and row.box_spread.available else "1.0x"
            winner = row.winner or "N/A"

            lines.append(
                f"{tenor_str:>8}  {bs_gross:>9}  {bs_after:>9}  {bs_roc:>8}  "
                f"{tr_gross:>9}  {tr_after:>9}  {spread:>8}  {leverage:>8}  {winner:>12}"
            )

        lines.append("-" * 115)
        s = matrix.summary
        lines.append(
            f"  Box spread wins: {s['box_spread_wins']}  |  "
            f"Treasury wins: {s['treasury_wins']}  |  "
            f"Ties: {s['ties']}"
        )
        lines.append("")
        lines.append("  RoC = Return on Capital (after-tax rate adjusted for margin leverage)")
        lines.append("  Leverage = Notional / Capital outlay (>1x = capital freed for other uses)")
        lines.append("=" * 115)
        return "\n".join(lines)
