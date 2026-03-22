# Libraries & Frameworks for Primary Goals

**Date**: 2025-11-19
**Purpose**: Identify existing and recommended libraries to avoid reinventing the wheel for unified positions, cash flow modeling, opportunity simulation, and multi-instrument relationships

---

## 🎯 Quick Reference: What to Use

### ✅ Already Available (Use These!)

| Goal | Library | Location | Action |
|------|---------|----------|--------|
| **Unified Positions** | Ledger System (Rust) | `agents/backend/crates/ledger/` | Extend queries |
| **Cash Flow Modeling** | Pandas + Chrono | `requirements-notebooks.txt` + `Cargo.toml` | Use directly |
| **Opportunity Simulation** | NLopt (C++) | `docs/NLOPT_INTEGRATION_GUIDE.md` | Integrate |
| **Relationship Modeling** | NetworkX (Python) | Not yet | `pip install networkx` |
| **Optimization** | Eigen (C++) | ✅ Integrated | Use directly |
| **Visualization** | Plotly | `requirements-notebooks.txt` | Use directly |

### 🆕 Add These (High Priority)

1. **NetworkX** (Python) - `pip install networkx`
   - ⭐⭐⭐ **CRITICAL**: Perfect for relationship chains (loan → margin → box spread → fund → cheaper loan)
   - Models relationships as graphs, finds optimal paths automatically

2. **NLopt** (C++) - Integration guide ready
   - Constrained optimization for opportunity simulation
   - Documentation: `docs/NLOPT_INTEGRATION_GUIDE.md`

3. **QuantLib-Python** (Python) - `pip install QuantLib-Python`
   - Financial date calculations, cash flow scheduling
   - Day count conventions, business day calendars

---

## Summary: What You Already Have

### ✅ Already in Codebase

1. **Ledger System** (Rust) - `agents/backend/crates/ledger/`
   - ✅ Multi-currency transaction tracking
   - ✅ Position recording
   - ✅ Account structure (Assets, Liabilities, Equity)
   - ✅ **Use for**: Unified position tracking foundation

2. **NumPy** (Python) - `requirements.txt`
   - ✅ Numerical calculations
   - ✅ Array operations
   - ✅ **Use for**: Cash flow calculations, matrix operations

3. **Pandas** (Python) - `requirements-notebooks.txt` ✅
   - ✅ Data manipulation and aggregation
   - ✅ Time-series operations
   - ✅ **Use for**: Cash flow modeling, position aggregation, scenario comparison

4. **Plotly** (Python) - `requirements-notebooks.txt` ✅
   - ✅ Interactive visualization
   - ✅ **Use for**: Cash flow charts, relationship diagrams

5. **SciPy** (Python) - `requirements-notebooks.txt` ✅
   - ✅ Optimization algorithms
   - ✅ Scientific computing
   - ✅ **Use for**: Alternative to NLopt for Python-based optimization

6. **NautilusTrader** (Python) - Already integrated
   - ✅ Position tracking
   - ✅ Portfolio management
   - ✅ **Use for**: Real-time position updates

7. **Rust Ecosystem** (Cargo.toml)
   - ✅ `chrono` - Date/time handling for cash flow projections
   - ✅ `rust_decimal` - Precise decimal arithmetic (financial calculations)
   - ✅ `serde`/`serde_json` - Data serialization
   - ✅ `tokio` - Async runtime for real-time updates
   - ✅ **Use for**: Backend position/cash flow engine

8. **Eigen** (C++) - ✅ Integrated
   - ✅ Linear algebra for optimization
   - ✅ Matrix operations
   - ✅ **Use for**: Portfolio optimization, relationship calculations

---

## Recommended Libraries by Goal

### 1. Unified Positions Panel

#### Existing (Use These First)

**Ledger System** (`agents/backend/crates/ledger/`)

- ✅ Already tracks all positions
- ✅ Multi-currency support
- ✅ Account hierarchy (Assets:Bank:*, Assets:Investments:*, etc.)
- **Action**: Extend ledger queries to return unified position view

**Rust Position Types** (`agents/backend/crates/api/src/state.rs`)

- ✅ `PositionSnapshot` already exists
- ✅ Real-time position tracking
- **Action**: Extend to include loan positions, box spreads, bonds

#### Recommended Additions

**Pandas** (Python) - ✅ **ALREADY IN CODEBASE** (`requirements-notebooks.txt`)

- **Purpose**: Data manipulation and aggregation for positions
- **Use Case**:
  - Aggregate positions by type (loans, box spreads, bonds)
  - Calculate totals, averages, weighted rates
  - Time-series operations for cash flow

