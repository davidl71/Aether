"""
risk_calculator.py - Portfolio risk calculation engine (Python port)

Ported from native/src/risk_calculator.cpp (928 LOC).

Provides position-level and portfolio-level risk metrics including Greeks,
VaR (historical and parametric), drawdown analysis, risk-adjusted return
ratios (Sharpe, Sortino, Calmar, Information), correlation risk, and
position sizing (Kelly, fixed-fractional).

Statistical helpers (calculate_mean, calculate_percentile, calculate_correlation)
use C++ via pybind11 when box_spread_bindings is available; otherwise fall back
to Python implementations.
"""

from __future__ import annotations

import logging
import math
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import List, Optional

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Statistical helpers: prefer C++ (box_spread_bindings), fallback to Python
# ---------------------------------------------------------------------------
try:
    from ..bindings.box_spread_bindings import (  # type: ignore[attr-defined]
        calculate_mean as _cxx_calculate_mean,
        calculate_percentile as _cxx_calculate_percentile,
        calculate_correlation as _cxx_calculate_correlation,
    )
    _USE_CXX_STATS = True
except ImportError:
    _USE_CXX_STATS = False


def _py_calculate_mean(values: List[float]) -> float:
    if not values:
        return 0.0
    return sum(values) / len(values)


def _py_calculate_percentile(values: List[float], percentile: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    idx = min(int(percentile * len(s)), len(s) - 1)
    return s[idx]


def _py_calculate_correlation(x: List[float], y: List[float]) -> float:
    if len(x) != len(y) or not x:
        return 0.0
    mx, my = _py_calculate_mean(x), _py_calculate_mean(y)
    sxy, sx2, sy2 = 0.0, 0.0, 0.0
    for xi, yi in zip(x, y, strict=False):
        dx, dy = xi - mx, yi - my
        sxy += dx * dy
        sx2 += dx * dx
        sy2 += dy * dy
    denom = math.sqrt(sx2 * sy2)
    return sxy / denom if denom != 0 else 0.0


def calculate_mean(values: List[float]) -> float:
    if _USE_CXX_STATS:
        return _cxx_calculate_mean(values)
    return _py_calculate_mean(values)


def calculate_standard_deviation(values: List[float]) -> float:
    if not values:
        return 0.0
    mean = calculate_mean(values)
    ss = sum((v - mean) ** 2 for v in values)
    return math.sqrt(ss / len(values))


def calculate_percentile(values: List[float], percentile: float) -> float:
    if _USE_CXX_STATS:
        return _cxx_calculate_percentile(values, percentile)
    return _py_calculate_percentile(values, percentile)


def calculate_correlation(x: List[float], y: List[float]) -> float:
    if _USE_CXX_STATS:
        return _cxx_calculate_correlation(x, y)
    return _py_calculate_correlation(x, y)


def calculate_beta(
    asset_returns: List[float], market_returns: List[float]
) -> float:
    corr = calculate_correlation(asset_returns, market_returns)
    a_std = calculate_standard_deviation(asset_returns)
    m_std = calculate_standard_deviation(market_returns)
    return corr * (a_std / m_std) if m_std != 0 else 0.0


def annualize_return(period_return: float, periods_per_year: int) -> float:
    return period_return * periods_per_year


def annualize_volatility(period_volatility: float, periods_per_year: int) -> float:
    return period_volatility * math.sqrt(periods_per_year)


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class RiskConfig:
    max_total_exposure: float = 100_000.0
    max_positions: int = 20
    position_size_percent: float = 0.10
    max_single_position: float = 25_000.0
    max_sector_exposure: float = 0.30
    var_confidence: float = 0.95
    max_drawdown: float = 0.20


@dataclass
class RiskMetrics:
    delta: float = 0.0
    gamma: float = 0.0
    theta: float = 0.0
    vega: float = 0.0
    rho: float = 0.0
    max_loss: float = 0.0
    max_gain: float = 0.0
    probability_of_profit: float = 0.0


@dataclass
class PositionRisk:
    position_size: float = 0.0
    max_loss: float = 0.0
    max_gain: float = 0.0
    expected_value: float = 0.0
    delta: float = 0.0
    gamma: float = 0.0
    theta: float = 0.0
    vega: float = 0.0
    leverage: float = 1.0
    probability_of_profit: float = 0.0
    risk_reward_ratio: float = 0.0
    initial_margin: float = 0.0
    maintenance_margin: float = 0.0
    margin_utilization: float = 0.0
    margin_call_risk: bool = False
    margin_timestamp: Optional[datetime] = None


@dataclass
class PortfolioRisk:
    total_exposure: float = 0.0
    total_delta: float = 0.0
    total_gamma: float = 0.0
    total_theta: float = 0.0
    total_vega: float = 0.0
    var_95: float = 0.0
    var_99: float = 0.0
    concentration_risk: float = 0.0
    liquidity_risk: float = 0.0
    correlation_risk: float = 0.0


class RiskAlertLevel(Enum):
    Info = "info"
    Warning = "warning"
    Critical = "critical"


@dataclass
class RiskAlert:
    level: RiskAlertLevel = RiskAlertLevel.Info
    category: str = ""
    message: str = ""
    timestamp: Optional[datetime] = None


@dataclass
class ScenarioResult:
    scenario_name: str = ""
    price_change_percent: float = 0.0
    position_pnl: float = 0.0


@dataclass
class SimplePosition:
    """Lightweight position representation for risk calculations."""

    symbol: str = ""
    quantity: int = 0
    current_price: float = 0.0
    avg_price: float = 0.0
    cost_basis: float = 0.0
    strike: float = 0.0
    option_type: str = ""  # "C", "P", or "" for stock
    is_option: bool = False

    def get_market_value(self) -> float:
        multiplier = 100.0 if self.is_option else 1.0
        return self.current_price * self.quantity * multiplier

    def get_cost_basis(self) -> float:
        if self.cost_basis > 0:
            return self.cost_basis
        multiplier = 100.0 if self.is_option else 1.0
        return abs(self.avg_price * self.quantity * multiplier)

    def is_long(self) -> bool:
        return self.quantity > 0


@dataclass
class SimpleBoxSpreadLeg:
    """Lightweight box spread leg for risk calculations."""

    net_debit: float = 0.0
    strike_low: float = 0.0
    strike_high: float = 0.0

    def get_strike_width(self) -> float:
        return self.strike_high - self.strike_low


@dataclass
class AccountInfo:
    net_liquidation: float = 0.0
    total_cash: float = 0.0
    buying_power: float = 0.0


def _total_abs_exposure(positions: List[SimplePosition]) -> float:
    return sum(abs(p.get_market_value()) for p in positions)


# ---------------------------------------------------------------------------
# RiskCalculator
# ---------------------------------------------------------------------------


class RiskCalculator:
    """Portfolio-level risk engine."""

    def __init__(self, config: Optional[RiskConfig] = None):
        self.config = config or RiskConfig()

    # -- Box spread risk (defined risk) --

    def calculate_box_spread_risk(
        self,
        spread: SimpleBoxSpreadLeg,
        underlying_price: float = 0.0,
        implied_volatility: float = 0.0,
    ) -> PositionRisk:
        risk = PositionRisk()
        risk.position_size = spread.net_debit * 100.0
        risk.max_loss = spread.net_debit * 100.0
        risk.max_gain = (spread.get_strike_width() - spread.net_debit) * 100.0
        risk.expected_value = risk.max_gain
        risk.delta = 0.0
        risk.gamma = 0.0
        risk.theta = 0.0
        risk.vega = 0.0
        risk.leverage = 1.0
        risk.probability_of_profit = 1.0
        if risk.max_loss > 0:
            risk.risk_reward_ratio = risk.max_gain / risk.max_loss
        risk.margin_timestamp = datetime.now()
        return risk

    # -- Position risk --

    def calculate_position_risk(
        self,
        position: SimplePosition,
        underlying_price: float = 0.0,
        implied_volatility: float = 0.0,
    ) -> PositionRisk:
        risk = PositionRisk()
        risk.position_size = abs(position.get_market_value())
        risk.max_loss = risk.position_size
        risk.max_gain = risk.position_size * 2.0
        return risk

    def calculate_max_loss(self, position: SimplePosition) -> float:
        if position.is_long():
            return position.get_cost_basis()
        return position.strike * 100.0 * abs(position.quantity)

    def calculate_max_gain(self, position: SimplePosition) -> float:
        if position.is_long():
            return position.strike * 100.0 * position.quantity
        return position.get_cost_basis()

    # -- Portfolio risk --

    def calculate_portfolio_risk(
        self,
        positions: List[SimplePosition],
        account: Optional[AccountInfo] = None,
    ) -> PortfolioRisk:
        pr = PortfolioRisk()
        for pos in positions:
            pr.total_exposure += abs(pos.get_market_value())
        pr.var_95 = pr.total_exposure * 0.05
        pr.var_99 = pr.total_exposure * 0.10
        return pr

    # -- Limits --

    def is_within_limits(
        self, position: SimplePosition, existing: List[SimplePosition]
    ) -> bool:
        total = _total_abs_exposure(existing) + abs(position.get_market_value())
        return total <= self.config.max_total_exposure

    def is_box_spread_within_limits(
        self, spread: SimpleBoxSpreadLeg, existing: List[SimplePosition]
    ) -> bool:
        cost = spread.net_debit * 100.0
        total = _total_abs_exposure(existing)
        return (
            (total + cost) <= self.config.max_total_exposure
            and len(existing) < self.config.max_positions
        )

    def calculate_remaining_capacity(
        self, positions: List[SimplePosition], account_value: float
    ) -> float:
        total = _total_abs_exposure(positions)
        max_allowed = min(
            self.config.max_total_exposure,
            account_value * self.config.position_size_percent,
        )
        return max_allowed - total

    def would_exceed_limits(
        self,
        new_position: SimplePosition,
        existing: List[SimplePosition],
        account_value: float = 0.0,
    ) -> bool:
        return not self.is_within_limits(new_position, existing)

    # -- Position sizing --

    def calculate_optimal_position_size(
        self,
        spread: SimpleBoxSpreadLeg,
        account_value: float,
        risk_tolerance: float = 0.10,
    ) -> int:
        cost = spread.net_debit * 100.0
        if cost <= 0:
            return 0
        max_exposure = account_value * risk_tolerance
        return max(1, int(max_exposure / cost))

    def calculate_kelly_position_size(
        self,
        win_probability: float,
        win_amount: float,
        loss_amount: float,
        account_value: float,
    ) -> int:
        """Kelly Criterion position sizing with half-Kelly and 25 % cap."""
        if loss_amount == 0:
            return 0
        b = win_amount / loss_amount
        p = win_probability
        q = 1.0 - p
        kelly = (b * p - q) / b
        kelly *= 0.5
        kelly = max(0.0, min(kelly, 0.25))
        return int(account_value * kelly / 100.0)

    def calculate_fixed_fractional_size(
        self,
        position_cost: float,
        account_value: float,
        risk_percent: float = 0.02,
    ) -> int:
        risk_amount = account_value * risk_percent
        if position_cost <= 0:
            return 0
        return max(1, int(risk_amount / position_cost))

    # -- VaR --

    def calculate_var_historical(
        self, returns: List[float], confidence_level: float = 0.95
    ) -> float:
        if not returns:
            return 0.0
        s = sorted(returns)
        idx = min(int((1.0 - confidence_level) * len(s)), len(s) - 1)
        return -s[idx]

    def calculate_var_parametric(
        self,
        expected_return: float,
        volatility: float,
        position_value: float,
        confidence_level: float = 0.95,
        time_horizon_days: int = 1,
    ) -> float:
        z_score = 2.326 if confidence_level >= 0.99 else 1.645
        time_factor = math.sqrt(time_horizon_days / 252.0)
        return position_value * z_score * volatility * time_factor

    def calculate_expected_shortfall(
        self, returns: List[float], confidence_level: float = 0.95
    ) -> float:
        """CVaR / Expected Shortfall."""
        if not returns:
            return 0.0
        s = sorted(returns)
        cutoff = max(1, int((1.0 - confidence_level) * len(s)))
        return -sum(s[:cutoff]) / cutoff

    # -- Scenario analysis --

    def run_scenario_analysis(
        self,
        position: SimplePosition,
        current_price: float,
        price_scenarios: List[float],
    ) -> List[ScenarioResult]:
        results = []
        for sp in price_scenarios:
            pct = ((sp - current_price) / current_price) * 100.0 if current_price else 0.0
            results.append(
                ScenarioResult(
                    scenario_name=f"Price: ${sp:.2f}",
                    price_change_percent=pct,
                    position_pnl=0.0,  # would need full pricer
                )
            )
        return results

    # -- Greeks --

    def calculate_box_spread_greeks(
        self,
        spread: SimpleBoxSpreadLeg,
        underlying_price: float = 0.0,
        volatility: float = 0.0,
    ) -> RiskMetrics:
        m = RiskMetrics()
        m.max_loss = spread.net_debit * 100.0
        m.max_gain = (spread.get_strike_width() - spread.net_debit) * 100.0
        m.probability_of_profit = 1.0
        return m

    # -- Risk-adjusted returns --

    def calculate_sharpe_ratio(
        self, returns: List[float], risk_free_rate: float = 0.0
    ) -> float:
        if not returns:
            return 0.0
        mean_r = calculate_mean(returns)
        std = calculate_standard_deviation(returns)
        if std == 0:
            return 0.0
        return (mean_r - risk_free_rate) / std

    def calculate_sortino_ratio(
        self, returns: List[float], risk_free_rate: float = 0.0
    ) -> float:
        if not returns:
            return 0.0
        mean_r = calculate_mean(returns)
        downside = [r for r in returns if r < 0]
        if not downside:
            return float("inf")
        ds_std = calculate_standard_deviation(downside)
        if ds_std == 0:
            return 0.0
        return (mean_r - risk_free_rate) / ds_std

    def calculate_calmar_ratio(
        self, annualized_return: float, max_drawdown: float
    ) -> float:
        if max_drawdown == 0:
            return 0.0
        return annualized_return / max_drawdown

    def calculate_information_ratio(
        self, returns: List[float], benchmark_returns: List[float]
    ) -> float:
        if len(returns) != len(benchmark_returns) or not returns:
            return 0.0
        excess = [r - b for r, b in zip(returns, benchmark_returns, strict=False)]
        mean_ex = calculate_mean(excess)
        te = calculate_standard_deviation(excess)
        if te == 0:
            return 0.0
        return mean_ex / te

    # -- Drawdown analysis --

    def calculate_max_drawdown(self, equity_curve: List[float]) -> float:
        if not equity_curve:
            return 0.0
        peak = equity_curve[0]
        max_dd = 0.0
        for v in equity_curve:
            if v > peak:
                peak = v
            dd = (peak - v) / peak if peak > 0 else 0.0
            max_dd = max(max_dd, dd)
        return max_dd

    def calculate_current_drawdown(self, equity_curve: List[float]) -> float:
        if not equity_curve:
            return 0.0
        peak = max(equity_curve)
        current = equity_curve[-1]
        return (peak - current) / peak if peak > 0 else 0.0

    # -- Correlation risk --

    def calculate_correlation_risk(
        self, positions: List[SimplePosition]
    ) -> float:
        """Simplified correlation risk based on same-symbol detection.

        Full implementation would use historical return correlation matrix
        (ported from the Eigen-based C++ version).
        """
        if len(positions) < 2:
            return 0.0

        n = len(positions)
        total_value = sum(abs(p.get_market_value()) for p in positions)
        if total_value == 0:
            return 0.0

        weights = [abs(p.get_market_value()) / total_value for p in positions]

        corr_matrix = [[0.0] * n for _ in range(n)]
        for i in range(n):
            corr_matrix[i][i] = 1.0
            for j in range(i + 1, n):
                if positions[i].symbol == positions[j].symbol:
                    c = 1.0
                else:
                    p1, p2 = positions[i], positions[j]
                    if (
                        p1.current_price > 0
                        and p2.current_price > 0
                        and p1.avg_price > 0
                        and p2.avg_price > 0
                    ):
                        r1 = (p1.current_price - p1.avg_price) / p1.avg_price
                        r2 = (p2.current_price - p2.avg_price) / p2.avg_price
                        c = 0.7 if (r1 > 0) == (r2 > 0) else 0.3
                    else:
                        c = 0.5
                corr_matrix[i][j] = c
                corr_matrix[j][i] = c

        # w^T * C * w
        temp = [sum(corr_matrix[i][j] * weights[j] for j in range(n)) for i in range(n)]
        variance = sum(weights[i] * temp[i] for i in range(n))
        return math.sqrt(variance)

    # -- Config --

    def update_config(self, config: RiskConfig) -> None:
        self.config = config

    def get_config(self) -> RiskConfig:
        return self.config


# ---------------------------------------------------------------------------
# RiskMonitor
# ---------------------------------------------------------------------------


class RiskMonitor:
    """Monitors positions and generates risk alerts."""

    def __init__(self, config: Optional[RiskConfig] = None):
        self.config = config or RiskConfig()

    def check_risks(
        self, positions: List[SimplePosition], account: Optional[AccountInfo] = None
    ) -> List[RiskAlert]:
        alerts: List[RiskAlert] = []
        total = _total_abs_exposure(positions)
        if total > self.config.max_total_exposure * 0.9:
            alerts.append(
                RiskAlert(
                    level=RiskAlertLevel.Warning,
                    category="EXPOSURE",
                    message="Approaching maximum exposure limit",
                    timestamp=datetime.now(),
                )
            )
        return alerts
