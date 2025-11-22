# Library Usage in PWA/TUI Functions

**Date:** 2025-11-18
**Status:** Analysis Complete
**Purpose:** Map QuantLib, Eigen, and NLopt usage to PWA/TUI functions

## Overview

This document identifies which PWA (Progressive Web App) and TUI (Terminal User Interface) functions can utilize QuantLib, Eigen, and NLopt libraries to enhance calculations, visualizations, and user experience.

---

## QuantLib Usage

### 1. Option Pricing & Greeks Calculations

#### **TUI Functions:**

- **Current Positions Tab** (`native/src/tui_app.cpp` - `RenderPositions()`)
  - ✅ **Greeks Display (Vega, Theta)** - Currently stub, can use QuantLib
  - ✅ **Fair Value Difference** - Compare market price vs. theoretical price
  - ✅ **ROI Calculations** - Enhanced with accurate option pricing

- **Box Spread Scenario Explorer** (`native/src/tui_app.cpp`)
  - ✅ **Scenario APR Calculations** - Accurate box spread pricing
  - ✅ **Theoretical Price Comparison** - Market vs. theoretical
  - ✅ **Probability Calculations** - Risk-adjusted probabilities

#### **PWA Functions:**

- **PositionsTable Component** (`web/src/components/PositionsTable.tsx`)
  - ✅ **Greeks Display** - Show Delta, Gamma, Vega, Theta, Rho
  - ✅ **Fair Value Indicators** - Visual indicators for mispricing

- **BoxSpreadTable Component** (`web/src/components/BoxSpreadTable.tsx`)
  - ✅ **APR Calculations** - Accurate annualized returns
  - ✅ **Theoretical Pricing** - Compare with market prices
  - ✅ **Risk Metrics** - Greeks for each scenario

- **OptionsChainTable Component** (`web/src/components/OptionsChainTable.tsx`)
  - ✅ **Implied Volatility** - Calculate IV from market prices
  - ✅ **Greeks for Each Option** - Delta, Gamma, Vega, Theta
  - ✅ **Theoretical Prices** - Black-Scholes pricing

#### **Backend Functions:**

- **OptionChainBuilder** (`native/src/option_chain.cpp`)
  - ✅ **calculate_theoretical_price()** - Replace stub with QuantLib
  - ✅ **calculate_implied_volatility()** - Replace stub with QuantLib
  - ✅ **calculate_delta/gamma/theta/vega()** - Replace stubs with QuantLib

- **RiskCalculator** (`native/src/risk_calculator.cpp`)
  - ✅ **calculate_aggregate_greeks()** - Use QuantLib for accurate Greeks
  - ✅ **Portfolio Greeks** - Aggregate across all positions

### 2. Volatility Modeling

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp` - `RenderDashboard()`)
  - ✅ **Volatility Skew Display** - Show IV skew for symbols
  - ✅ **Volatility Surface** - Visualize IV across strikes/expirations

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Volatility Metrics** - Display IV percentiles, skew
  - ✅ **Volatility Charts** - Visualize volatility surface

- **OptionsChainTable Component** (`web/src/components/OptionsChainTable.tsx`)
  - ✅ **IV Surface Visualization** - 3D surface or heatmap
  - ✅ **Skew Analysis** - Compare put/call IV

### 3. Yield Curve Construction

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Risk-Free Rate Display** - Show current T-bill rates
  - ✅ **Yield Curve Visualization** - Terminal-based chart

#### **PWA Functions:**

- **YieldCurveTable Component** (`web/src/components/YieldCurveTable.tsx`)
  - ✅ **Yield Curve Construction** - Interpolate T-bill rates
  - ✅ **Rate Comparison** - Compare box spread rates vs. T-bills
  - ✅ **Yield Curve Chart** - Visualize term structure

#### **Backend Functions:**

- **SpareCashAllocator** (from `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **T-bill Rate Interpolation** - Use QuantLib yield curves
  - ✅ **Risk-Free Rate for Option Pricing** - Accurate rates

### 4. Box Spread Pricing

#### **TUI Functions:**

