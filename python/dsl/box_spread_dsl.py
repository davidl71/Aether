"""
Box Spread DSL - Builder for box spread scenarios

Provides fluent interface for defining box spread synthetic financing scenarios.
"""

from typing import Optional, List
from decimal import Decimal
import logging
from .types import (
    Rate, StrikeWidth, Expiration, Money,
    Direction, Benchmark, LiquidityConstraints
)

logger = logging.getLogger(__name__)

try:
    from python.integration.box_spread_strategy import BoxSpreadStrategy as _PyStrategy
except ImportError:
    _PyStrategy = None

try:
    from python.bindings.box_spread_bindings import (
        PyBoxSpreadStrategy as _CppStrategy,
    )
except ImportError:
    _CppStrategy = None


class BoxSpreadResult:
    """Result from evaluating a box spread scenario"""

    def __init__(self, scenario: 'BoxSpread', opportunities: List = None, errors: List = None):
        self.scenario = scenario
        self.opportunities = opportunities or []
        self.errors = errors or []

    def is_valid(self) -> bool:
        """Check if scenario evaluation was successful"""
        return len(self.errors) == 0 and len(self.opportunities) > 0


class BoxSpread:
    """Builder for box spread scenarios"""

    def __init__(self, symbol: str):
        if not symbol:
            raise ValueError("Symbol is required")
        self.symbol = symbol
        self.strike_width: Optional[StrikeWidth] = None
        self.expiration: Optional[Expiration] = None
        self.direction: Optional[Direction] = None
        self.min_implied_rate: Optional[Rate] = None
        self.benchmark: Optional[Benchmark] = None
        self.min_advantage_bps: Optional[int] = None
        self.liquidity: Optional[LiquidityConstraints] = None

    def strike_width(self, width: float, currency: str = "USD") -> 'BoxSpread':
        """Set strike width

        Args:
            width: Strike width (difference between high and low strikes)
            currency: Currency code (default: USD)
        """
        self.strike_width = StrikeWidth(Decimal(str(width)), currency)
        return self

    def expiration(self, date: str) -> 'BoxSpread':
        """Set expiration date

        Args:
            date: Expiration date in ISO format (YYYY-MM-DD)
        """
        self.expiration = Expiration(date)
        return self

    def direction(self, direction: Direction) -> 'BoxSpread':
        """Set direction: Lending or Borrowing

        Args:
            direction: Direction.LENDING or Direction.BORROWING
        """
        self.direction = direction
        return self

    def min_implied_rate(self, rate: float, unit: str = "percent") -> 'BoxSpread':
        """Set minimum implied interest rate

        Args:
            rate: Minimum rate value
            unit: "percent" or "bps" (default: "percent")
        """
        self.min_implied_rate = Rate(Decimal(str(rate)), unit)
        return self

    def benchmark(self, benchmark: Benchmark) -> 'BoxSpread':
        """Set benchmark rate source

        Args:
            benchmark: Benchmark enum (SOFR, TREASURY, MARGIN_LOAN, CUSTOM)
        """
        self.benchmark = benchmark
        return self

    def min_advantage_bps(self, bps: int) -> 'BoxSpread':
        """Set minimum rate advantage over benchmark in basis points

        Args:
            bps: Minimum advantage in basis points (1 bps = 0.01%)
        """
        if bps < 0:
            raise ValueError("Minimum advantage must be non-negative")
        self.min_advantage_bps = bps
        return self

    def liquidity(
        self,
        min_volume: int = 100,
        min_open_interest: int = 500,
        max_spread: float = 0.1,
        min_fill_probability: float = 0.5
    ) -> 'BoxSpread':
        """Set liquidity constraints

        Args:
            min_volume: Minimum daily volume
            min_open_interest: Minimum open interest
            max_spread: Maximum bid-ask spread in dollars
            min_fill_probability: Minimum fill probability (0.0 to 1.0)
        """
        self.liquidity = LiquidityConstraints(
            min_volume=min_volume,
            min_open_interest=min_open_interest,
            max_bid_ask_spread=Decimal(str(max_spread)),
            min_fill_probability=Decimal(str(min_fill_probability))
        )
        return self

    def validate(self) -> List[str]:
        """Validate scenario constraints

        Returns:
            List of validation error messages (empty if valid)
        """
        errors = []

        if not self.symbol:
            errors.append("Symbol is required")

        if not self.strike_width:
            errors.append("Strike width is required")

        if not self.expiration:
            errors.append("Expiration is required")

        if self.min_implied_rate:
            rate_pct = self.min_implied_rate.to_percent()
            if rate_pct < 0 or rate_pct > 20:
                errors.append("Implied rate must be between 0% and 20%")

        return errors

    def evaluate(self) -> BoxSpreadResult:
        """Evaluate scenario against live/cached market data.

        Attempts C++ bindings first for performance, then falls back to
        the pure-Python BoxSpreadStrategy in ``python/integration/``.

        Returns:
            BoxSpreadResult with opportunities and errors
        """
        errors = self.validate()
        if errors:
            return BoxSpreadResult(scenario=self, errors=errors)

        min_rate = float(self.min_implied_rate.to_percent()) if self.min_implied_rate else 0.0
        strike_w = float(self.strike_width.value) if self.strike_width else 0.0
        expiry = self.expiration.date if self.expiration else None

        # Attempt 1: C++ native bindings
        if _CppStrategy is not None:
            try:
                cpp = _CppStrategy(min_roi=min_rate, max_position_size=10)
                raw = cpp.find_box_spreads(self.symbol)
                opportunities = self._filter(raw, strike_w, expiry, min_rate)
                if opportunities:
                    logger.debug("C++ bindings returned %d opportunities", len(opportunities))
                    return BoxSpreadResult(scenario=self, opportunities=opportunities)
            except Exception as exc:
                logger.warning("C++ evaluation failed, falling back to Python: %s", exc)

        # Attempt 2: pure-Python strategy
        if _PyStrategy is not None:
            try:
                py_strat = _PyStrategy(
                    min_roi=min_rate,
                    min_days_to_expiry=7,
                    max_days_to_expiry=365,
                )
                raw = py_strat.find_opportunities(self.symbol)
                opportunities = self._filter(raw, strike_w, expiry, min_rate)
                if opportunities:
                    logger.debug("Python strategy returned %d opportunities", len(opportunities))
                    return BoxSpreadResult(scenario=self, opportunities=opportunities)
            except Exception as exc:
                logger.warning("Python strategy evaluation failed: %s", exc)

        return BoxSpreadResult(scenario=self, opportunities=[], errors=[])

    def _filter(self, raw: list, strike_w: float, expiry: Optional[str], min_rate: float) -> list:
        """Apply DSL constraints (strike width, expiry, rate, liquidity)."""
        filtered = []
        for opp in raw:
            sw = opp.get("strike_width", opp.get("high_strike", 0) - opp.get("low_strike", 0))
            if strike_w > 0 and abs(sw - strike_w) > 0.01:
                continue
            if expiry and opp.get("expiry", "") != expiry.replace("-", ""):
                continue
            if opp.get("roi_percent", 0) < min_rate:
                continue
            if self.liquidity:
                vol = opp.get("volume", 0)
                oi = opp.get("open_interest", 0)
                if vol < self.liquidity.min_volume or oi < self.liquidity.min_open_interest:
                    continue
            filtered.append(opp)
        return filtered

    def to_cpp(self) -> str:
        """Generate C++ code for this scenario

        Returns:
            C++ code string
        """
        cpp_symbol = self.symbol.replace("-", "_").upper()
        cpp_date = self.expiration.date.replace("-", "_")

        code = f"""// Generated from DSL: BoxSpread("{self.symbol}").strike_width({self.strike_width.value}).expiration("{self.expiration.date}")

namespace generated {{
namespace scenarios {{

struct {cpp_symbol}_BoxSpread_{cpp_date} {{
    // Constants
    static constexpr const char* symbol = "{self.symbol}";
    static constexpr double strike_width = {float(self.strike_width.value)};
    static constexpr const char* expiration = "{self.expiration.date}";
"""

        if self.min_implied_rate:
            code += f"""    static constexpr double min_implied_rate = {float(self.min_implied_rate.to_percent())};
"""

        if self.benchmark:
            code += f"""    static constexpr const char* benchmark = "{self.benchmark.value}";
"""

        code += """
    // Evaluation function
    static std::optional<types::BoxSpreadLeg> evaluate(
        const option_chain::OptionChain& chain,
        double underlying_price
    ) {
        // TODO: Generated evaluation logic
        // Finds box spreads matching criteria
        // Returns best opportunity
        return std::nullopt;
    }

    // Validation function
    static bool validate(const types::BoxSpreadLeg& leg) {
        return leg.get_strike_width() == strike_width &&
               leg.expiration == expiration;
    }
};

} // namespace scenarios
} // namespace generated
"""
        return code

    def __str__(self) -> str:
        """String representation of scenario"""
        parts = [f"BoxSpread({self.symbol})"]
        if self.strike_width:
            parts.append(f"strike_width={self.strike_width.value}")
        if self.expiration:
            parts.append(f"expiration={self.expiration.date}")
        if self.direction:
            parts.append(f"direction={self.direction.value}")
        if self.min_implied_rate:
            parts.append(f"min_rate={self.min_implied_rate}")
        if self.benchmark:
            parts.append(f"benchmark={self.benchmark.value}")
        return " ".join(parts)