- **Status**: ✅ Available in notebooks environment
- **Why**: Industry standard for financial data manipulation

**React Table / TanStack Table** (TypeScript) - **FOR PWA**

- **Purpose**: Advanced table component for positions panel
- **Use Case**:
  - Sortable, filterable positions table
  - Grouping by instrument type
  - Real-time updates

- **Install**: `npm install @tanstack/react-table`
- **Why**: Best-in-class table library for React

---

### 2. Cash Flow Modeling

#### Existing (Use These First)

**Ledger System** (`agents/backend/crates/ledger/`)

- ✅ Transaction history (source of cash flows)
- ✅ Date tracking
- ✅ Multi-currency
- **Action**: Add cash flow projection queries

**Chrono** (Rust) - Already in `Cargo.toml`

- ✅ Date/time calculations
- ✅ Duration arithmetic
- ✅ **Use for**: Maturity date calculations, cash flow scheduling

**Rust Decimal** - Already in `Cargo.toml`

- ✅ Precise decimal arithmetic (no floating-point errors)
- ✅ **Use for**: Interest calculations, cash flow amounts

#### Recommended Additions

**Pandas** (Python) - ✅ **ALREADY IN CODEBASE** (`requirements-notebooks.txt`)

- **Purpose**: Time-series cash flow modeling
- **Use Case**:
  - Project future cash flows by date
  - Aggregate cash flows by period (daily, weekly, monthly)
  - Calculate net cash flow at any point
  - Handle multiple currencies

- **Key Features**:
  - `pd.date_range()` for cash flow scheduling
  - `groupby()` for aggregating by period
  - `resample()` for time-series operations

- **Status**: ✅ Available in notebooks environment
- **Example**:

```python
import pandas as pd
from datetime import datetime, timedelta

# Project cash flows from positions

cash_flows = pd.DataFrame({
    'date': [datetime(2025, 12, 1), datetime(2025, 12, 15)],
    'amount': [-1000.0, 500.0],  # Negative = outflow, Positive = inflow
    'currency': ['USD', 'USD'],
    'position_id': ['loan-1', 'box-spread-1']
})

# Calculate net cash flow by date

net_cash_flow = cash_flows.groupby('date')['amount'].sum()
```

**QuantLib-Python** - **NOT YET IN CODEBASE**

- **Purpose**: Financial date calculations, cash flow scheduling
- **Use Case**:
  - Day count conventions (Actual/360, 30/360, etc.)
  - Business day calendars
  - Cash flow schedule generation
  - Interest accrual calculations

- **Install**: `pip install QuantLib-Python`
- **Why**: Industry standard for financial date/cash flow calculations
- **Note**: You have QuantLib C++ integration guide, Python bindings are easier for cash flow modeling

**dateutil** (Python) - **NOT YET IN CODEBASE**

- **Purpose**: Date parsing and manipulation
- **Use Case**:
  - Parse maturity dates from various formats
  - Calculate days between dates
  - Handle business days

- **Install**: `pip install python-dateutil`
- **Why**: Simple, reliable date handling

---

### 3. Opportunity Simulation (What-If Analysis)

#### Existing (Use These First)

**Risk Calculator** (`native/src/risk_calculator.cpp`)

- ✅ Position sizing calculations
- ✅ Risk metrics
- ✅ **Use for**: Validate simulated scenarios

**Box Spread Strategy** (`native/src/box_spread_strategy.cpp`)

- ✅ Opportunity evaluation
- ✅ Profitability calculations
- ✅ **Use for**: Evaluate box spread opportunities in scenarios

#### Recommended Additions

**NLopt** (C++) - **DOCUMENTATION READY, NOT INTEGRATED**

- **Purpose**: Constrained optimization for opportunity simulation
- **Use Case**:
  - Optimize loan allocation across opportunities
  - Find optimal multi-instrument chains
  - Constrained optimization (e.g., "maximize benefit subject to risk limits")

- **Status**: Integration guide exists (`docs/NLOPT_INTEGRATION_GUIDE.md`)
- **Why**: Best-in-class optimization library, perfect for "what-if" scenarios
- **Example Use**:
  - "What's the optimal allocation of this loan across consolidation, margin, and investment?"
  - "What's the best multi-instrument chain given constraints?"

**Eigen** (C++) - **INTEGRATED** ✅

- **Purpose**: Linear algebra for optimization
- **Use Case**:
  - Matrix operations for portfolio optimization
  - Solve linear systems for cash flow optimization
  - Portfolio allocation calculations

- **Status**: Already integrated (from `docs/RESEARCH_CPP_FINANCIAL_LIBRARIES.md`)
- **Why**: Essential for multi-instrument optimization

