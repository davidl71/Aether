# Box Spread DSL Architecture Design

**Date**: 2025-11-19
**Purpose**: Detailed architecture design for box spread DSL implementation
**Status**: Design Phase

---

## Architecture Overview

### Three-Tier DSL Architecture

```
┌─────────────────────────────────────────────────┐
│  DSL Layer (Domain Expert Interface)            │
│  - Python Embedded DSL (Internal)                │
│  - External DSL Syntax (Future)                  │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│  Model Layer (Domain Model)                      │
│  - BoxSpreadScenario                             │
│  - FinancingStrategy                             │
│  - AssetRelationship                             │
│  - CashFlowModel                                 │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│  Code Generation Layer                           │
│  - C++ Code Generator                            │
│  - Test Generator                                │
│  - Documentation Generator                        │
│  - Graph Generator                                │
└──────────────────────────────────────────────────┘
```

---

## DSL Syntax Design

### 1. Box Spread Scenario DSL

#### Basic Syntax

```python

# Minimal box spread

scenario = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19")

# With financing constraints

scenario = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19") \
    .direction(Lending) \
    .min_implied_rate(4.5) \
    .benchmark(SOFR) \
    .min_advantage_bps(50) \
    .liquidity(min_volume=100, min_oi=500, max_spread=0.1)
```

#### Advanced Syntax

```python

# Multi-expiration scan

scenarios = BoxSpread("SPX") \
    .strike_width(50) \
    .scan_expirations(
        min_dte=30,
        max_dte=90,
        step_days=7
    ) \
    .filter(min_rate=4.0) \
    .rank_by(rate_advantage)

# Multi-symbol comparison

comparison = BoxSpread.multi_symbol(
    ["SPX", "XSP", "NDX"]
) \
    .strike_width(50) \
    .expiration("2025-12-19") \
    .compare_rates() \
    .best_rate()
```

### 2. Domain-Specific Types

```python

# python/dsl/types.py

from dataclasses import dataclass
from enum import Enum
from typing import Optional
from decimal import Decimal

class Direction(Enum):
    LENDING = "lending"
    BORROWING = "borrowing"

class Benchmark(Enum):
    SOFR = "SOFR"
    TREASURY = "TREASURY"
    MARGIN_LOAN = "MARGIN_LOAN"
    CUSTOM = "CUSTOM"

@dataclass
class Rate:
    """Domain-specific rate type with precision"""
    value: Decimal
    unit: str = "percent"  # "percent" or "bps"

    def to_bps(self) -> int:
        if self.unit == "percent":
            return int(self.value * 100)
        return int(self.value)

    def to_percent(self) -> Decimal:
        if self.unit == "bps":
            return self.value / 100
        return self.value

@dataclass
class StrikeWidth:
    """Strike width with currency"""
    value: Decimal
    currency: str = "USD"

    def __post_init__(self):
        if self.value <= 0:
            raise ValueError("Strike width must be positive")

@dataclass
class Expiration:
    """Expiration date with validation"""
    date: str  # ISO format: "YYYY-MM-DD"
    days_to_expiry: Optional[int] = None

    def __post_init__(self):
        from datetime import datetime
        try:
            dt = datetime.fromisoformat(self.date)
            if dt < datetime.now():
                raise ValueError("Expiration date must be in the future")
        except ValueError as e:
            raise ValueError(f"Invalid date format: {self.date}") from e

@dataclass
class Money:
    """Money type with currency"""
    amount: Decimal
    currency: str = "USD"

    def __post_init__(self):
        if self.amount < 0:
            raise ValueError("Money amount cannot be negative")

@dataclass
class LiquidityConstraints:
    """Liquidity filtering constraints"""
    min_volume: int = 100
    min_open_interest: int = 500
    max_bid_ask_spread: Decimal = Decimal("0.1")
    min_fill_probability: Decimal = Decimal("0.5")
```

### 3. Box Spread Builder Implementation