- **Box Spread Scenario Explorer**
  - ✅ **Accurate Box Spread Pricing** - Four-leg option pricing
  - ✅ **Arbitrage Detection** - Compare with strike width
  - ✅ **ROI Calculations** - Annualized returns

#### **PWA Functions:**

- **BoxSpreadTable Component** (`web/src/components/BoxSpreadTable.tsx`)
  - ✅ **Box Spread Valuation** - Accurate pricing
  - ✅ **Arbitrage Opportunities** - Highlight profitable spreads
  - ✅ **Risk Metrics** - Greeks for box spreads

- **BoxSpreadCombinations Component** (`web/src/components/BoxSpreadCombinations.tsx`)
  - ✅ **Combination Analysis** - Evaluate multiple box spreads
  - ✅ **Portfolio Greeks** - Aggregate Greeks across combinations

#### **Backend Functions:**

- **BoxSpreadStrategy** (`native/src/box_spread_strategy.cpp`)
  - ✅ **Box Spread Pricing** - Replace current implementation
  - ✅ **Greeks Calculation** - Accurate Greeks for box spreads

---

## Eigen Usage

### 1. Portfolio Optimization & Allocation

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Portfolio Allocation Display** - Show current vs. target allocations
  - ✅ **Allocation Matrix Visualization** - Matrix-based display

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Portfolio Allocation Chart** - Visualize allocation percentages
  - ✅ **Allocation Matrix** - Show allocation across asset classes

#### **Backend Functions:**