**Pandas** (Python) - ✅ **ALREADY IN CODEBASE** (`requirements-notebooks.txt`)

- **Purpose**: Scenario comparison and analysis
- **Use Case**:
  - Compare multiple "what-if" scenarios side-by-side
  - Calculate scenario metrics (net benefit, cash flow impact, risk)
  - Filter and rank scenarios

- **Status**: ✅ Available in notebooks environment
- **Example**:

```python
import pandas as pd

# Compare scenarios

scenarios = pd.DataFrame({
    'scenario': ['consolidate', 'margin', 'invest'],
    'net_benefit': [500.0, 300.0, 400.0],
    'cash_flow_impact': [-1000.0, 0.0, -500.0],
    'risk_score': [0.2, 0.5, 0.3]
})

# Rank by net benefit

best_scenario = scenarios.loc[scenarios['net_benefit'].idxmax()]
```

**NetworkX** (Python) - **NOT YET IN CODEBASE**

- **Purpose**: Graph algorithms for relationship chains
- **Use Case**:
  - Model multi-instrument relationships as graph
  - Find optimal paths (loan → margin → box spread → fund → cheaper loan)
  - Calculate shortest/optimal paths

- **Install**: `pip install networkx`
- **Why**: Perfect for modeling instrument relationships and finding chains
- **Example**:

```python
import networkx as nx

# Model relationships as graph

G = nx.DiGraph()
G.add_edge('loan-5%', 'box-spread-margin', weight=0.04, benefit=100)
G.add_edge('box-spread-margin', 'fund-investment', weight=0.06, benefit=200)
G.add_edge('fund-investment', 'loan-3%', weight=0.03, benefit=300)

# Find optimal path

path = nx.shortest_path(G, 'loan-5%', 'loan-3%', weight='weight')
```

---

### 4. Multi-Instrument Relationship Modeling

#### Existing (Use These First)

**Synthetic Financing Architecture** (`docs/SYNTHETIC_FINANCING_ARCHITECTURE.md`)

- ✅ Complete design for asset relationships
- ✅ Relationship types defined (Collateral, Financing, Cross-Currency)
- ✅ **Action**: Implement the designed architecture

**Ledger System** (`agents/backend/crates/ledger/`)

- ✅ Account structure supports relationships
- ✅ Multi-currency
- ✅ **Action**: Extend to track relationship metadata

#### Recommended Additions

**NetworkX** (Python) - **NOT YET IN CODEBASE** ⭐ **HIGHLY RECOMMENDED**

- **Purpose**: Graph-based relationship modeling
- **Use Case**:
  - Model asset relationships as directed graph
  - Find relationship chains (loan → margin → box spread → fund → cheaper loan)
  - Calculate optimal paths
  - Visualize relationships

- **Why**: Perfect fit for your use case - relationships are graphs!
- **Install**: `pip install networkx`
- **Example**:

```python
import networkx as nx

# Create relationship graph

G = nx.DiGraph()

# Add relationships

G.add_edge('bank-loan-5%', 'box-spread-margin',
           relationship='collateral',
           rate_benefit=0.01,  # 1% benefit
           collateral_ratio=0.95)

G.add_edge('box-spread-margin', 'fund-investment',
           relationship='financing',
           rate=0.04,
           return_rate=0.06)

# Find optimal chain

paths = list(nx.all_simple_paths(G, 'bank-loan-5%', 'cheaper-loan-3%'))
optimal_path = max(paths, key=lambda p: calculate_path_benefit(G, p))
```

**Eigen** (C++) - **INTEGRATED** ✅

- **Purpose**: Matrix operations for relationship calculations
- **Use Case**:
  - Collateral valuation matrices
  - Portfolio margin calculations
  - Relationship strength calculations

- **Status**: Already integrated

**NLopt** (C++) - **DOCUMENTATION READY**

- **Purpose**: Optimize relationship chains
- **Use Case**:
  - Find optimal multi-instrument chain
  - Constrained optimization (e.g., "maximize benefit subject to risk")

- **Status**: Integration guide exists

---

## Python Libraries Summary

### High Priority (Add These)

| Library | Purpose | Install | Priority |
|---------|---------|---------|----------|
| **pandas** | Cash flow modeling, position aggregation, scenario comparison | ✅ **ALREADY IN CODEBASE** (`requirements-notebooks.txt`) | ⭐⭐⭐ HIGH |
| **networkx** | Relationship modeling, optimal path finding | `pip install networkx` | ⭐⭐⭐ HIGH |
| **QuantLib-Python** | Financial date calculations, cash flow scheduling | `pip install QuantLib-Python` | ⭐⭐ MEDIUM |
| **python-dateutil** | Date parsing and manipulation | `pip install python-dateutil` | ⭐⭐ MEDIUM |

