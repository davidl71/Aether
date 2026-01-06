# Financial DSL Research and Design for Box Spread Synthetic Financing

**Date**: 2025-11-19
**Purpose**: Research financial DSL implementations and design DSL architecture for box spread synthetic financing scenarios

---

## Executive Summary

This document researches existing financial Domain-Specific Languages (DSLs) and designs a DSL architecture for expressing box spread synthetic
financing scenarios, multi-asset relationships, and cash flow modeling in the IBKR box spread project.

**Key Findings:**

- **Quant DSL** (Python) and **MLFi** (OCaml) use combinator-based approaches for financial contracts
- **Rebel** (ING bank) provides lightweight formal specification with domain types (Money, IBAN)
- **COBOL** patterns show importance of domain-specific types and business-readable syntax
- **Icon Solutions MPS** demonstrates code generation from DSL models to executable code

**Recommendation:** Start with Python embedded DSL (internal DSL) for rapid prototyping, then consider external DSL with code generation for
production use.

---

## Research: Financial DSL Implementations

### 1. Quant DSL (Python)

**Source:** [dslfin.org/resources.html](https://www.dslfin.org/resources.html)

**Key Characteristics:**

- Python-based DSL for expressing financial contracts
- Follows combinator approach from MLFi paper
- Can be evaluated against price processes (e.g., multi-market Black-Scholes)
- Based on "Composing Contracts: An Adventure in Financial Engineering" paper

**Relevance to Box Spreads:**

- Combinator approach could express box spread legs declaratively
- Python integration aligns with existing Python bindings
- Can generate pricing formulas and documentation

**Example Pattern (Inferred):**

```python

# Hypothetical Quant DSL pattern

contract = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19") \
    .direction(Lending) \
    .min_rate(4.5)
```

### 2. MLFi (LexiFi - OCaml)

**Source:** [dslfin.org/resources.html](https://www.dslfin.org/resources.html), [Stack Overflow
Discussion](https://stackoverflow.com/questions/23448/dsls-domain-specific-languages-in-finance)

**Key Characteristics:**

- OCaml-based contract modeling language
- Combinator library for composing contracts
- Used in production pricing and operations management
- Based on "Composing Contracts: An Adventure in Financial Engineering" (Simon Peyton Jones, Jean-Marc Eber)

**Relevance:**

- Demonstrates combinator pattern for financial instruments
- Shows how to compose complex contracts from primitives
- Production-proven approach

**Key Insight:** Contracts are composed using combinators (When, Scale, Give, And, Or) that can be combined to express complex payoffs.

### 3. Rebel DSL (ING Bank / CWI)

**Source:** [CWI Rebel
Page](https://www.cwi.nl/en/research/software-analysis-and-transformation/software/rebel-a-domain-specific-language-for-product-development-in-finance/)

**Key Characteristics:**

- Lightweight formal specification language
- Domain-specific types: `Money`, `IBAN`
- High-level primitives for financial transactions and states
- Written in Rascal (metaprogramming language)
- Enables visualization, simulation, and validation
- Deployed in product development at ING bank

**Relevance:**

- Shows importance of domain-specific types (Money, IBAN) - we need `Rate`, `StrikeWidth`, `Expiration`
- Formal specification approach could validate box spread scenarios
- Visualization and simulation capabilities align with our needs

**Key Insight:** Domain-specific types (Money, IBAN) make the language more expressive and type-safe for financial domain.

### 4. COBOL Patterns

**Source:** Web search results, [IBM COBOL](https://www.ibm.com/think/topics/cobol)

**Key Characteristics:**

- English-like, self-documenting syntax
- Strong support for fixed-point decimal calculations (critical for finance)
- Designed for business, finance, and administrative systems
- Still supports $3+ trillion in daily commerce

**Relevance:**

- Emphasizes readability for domain experts
- Fixed-point decimal precision important for interest rate calculations
- Business-readable syntax aligns with our goal of enabling domain experts

**Key Insight:** Business-readable syntax and precise decimal arithmetic are essential for financial DSLs.

### 5. Icon Solutions MPS Approach

**Source:** [Icon Solutions DSL
Article](https://iconsolutions.com/blog/accelerating-software-engineering-through-the-adoption-of-domain-specific-languages)

**Key Characteristics:**

- Uses JetBrains Meta Programming System (MPS) for external DSL
- Generates Java code, BDD tests, graphs, and documentation
- Projectional editor (AST-based, not text-based)
- Compile-time safety and consistency
- Can generate multiple outputs from same model

**Relevance:**

- Code generation approach could generate C++ from DSL
- Multiple output formats (code, tests, docs, graphs) valuable
- Projectional editor ensures consistency

**Key Insight:** External DSLs with code generation can produce high-quality, consistent output across multiple formats.

---

## Current State Analysis

### Existing Patterns in Codebase

**1. Box Spread Scenario Definition:**

```typescript
// web/src/types.ts
export interface BoxSpreadScenario {
  width: number;
  put_bid: number;
  call_ask: number;
  synthetic_bid: number;
  synthetic_ask: number;
  mid_price: number;
  annualized_return: number;
  fill_probability: number;
  option_style: 'European' | 'American';
}
```

**2. JSON Configuration:**

```json
// config/config.example.json
{
  "strategy": {
    "symbols": ["SPX", "XSP", "NDX"],
    "min_implied_rate_advantage_bps": 50,
    "min_implied_rate_percent": 4.0,
    "benchmark_rate_source": "SOFR"
  }
}
```

**3. C++ Box Spread Structure:**

```cpp
// native/include/types.h
struct BoxSpreadLeg {
    OptionContract long_call;
    OptionContract short_call;
    OptionContract long_put;
    OptionContract short_put;
    double net_debit;
    double buy_implied_rate;
    double sell_implied_rate;
    // ... more fields
};
```

### Gaps and Opportunities

1. **No Declarative Scenario Language:** Scenarios are defined imperatively in C++/TypeScript
2. **Configuration vs. Logic Separation:** JSON config is separate from business logic
3. **No Multi-Asset Relationship DSL:** Asset relationships defined in C++ structs, not declaratively
4. **Limited Expressiveness:** Cannot easily express "loan → margin → box spread" relationships

---

## DSL Design: Box Spread Synthetic Financing

### Design Goals

1. **Domain Expert Readability:** Finance professionals should understand DSL syntax
2. **Type Safety:** Domain-specific types (Rate, StrikeWidth, Expiration, Money)
3. **Composability:** Combine primitives to express complex scenarios
4. **Code Generation:** Generate C++ execution code, tests, and documentation
5. **Validation:** Compile-time validation of box spread constraints

### Proposed Syntax (Python Embedded DSL)

```python

# python/dsl/box_spread_dsl.py

from box_spread_dsl import *

# Simple box spread scenario

scenario = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19") \
    .direction(Lending) \
    .min_implied_rate(4.5) \
    .benchmark(SOFR) \
    .min_advantage_bps(50) \
    .execute()

# Multi-asset financing strategy

strategy = FinancingStrategy("optimize_loan_to_box_spread") \
    .source(bank_loan(rate=5.5, amount=100000)) \
    .use_as(margin_for(
        box_spread(symbol="SPX", min_rate=4.0)
    )) \
    .then(invest_in(fund="providence", rate=3.0)) \
    .optimize(minimize_total_cost())

# Cash flow modeling

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

### Domain-Specific Types

```python

# Domain types for type safety and expressiveness

@dataclass
class Rate:
    value: float
    unit: str = "percent"  # or "bps"

@dataclass
class StrikeWidth:
    value: float
    currency: str = "USD"

@dataclass
class Expiration:
    date: str  # ISO format
    days_to_expiry: int

@dataclass
class Money:
    amount: float
    currency: str
```

### Multi-Asset Relationship DSL

```python

# Express asset relationships declaratively

relationships = AssetRelationships() \
    .collateral(
        source="T-BILL-3M-USD",
        target="SPX-OPTIONS",
        ratio=0.95,  # 95% collateral value
        brokers=["IBKR", "ALPACA"]
    ) \
    .financing(
        source="BOX-SPREAD-SPX",
        target="MARGIN-LOAN",
        rate_advantage_bps=50
    ) \
    .optimize(
        objective=minimize_total_cost,
        constraints=[
            max_exposure(50000),
            max_positions(10)
        ]
    )
```

---

## Implementation Phases

### Phase 1: Python Embedded DSL (Internal DSL)

**Goal:** Rapid prototyping with Python syntax

**Deliverables:**

- `python/dsl/box_spread_dsl.py` - DSL builder classes
- `python/dsl/types.py` - Domain-specific types
- `python/dsl/generator.py` - Code generator to C++
- Tests and examples

**Benefits:**

- Fast to implement
- Leverages existing Python bindings
- Domain experts can use Python
- Can generate C++ code

### Phase 2: External DSL with Code Generation

**Goal:** Production-ready DSL with full code generation

**Options:**

- **MPS (Meta Programming System):** Generate C++ from DSL model
- **Xtext (Eclipse):** Text-based DSL with code generation
- **Custom Parser:** ANTLR/Lark parser with code generation

**Deliverables:**

- DSL syntax definition
- Parser/compiler
- Code generator (DSL → C++)
- Test generator
- Documentation generator
- Graph visualization generator

**Benefits:**

- Complete separation of domain model from implementation
- Multiple output formats (code, tests, docs, graphs)
- Compile-time validation
- Version control friendly

### Phase 3: Multi-Asset Relationship DSL

**Goal:** Declarative language for asset relationships

**Deliverables:**

- Relationship DSL syntax
- Relationship validator
- Optimization engine integration
- Visualization of relationship graphs

---

## Code Generation Strategy

### Generated Artifacts

1. **C++ Code:**
   - `BoxSpreadScenario` structs
   - Strategy execution code
   - Validation logic

2. **Tests:**
   - BDD scenarios (Gherkin)
   - Unit tests for each scenario
   - Integration tests

3. **Documentation:**
   - AsciiDoc documentation
   - API documentation
   - User guides

4. **Visualizations:**
   - Graphviz graphs of scenarios
   - Relationship diagrams
   - Cash flow charts

### Example: DSL → C++ Generation

**DSL Input:**

```python
scenario = BoxSpread("SPX") \
    .strike_width(50) \
    .expiration("2025-12-19") \
    .min_implied_rate(4.5)
```

**Generated C++:**

```cpp
// Generated from DSL
namespace generated {
    struct SPX_BoxSpread_Scenario {
        static constexpr const char* symbol = "SPX";
        static constexpr double strike_width = 50.0;
        static constexpr const char* expiration = "2025-12-19";
        static constexpr double min_implied_rate = 4.5;

        static types::BoxSpreadLeg evaluate(
            const option_chain::OptionChain& chain
        ) {
            // Generated evaluation logic
        }
    };
}
```

---

## Validation and Constraints

### Box Spread Constraints

1. **Strike Width Consistency:** `strike_width == (high_strike - low_strike)`
2. **Expiration Alignment:** All legs must have same expiration
3. **Symbol Consistency:** All legs must be same underlying
4. **Rate Validation:** Implied rate must be positive and reasonable (0-20%)
5. **Liquidity Checks:** Minimum volume, open interest, bid-ask spread

### Multi-Asset Constraints

1. **Collateral Sufficiency:** Collateral value must exceed margin requirements
2. **Currency Matching:** Cross-currency relationships must specify FX rates
3. **Regulatory Compliance:** Portfolio margin vs. Reg-T constraints
4. **Broker Compatibility:** Relationships must be valid for specified brokers

---

## Integration with Existing Codebase

### Python Bindings Integration

```python

# DSL uses existing Python bindings

from box_spread_bindings import BoxSpreadStrategy, BoxSpreadLeg

scenario = BoxSpread("SPX").strike_width(50)
leg = scenario.evaluate()  # Returns BoxSpreadLeg
strategy = BoxSpreadStrategy(client, order_mgr, params)
opportunity = strategy.evaluate_box_spread(leg)
```

### Configuration Integration

```python

# DSL can read from config

from config_manager import load_config

config = load_config("config/config.json")
scenario = BoxSpread(config.strategy.symbols[0]) \
    .min_implied_rate(config.strategy.min_implied_rate_percent) \
    .benchmark(config.strategy.benchmark_rate_source)
```

---

## Next Steps

1. ✅ **Research Complete** - Documented DSL implementations and patterns
2. **Design DSL Architecture** - Detailed syntax and semantics
3. **Prototype Python Embedded DSL** - Working implementation
4. **Create Relationship DSL Design** - Multi-asset relationship language
5. **Store Research as Knowledge** - Add to project memory

---

## References

- [dslfin.org/resources.html](https://www.dslfin.org/resources.html) - Comprehensive financial DSL listing
- [Stack Overflow: DSLs in Finance](https://stackoverflow.com/questions/23448/dsls-domain-specific-languages-in-finance) - Community discussion
- [CWI Rebel DSL](https://www.cwi.nl/en/research/software-analysis-and-transformation/software/rebel-a-domain-specific-language-for-product-development-in-finance/) - ING bank DSL
- [Icon Solutions:
  DSL Adoption](https://iconsolutions.com/blog/accelerating-software-engineering-through-the-adoption-of-domain-specific-languages) - MPS approach
- [IBM COBOL](https://www.ibm.com/think/topics/cobol) - Legacy financial language patterns

---

**Status:** Research phase complete. Ready for design and implementation.