```python

# python/dsl/box_spread_dsl.py

from typing import Optional, List
from decimal import Decimal
from .types import (
    Rate, StrikeWidth, Expiration, Money,
    Direction, Benchmark, LiquidityConstraints
)

class BoxSpread:
    """Builder for box spread scenarios"""

    def __init__(self, symbol: str):
        self.symbol = symbol
        self.strike_width: Optional[StrikeWidth] = None
        self.expiration: Optional[Expiration] = None
        self.direction: Optional[Direction] = None
        self.min_implied_rate: Optional[Rate] = None
        self.benchmark: Optional[Benchmark] = None
        self.min_advantage_bps: Optional[int] = None
        self.liquidity: Optional[LiquidityConstraints] = None

    def strike_width(self, width: float, currency: str = "USD") -> 'BoxSpread':
        """Set strike width"""
        self.strike_width = StrikeWidth(Decimal(str(width)), currency)
        return self

    def expiration(self, date: str) -> 'BoxSpread':
        """Set expiration date (ISO format: YYYY-MM-DD)"""
        self.expiration = Expiration(date)
        return self

    def direction(self, direction: Direction) -> 'BoxSpread':
        """Set direction: Lending or Borrowing"""
        self.direction = direction
        return self

    def min_implied_rate(self, rate: float, unit: str = "percent") -> 'BoxSpread':
        """Set minimum implied interest rate"""
        self.min_implied_rate = Rate(Decimal(str(rate)), unit)
        return self

    def benchmark(self, benchmark: Benchmark) -> 'BoxSpread':
        """Set benchmark rate source"""
        self.benchmark = benchmark
        return self

    def min_advantage_bps(self, bps: int) -> 'BoxSpread':
        """Set minimum rate advantage over benchmark in basis points"""
        self.min_advantage_bps = bps
        return self

    def liquidity(
        self,
        min_volume: int = 100,
        min_open_interest: int = 500,
        max_spread: float = 0.1,
        min_fill_probability: float = 0.5
    ) -> 'BoxSpread':
        """Set liquidity constraints"""
        self.liquidity = LiquidityConstraints(
            min_volume=min_volume,
            min_open_interest=min_open_interest,
            max_bid_ask_spread=Decimal(str(max_spread)),
            min_fill_probability=Decimal(str(min_fill_probability))
        )
        return self

    def validate(self) -> None:
        """Validate scenario constraints"""
        if not self.symbol:
            raise ValueError("Symbol is required")
        if not self.strike_width:
            raise ValueError("Strike width is required")
        if not self.expiration:
            raise ValueError("Expiration is required")

    def evaluate(self) -> 'BoxSpreadResult':
        """Evaluate scenario and return result"""
        self.validate()
        # Implementation would call C++ bindings
        # For now, return result structure
        return BoxSpreadResult(
            scenario=self,
            opportunities=[],
            errors=[]
        )

    def to_cpp(self) -> str:
        """Generate C++ code for this scenario"""
        # Code generation logic
        return f"""
// Generated from DSL
namespace generated {{
    struct {self.symbol}_BoxSpread_Scenario {{
        static constexpr const char* symbol = "{self.symbol}";
        static constexpr double strike_width = {self.strike_width.value};
        // ... more generated code
    }};
}}
"""
```

### 4. Multi-Asset Financing Strategy DSL

```python

# python/dsl/financing_strategy_dsl.py

class FinancingStrategy:
    """Builder for multi-asset financing strategies"""

    def __init__(self, name: str):
        self.name = name
        self.sources: List[FinancingSource] = []
        self.uses: List[FinancingUse] = []
        self.optimizations: List[Optimization] = []

    def source(self, source: FinancingSource) -> 'FinancingStrategy':
        """Add financing source (loan, box spread, etc.)"""
        self.sources.append(source)
        return self

    def use_as(self, use: FinancingUse) -> 'FinancingStrategy':
        """Specify how source is used"""
        self.uses.append(use)
        return self

    def then(self, action: FinancingAction) -> 'FinancingStrategy':
        """Chain financing actions"""
        self.uses.append(action)
        return self

    def optimize(self, objective: Optimization) -> 'FinancingStrategy':
        """Set optimization objective"""
        self.optimizations.append(objective)
        return self

# Example usage

strategy = FinancingStrategy("loan_to_box_spread") \
    .source(bank_loan(rate=5.5, amount=100000)) \
    .use_as(margin_for(
        box_spread(symbol="SPX", min_rate=4.0)
    )) \
    .then(invest_in(fund="providence", rate=3.0)) \
    .optimize(minimize_total_cost())
```