### Already Available ✅

| Library | Purpose | Location | Status |
|---------|---------|----------|--------|
| **pandas** | Data manipulation, cash flow modeling | `requirements-notebooks.txt` | ✅ Available |
| **numpy** | Numerical calculations | `requirements.txt` | ✅ Available |
| **plotly** | Visualization | `requirements-notebooks.txt` | ✅ Available |
| **scipy** | Optimization, scientific computing | `requirements-notebooks.txt` | ✅ Available |
| **matplotlib** | Plotting | `requirements-notebooks.txt` | ✅ Available |
| **seaborn** | Statistical visualization | `requirements-notebooks.txt` | ✅ Available |

### Medium Priority (Consider)

| Library | Purpose | Install | Priority |
|---------|---------|---------|----------|
| **networkx** | Relationship modeling, optimal path finding | `pip install networkx` | ⭐⭐⭐ HIGH (Add to requirements) |
| **QuantLib-Python** | Financial date calculations | `pip install QuantLib-Python` | ⭐⭐ MEDIUM |

---

## C++ Libraries Summary

### Already Integrated ✅

| Library | Status | Use For |
|---------|--------|---------|
| **Eigen** | ✅ Integrated | Linear algebra, portfolio optimization |

### Documentation Ready (Not Integrated)

| Library | Status | Use For | Integration Guide |
|---------|--------|---------|-------------------|
| **NLopt** | 📋 Docs Ready | Optimization, opportunity simulation | `docs/NLOPT_INTEGRATION_GUIDE.md` |
| **QuantLib** | 📋 Docs Ready | Option pricing, Greeks, yield curves | `docs/QUANTLIB_INTEGRATION_GUIDE.md` |

---

## TypeScript/React Libraries (PWA)

### Recommended

| Library | Purpose | Install | Priority |
|---------|---------|---------|----------|
| **@tanstack/react-table** | Advanced positions table | `npm install @tanstack/react-table` | ⭐⭐⭐ HIGH |
| **recharts** or **victory** | Cash flow charts | `npm install recharts` | ⭐⭐ MEDIUM |
| **react-flow** | Relationship diagram visualization | `npm install reactflow` | ⭐ MEDIUM |

---

## Implementation Strategy

### Phase 1: Foundation (Use Existing)

1. **Extend Ledger System**:
   - Add unified position query endpoint
   - Add cash flow projection queries
   - Track relationship metadata

2. **Use Existing Rust Libraries**:
   - `chrono` for date calculations
   - `rust_decimal` for financial math
   - `tokio` for async updates

### Phase 2: Add Python Libraries (High Priority)

1. **Pandas**:
   - Cash flow modeling
   - Position aggregation
   - Scenario comparison

2. **NetworkX**:
   - Relationship graph modeling
   - Optimal path finding
   - Chain discovery

### Phase 3: Add C++ Libraries (If Needed)

1. **NLopt**:
   - Constrained optimization
   - Opportunity simulation optimization

2. **QuantLib** (if needed):
   - Advanced option pricing
   - Greeks calculations

### Phase 4: Add Visualization (PWA)

1. **React Table**: Positions panel
2. **Recharts**: Cash flow charts
3. **React Flow**: Relationship diagrams

---

## Quick Start Recommendations

### For Cash Flow Modeling

**Start with**: Pandas + Chrono (Rust) + Ledger System

- Ledger provides transaction history
- Chrono handles dates
- Pandas models and projects cash flows

### For Opportunity Simulation

**Start with**: NetworkX + NLopt + Existing Risk Calculator

- NetworkX models relationships
- NLopt optimizes scenarios
- Risk Calculator validates scenarios

### For Unified Positions Panel

**Start with**: Extend Ledger System + React Table

- Ledger already tracks positions
- Extend queries to return unified view
- React Table displays in PWA

### For Relationship Modeling

**Start with**: NetworkX + Existing Architecture Design

- Architecture document has complete design
- NetworkX implements the graph model
- Find optimal chains automatically

---

## Code Examples

### Cash Flow Modeling with Pandas

