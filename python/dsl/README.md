# Box Spread DSL

Domain-Specific Language for Box Spread Synthetic Financing

## Overview

This module provides a Python embedded DSL (internal DSL) for expressing box spread synthetic financing scenarios, multi-asset relationships, and cash flow models.

## Installation

```bash
# From project root
cd python
pip install -e .
```

## Quick Start

### Basic Box Spread Scenario

```python
from box_spread_dsl import BoxSpread, Direction, Benchmark

# Simple scenario
scenario = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19")

# With financing constraints
scenario = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19") \
    .direction(Direction.LENDING) \
    .min_implied_rate(4.5) \
    .benchmark(Benchmark.SOFR) \
    .min_advantage_bps(50)

# Evaluate scenario
result = scenario.evaluate()
if result.is_valid():
    print(f"Found {len(result.opportunities)} opportunities")
```

### Multi-Asset Financing Strategy

```python
from box_spread_dsl import (
    FinancingStrategy, bank_loan, box_spread,
    margin_for, invest_in, minimize_total_cost
)

strategy = FinancingStrategy("optimize_loan_to_box_spread") \
    .source(bank_loan(rate=5.5, amount=100000)) \
    .use_as(margin_for(
        box_spread(symbol="SPX", min_rate=4.0)
    )) \
    .then(invest_in(fund="providence", rate=3.0)) \
    .optimize(minimize_total_cost())
```

### Cash Flow Modeling

```python
from box_spread_dsl import (
    CashFlowModel, box_spread_lending,
    bank_loan, pension_loan
)

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
            payments="monthly"
        )
    ) \
    .project(months=12) \
    .optimize("net_cash_flow")

result = cash_flow.simulate()
print(f"Total Net Cash Flow: {result.total_net_cash_flow}")
```

### Code Generation

```python
# Generate C++ code from DSL
scenario = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19") \
    .min_implied_rate(4.5)

cpp_code = scenario.to_cpp()
print(cpp_code)
```

## Examples

See `examples.py` for more usage examples.

## Architecture

- **Types**: Domain-specific types (`Rate`, `StrikeWidth`, `Expiration`, `Money`)
- **Builders**: Fluent interface builders (`BoxSpread`, `FinancingStrategy`, `CashFlowModel`)
- **Code Generation**: Generate C++ code from DSL models
- **Validation**: Compile-time validation of constraints

## Integration

The DSL integrates with:
- Existing C++ bindings (`box_spread_bindings`)
- Configuration system (`config_manager`)
- Asset relationship graph (future)

## Status

**Phase 1: Prototype** ✅ Complete
- Domain-specific types
- Box spread builder
- Basic validation
- Code generation (stub)

**Phase 2: Integration** (In Progress)
- C++ bindings integration
- Full evaluation logic
- Test generation

**Phase 3: Advanced Features** (Planned)
- Multi-asset relationships
- Cash flow simulation
- Graph visualization

## References

- [DSL Research and Design](../docs/DSL_RESEARCH_AND_DESIGN.md)
- [DSL Architecture Design](../docs/DSL_ARCHITECTURE_DESIGN.md)