### 5. Cash Flow Modeling DSL

```python

# python/dsl/cash_flow_dsl.py

class CashFlowModel:
    """Builder for cash flow models"""

    def __init__(self):
        self.positions: List[Position] = []
        self.projection_months: int = 12
        self.optimization: Optional[Optimization] = None

    def add_position(self, position: Position) -> 'CashFlowModel':
        """Add position to cash flow model"""
        self.positions.append(position)
        return self

    def project(self, months: int) -> 'CashFlowModel':
        """Set projection period"""
        self.projection_months = months
        return self

    def optimize(self, objective: Optimization) -> 'CashFlowModel':
        """Set optimization objective"""
        self.optimization = objective
        return self

    def simulate(self) -> CashFlowResult:
        """Run cash flow simulation"""
        # Implementation
        pass

# Example usage

cash_flow = CashFlowModel() \
    .add_position(
        box_spread_lending(
            amount=50000,
            rate=4.8,
            maturity="2025-12-19"
        )
    ) \
    .add_position(
        bank_loan(
            amount=100000,
            rate=5.5,
            payments=monthly
        )
    ) \
    .project(months=12) \
    .optimize(net_cash_flow)
```

---

## Code Generation Architecture

### Generator Interface

```python

# python/dsl/generator.py

from abc import ABC, abstractmethod
from typing import List

class CodeGenerator(ABC):
    """Base class for code generators"""

    @abstractmethod
    def generate(self, model: 'DSLModel') -> str:
        """Generate code from DSL model"""
        pass

class CppGenerator(CodeGenerator):
    """Generate C++ code from DSL model"""

    def generate(self, model: 'DSLModel') -> str:
        # Generate C++ structs, functions, etc.
        pass

class TestGenerator(CodeGenerator):
    """Generate test code from DSL model"""

    def generate(self, model: 'DSLModel') -> str:
        # Generate BDD scenarios, unit tests
        pass

class DocumentationGenerator(CodeGenerator):
    """Generate documentation from DSL model"""

    def generate(self, model: 'DSLModel') -> str:
        # Generate AsciiDoc documentation
        pass

class GraphGenerator(CodeGenerator):
    """Generate graph visualizations from DSL model"""

    def generate(self, model: 'DSLModel') -> str:
        # Generate Graphviz DOT files
        pass
```

### Generated C++ Code Structure

```cpp
// Generated from DSL: BoxSpread("SPX").strike_width(50).expiration("2025-12-19")

namespace generated {
namespace scenarios {

struct SPX_BoxSpread_50_2025_12_19 {
    // Constants
    static constexpr const char* symbol = "SPX";
    static constexpr double strike_width = 50.0;
    static constexpr const char* expiration = "2025-12-19";

    // Evaluation function
    static std::optional<types::BoxSpreadLeg> evaluate(
        const option_chain::OptionChain& chain,
        double underlying_price
    ) {
        // Generated evaluation logic
        // Finds box spreads matching criteria
        // Returns best opportunity
    }

    // Validation function
    static bool validate(const types::BoxSpreadLeg& leg) {
        // Generated validation logic
        return leg.get_strike_width() == strike_width &&
               leg.expiration == expiration &&
               // ... more validation
    }
};

} // namespace scenarios
} // namespace generated
```

---

## Integration Points

### 1. Python Bindings Integration

```python

# DSL uses existing C++ bindings

from box_spread_bindings import (
    BoxSpreadStrategy,
    BoxSpreadLeg,
    TWSClient,
    OrderManager
)

# DSL scenario evaluates to C++ types

scenario = BoxSpread("SPX").strike_width(50).expiration("2025-12-19")
leg = scenario.evaluate()  # Returns BoxSpreadLeg (C++ type)

# Use with existing strategy

strategy = BoxSpreadStrategy(client, order_mgr, params)
opportunity = strategy.evaluate_box_spread(leg)
```