```python
import pandas as pd
from datetime import datetime, timedelta

def project_cash_flows(positions, start_date, end_date):
    """Project cash flows from all positions."""
    cash_flows = []

    for position in positions:
        if position.type == 'loan':
            # Loan interest payments
            payment_dates = generate_payment_schedule(
                position.start_date,
                position.maturity_date,
                position.payment_frequency
            )
            for date in payment_dates:
                if start_date <= date <= end_date:
                    cash_flows.append({
                        'date': date,
                        'amount': -position.interest_payment,  # Outflow
                        'currency': position.currency,
                        'position_id': position.id,
                        'type': 'interest_payment'
                    })

        elif position.type == 'box_spread':
            # Box spread maturity
            if start_date <= position.maturity_date <= end_date:
                cash_flows.append({
                    'date': position.maturity_date,
                    'amount': position.strike_width,  # Inflow
                    'currency': position.currency,
                    'position_id': position.id,
                    'type': 'maturity'
                })

    # Create DataFrame
    df = pd.DataFrame(cash_flows)

    # Calculate net cash flow by date
    net_cash_flow = df.groupby('date')['amount'].sum()

    # Calculate cumulative cash flow
    cumulative = net_cash_flow.cumsum()

    return {
        'cash_flows': df,
        'net_by_date': net_cash_flow,
        'cumulative': cumulative
    }
```

### Relationship Modeling with NetworkX

```python
import networkx as nx

def build_relationship_graph(positions, relationships):
    """Build graph of instrument relationships."""
    G = nx.DiGraph()

    # Add nodes (positions)
    for position in positions:
        G.add_node(position.id,
                  type=position.type,
                  rate=position.rate,
                  currency=position.currency)

    # Add edges (relationships)
    for rel in relationships:
        G.add_edge(rel.source_id, rel.target_id,
                  relationship_type=rel.type,
                  benefit=rel.benefit,
                  constraints=rel.constraints)

    return G

def find_optimal_chain(G, start_id, end_id, max_length=5):
    """Find optimal relationship chain."""
    # Find all paths
    paths = list(nx.all_simple_paths(G, start_id, end_id, cutoff=max_length))

    if not paths:
        return None

    # Calculate benefit for each path
    path_benefits = []
    for path in paths:
        benefit = sum(G[u][v]['benefit'] for u, v in zip(path[:-1], path[1:]))
        path_benefits.append((path, benefit))

    # Return optimal path
    optimal_path, optimal_benefit = max(path_benefits, key=lambda x: x[1])

    return {
        'path': optimal_path,
        'benefit': optimal_benefit,
        'all_paths': paths
    }
```

### Opportunity Simulation with NLopt

```cpp
// C++ example using NLopt for opportunity optimization

#include <nlopt.hpp>
#include <vector>

// Objective: maximize net benefit from loan usage
double loan_usage_objective(unsigned n, const double* x, double* grad, void* data) {
    // x[0] = allocation to loan consolidation
    // x[1] = allocation to box spread margin
    // x[2] = allocation to fund investment

    double consolidate_benefit = 200.0;  // $200 benefit
    double margin_benefit = 150.0;       // $150 benefit
    double invest_benefit = 180.0;          // $180 benefit

    double total_benefit = x[0] * consolidate_benefit +
                          x[1] * margin_benefit +
                          x[2] * invest_benefit;

    // Minimize negative benefit (maximize benefit)
    return -total_benefit;
}

// Constraint: allocations sum to 1.0 (100% of loan)
double allocation_constraint(unsigned n, const double* x, double* grad, void* data) {
    double sum = x[0] + x[1] + x[2] - 1.0;

    if (grad) {
        grad[0] = 1.0;
        grad[1] = 1.0;
        grad[2] = 1.0;
    }

    return sum;  // Must equal 0
}

void optimize_loan_usage() {
    nlopt::opt opt(nlopt::LD_SLSQP, 3);  // 3 variables

    opt.set_min_objective(loan_usage_objective, nullptr);

    // Bounds: 0 to 1 (percentages)
    std::vector<double> lb(3, 0.0);
    std::vector<double> ub(3, 1.0);
    opt.set_lower_bounds(lb);
    opt.set_upper_bounds(ub);

    // Constraint: sum to 1.0
    opt.add_equality_constraint(allocation_constraint, nullptr, 1e-8);

    // Initial guess: equal allocation
    std::vector<double> x(3, 1.0/3.0);

    // Optimize
    double minf;
    nlopt::result result = opt.optimize(x, minf);

    // x now contains optimal allocation
    // minf is negative of maximum benefit
}
```

---

## References

- `docs/RESEARCH_CPP_FINANCIAL_LIBRARIES.md` - C++ library research
- `docs/NLOPT_INTEGRATION_GUIDE.md` - NLopt integration
- `docs/QUANTLIB_INTEGRATION_GUIDE.md` - QuantLib integration
- `docs/SYNTHETIC_FINANCING_ARCHITECTURE.md` - Relationship architecture
- `agents/backend/crates/ledger/` - Ledger system