- **PortfolioAllocationManager** (planned in `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **Allocation Matrix Calculations** - Use Eigen for matrix operations
  - ✅ **Target Allocation Calculations** - Matrix-based optimization
  - ✅ **Currency Conversion** - Matrix operations for multi-currency

### 2. Convexity Optimization (Barbell Strategy)

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Convexity Metrics Display** - Show portfolio convexity
  - ✅ **Barbell Allocation** - Display short/long bond allocation

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Convexity Visualization** - Chart showing convexity over time
  - ✅ **Barbell Strategy Display** - Show allocation breakdown

#### **Backend Functions:**

- **ConvexityCalculator** (planned in `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **Portfolio Convexity Calculation** - Weighted average using Eigen
  - ✅ **Barbell Allocation Matrix** - Matrix operations for optimization
  - ✅ **Convexity Drift Monitoring** - Track convexity changes

### 3. Risk Metrics & Correlation

#### **TUI Functions:**

- **Current Positions Tab** (`native/src/tui_app.cpp`)
  - ✅ **Portfolio Risk Display** - Show aggregate risk metrics
  - ✅ **Correlation Matrix** - Display position correlations

#### **PWA Functions:**

- **PositionsTable Component** (`web/src/components/PositionsTable.tsx`)
  - ✅ **Portfolio Risk Metrics** - VaR, Sharpe ratio, etc.
  - ✅ **Correlation Visualization** - Heatmap of correlations

#### **Backend Functions:**

- **RiskCalculator** (`native/src/risk_calculator.cpp`)
  - ✅ **calculate_correlation_risk()** - Use Eigen for correlation matrix
  - ✅ **Portfolio Variance Calculation** - w^T *C* w (Eigen matrix operations)
  - ✅ **Covariance Matrix** - Build and manipulate with Eigen

### 4. Greeks Aggregation

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Portfolio Greeks Display** - Aggregate Delta, Gamma, Vega, Theta
  - ✅ **Greeks Matrix** - Show Greeks across positions

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Portfolio Greeks Summary** - Aggregate Greeks display
  - ✅ **Greeks Visualization** - Charts showing Greeks over time

#### **Backend Functions:**

- **RiskCalculator** (`native/src/risk_calculator.cpp`)
  - ✅ **calculate_aggregate_greeks()** - Use Eigen VectorXd for aggregation
  - ✅ **Portfolio Greeks Matrix** - Matrix operations for Greeks

### 5. Spare Cash Allocation

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Cash Allocation Display** - Show allocation across vehicles
  - ✅ **Allocation Matrix** - Display allocation percentages

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Cash Allocation Chart** - Pie chart or bar chart
  - ✅ **Allocation Matrix** - Show box spread/T-bill/bond allocation

#### **Backend Functions:**

- **SpareCashAllocator** (planned in `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **Allocation Matrix Calculations** - Use Eigen for matrix operations
  - ✅ **Rate Comparison** - Matrix-based rate comparisons
  - ✅ **Allocation Optimization** - Matrix operations for optimization

---

## NLopt Usage

### 1. Convexity Optimization (Barbell Strategy)

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Optimal Allocation Display** - Show optimized barbell allocation
  - ✅ **Optimization Status** - Display optimization results

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Optimization Results** - Show optimal allocation
  - ✅ **Optimization Controls** - User inputs for optimization

#### **Backend Functions:**

- **ConvexityCalculator** (planned in `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **Optimize Barbell Allocation** - Use NLopt to maximize convexity
  - ✅ **Constrained Optimization** - Duration and weight constraints
  - ✅ **Rebalancing Triggers** - Optimize when convexity drifts

### 2. Portfolio Rebalancing

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Rebalancing Recommendations** - Show suggested changes
  - ✅ **Rebalancing Cost Display** - Show transaction costs

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Rebalancing Interface** - User controls for rebalancing
  - ✅ **Rebalancing Preview** - Show before/after allocation

#### **Backend Functions:**

- **PortfolioAllocationManager** (planned in `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **Optimize Rebalancing** - Minimize transaction costs
  - ✅ **Constrained Rebalancing** - Respect risk limits
  - ✅ **Tax-Efficient Rebalancing** - Optimize for tax efficiency

### 3. Spare Cash Allocation Optimization

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Optimal Cash Allocation** - Show optimized allocation
  - ✅ **Yield Maximization** - Display optimized yield

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Cash Allocation Optimizer** - Interactive optimization
  - ✅ **Optimization Results** - Show optimal allocation

#### **Backend Functions:**

- **SpareCashAllocator** (planned in `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **Maximize Yield** - Optimize allocation across vehicles
  - ✅ **Liquidity Constraints** - Constrained optimization
  - ✅ **Rate-Based Optimization** - Optimize based on current rates

### 4. Risk-Constrained Portfolio Optimization

#### **TUI Functions:**

- **Dashboard Tab** (`native/src/tui_app.cpp`)
  - ✅ **Risk-Optimized Allocation** - Show risk-adjusted allocation
  - ✅ **Risk Metrics** - Display risk constraints

#### **PWA Functions:**

- **DashboardTab Component** (`web/src/components/DashboardTab.tsx`)
  - ✅ **Risk Optimization Interface** - User controls for risk limits
  - ✅ **Risk-Adjusted Allocation** - Show optimized allocation

#### **Backend Functions:**

- **PortfolioAllocationManager** (planned in `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
  - ✅ **Risk-Constrained Optimization** - Optimize with risk limits
  - ✅ **Greeks Constraints** - Optimize subject to Greeks limits
  - ✅ **VaR Constraints** - Optimize subject to VaR limits

---

## Implementation Priority

### High Priority (Immediate Value)

1. **QuantLib:**
   - ✅ Option pricing and Greeks in `OptionChainBuilder` (replace stubs)
   - ✅ Greeks aggregation in `RiskCalculator`
   - ✅ Box spread pricing accuracy

2. **Eigen:**
   - ✅ Portfolio Greeks aggregation (matrix operations)
   - ✅ Correlation matrix calculations
   - ✅ Portfolio variance calculations

3. **NLopt:**
   - ✅ Convexity optimization (barbell strategy)
   - ✅ Spare cash allocation optimization

### Medium Priority (Enhanced Features)

1. **QuantLib:**
   - ✅ Volatility surface modeling
   - ✅ Yield curve construction
   - ✅ Implied volatility calculations

2. **Eigen:**
   - ✅ Portfolio allocation matrix operations
   - ✅ Multi-currency conversion matrices

3. **NLopt:**
   - ✅ Portfolio rebalancing optimization
   - ✅ Risk-constrained optimization

### Low Priority (Future Enhancements)

1. **QuantLib:**
   - ✅ Advanced option pricing models (Monte Carlo, etc.)
   - ✅ Exotic options support

2. **Eigen:**
   - ✅ Advanced matrix decompositions
   - ✅ Sparse matrix operations

3. **NLopt:**
   - ✅ Multi-objective optimization
   - ✅ Global optimization algorithms

---

## Integration Points

### Backend → Frontend Data Flow

```
Backend (C++ with QuantLib/Eigen/NLopt)
  ↓ Calculations
REST API / WebSocket
  ↓ JSON Data
PWA/TUI Frontend
  ↓ Display
User Interface
```

### Example: Greeks Calculation Flow

1. **Backend:** `OptionChainBuilder::calculate_delta()` uses QuantLib
2. **Backend:** `RiskCalculator::calculate_aggregate_greeks()` uses Eigen for aggregation
3. **API:** REST endpoint returns Greeks data
4. **TUI:** `RenderPositions()` displays Greeks
5. **PWA:** `PositionsTable` component displays Greeks

### Example: Convexity Optimization Flow

1. **Backend:** `ConvexityCalculator` uses NLopt to optimize allocation
2. **Backend:** Uses Eigen for matrix operations
3. **API:** REST endpoint returns optimal allocation
4. **TUI:** `RenderDashboard()` displays allocation
5. **PWA:** `DashboardTab` displays allocation chart

---

## Files to Modify

### Backend (C++)

1. **`native/src/option_chain.cpp`**
   - Replace stubs with QuantLib implementations
   - Add QuantLib includes

2. **`native/src/risk_calculator.cpp`**
   - Use QuantLib for Greeks
   - Use Eigen for aggregation and correlation

3. **`native/src/box_spread_strategy.cpp`**
   - Use QuantLib for box spread pricing

4. **`native/include/portfolio_allocation_manager.h`** (new)
   - Use Eigen for allocation matrices
   - Use NLopt for optimization

5. **`native/include/convexity_calculator.h`** (new)
   - Use Eigen for convexity calculations
   - Use NLopt for optimization

6. **`native/include/spare_cash_allocator.h`** (new)
   - Use Eigen for allocation matrices
   - Use NLopt for optimization

### Frontend (TUI)

1. **`native/src/tui_app.cpp`**
   - Display enhanced metrics from backend
   - Show optimization results

### Frontend (PWA)

1. **`web/src/components/PositionsTable.tsx`**
   - Display enhanced Greeks
   - Show fair value indicators

2. **`web/src/components/BoxSpreadTable.tsx`**
   - Display accurate pricing
   - Show risk metrics

3. **`web/src/components/DashboardTab.tsx`**
   - Display portfolio allocation
   - Show optimization results

4. **`web/src/components/OptionsChainTable.tsx`**
   - Display IV and Greeks
   - Show theoretical prices

---

## Testing Requirements

### Unit Tests

1. **QuantLib Integration Tests**
   - Option pricing accuracy
   - Greeks calculation correctness
   - Volatility surface construction

2. **Eigen Integration Tests**
   - Matrix operations correctness
   - Portfolio aggregation accuracy
   - Correlation calculations

3. **NLopt Integration Tests**
   - Optimization convergence
   - Constraint satisfaction
   - Algorithm selection

### Integration Tests

1. **End-to-End Tests**
   - Backend calculations → API → Frontend display
   - Optimization results display
   - Real-time updates

### Performance Tests

1. **Calculation Performance**
   - QuantLib pricing speed
   - Eigen matrix operation speed
   - NLopt optimization speed

---

## References

- **QuantLib Integration Guide:** `docs/QUANTLIB_INTEGRATION_GUIDE.md`
- **Eigen Integration Guide:** `docs/EIGEN_INTEGRATION.md`
- **NLopt Integration Guide:** `docs/NLOPT_INTEGRATION_GUIDE.md`
- **Investment Strategy Framework:** `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`
- **Feature Tracking:** `docs/FEATURE_TRACKING.md`

---

**Document Status:** Analysis Complete ✅
**Last Updated:** 2025-11-18