### 2. Configuration Integration

```python

# DSL reads from existing config

from config_manager import load_config

config = load_config("config/config.json")
scenario = BoxSpread(config.strategy.symbols[0]) \
    .min_implied_rate(config.strategy.min_implied_rate_percent) \
    .benchmark(Benchmark(config.strategy.benchmark_rate_source))
```

### 3. Multi-Asset Relationship Integration

```python

# DSL integrates with asset relationship system

from asset_relationship_graph import AssetRelationshipGraph

relationships = AssetRelationshipGraph()
strategy = FinancingStrategy("optimize") \
    .source(bank_loan(rate=5.5)) \
    .use_as(margin_for(box_spread("SPX"))) \
    .validate_against(relationships)  # Validate using relationship graph
```

---

## Validation and Error Handling

### Validation Rules

```python
class ValidationError(Exception):
    """Base class for validation errors"""
    pass

class BoxSpreadValidator:
    """Validates box spread scenarios"""

    def validate(self, scenario: BoxSpread) -> List[ValidationError]:
        errors = []

        # Required fields
        if not scenario.symbol:
            errors.append(ValidationError("Symbol is required"))

        if not scenario.strike_width:
            errors.append(ValidationError("Strike width is required"))

        if not scenario.expiration:
            errors.append(ValidationError("Expiration is required"))

        # Constraint validation
        if scenario.min_implied_rate:
            if scenario.min_implied_rate.value < 0 or scenario.min_implied_rate.value > 20:
                errors.append(ValidationError("Implied rate must be between 0% and 20%"))

        # Liquidity validation
        if scenario.liquidity:
            if scenario.liquidity.min_volume < 0:
                errors.append(ValidationError("Minimum volume must be non-negative"))

        return errors
```

---

## Testing Strategy

### DSL Unit Tests

```python

# tests/dsl/test_box_spread_dsl.py

def test_basic_box_spread():
    scenario = BoxSpread("SPX") \
        .strike_width(50) \
        .expiration("2025-12-19")

    assert scenario.symbol == "SPX"
    assert scenario.strike_width.value == 50
    assert scenario.expiration.date == "2025-12-19"

def test_validation():
    scenario = BoxSpread("SPX")  # Missing required fields
    validator = BoxSpreadValidator()
    errors = validator.validate(scenario)
    assert len(errors) > 0
```

### Generated Code Tests

```python

# Tests for generated C++ code

def test_generated_cpp():
    scenario = BoxSpread("SPX").strike_width(50).expiration("2025-12-19")
    cpp_code = scenario.to_cpp()

    assert "SPX_BoxSpread" in cpp_code
    assert "strike_width = 50.0" in cpp_code
    assert "expiration = \"2025-12-19\"" in cpp_code
```

---

## Implementation Plan

### Phase 1: Core DSL (Week 1-2)

- [ ] Domain-specific types (Rate, StrikeWidth, Expiration, Money)
- [ ] BoxSpread builder class
- [ ] Basic validation
- [ ] Unit tests

### Phase 2: Code Generation (Week 3-4)

- [ ] C++ code generator
- [ ] Test generator
- [ ] Documentation generator
- [ ] Integration tests

### Phase 3: Advanced Features (Week 5-6)

- [ ] Multi-asset financing strategy DSL
- [ ] Cash flow modeling DSL
- [ ] Relationship validation
- [ ] Graph visualization

### Phase 4: Production (Week 7-8)

- [ ] Performance optimization
- [ ] Error handling improvements
- [ ] Documentation
- [ ] User guide

---

## Next Steps

1. ✅ **Design Complete** - Architecture and syntax defined
2. **Prototype Implementation** - Build Python embedded DSL
3. **Code Generation** - Implement generators
4. **Integration** - Connect with existing codebase
5. **Testing** - Comprehensive test suite

---

**Status:** Design phase complete. Ready for prototype implementation.
