# Synthetic Financing Platform - Core Documentation

**Status**: Active Development
**Purpose**: Comprehensive multi-asset financing optimization system

---

## Platform Overview

The **Synthetic Financing Platform** provides comprehensive multi-asset financing optimization across options, futures, bonds, bank loans, and pension funds. It enables unified portfolio management, cash flow modeling, opportunity simulation, and multi-instrument relationship optimization across 21+ accounts and multiple brokers.

---

## Core Platform Capabilities

### 1. Multi-Account Aggregation

**Purpose**: Unified view across all accounts and brokers

**Features**:

- Aggregates positions from 21+ accounts
- Supports US brokers (IBKR, Alpaca, Tradier, Tastytrade)
- Supports Israeli banks (Fibi, Discount)
- Supports Israeli brokers (Meitav, IBI)
- Supports pension funds (9 accounts)
- Account-level and portfolio-level views
- Currency conversion (ILS → USD)

**Documentation**:

- [Multi-Account Aggregation Design](MULTI_ACCOUNT_AGGREGATION_DESIGN.md)

---

### 2. Cash Flow Modeling & Forecasting

**Purpose**: Project and optimize cash flows across all positions

**Features**:

- Track cash inflows/outflows from all positions
- Project future cash flows based on maturity dates
- Calculate net cash flow at any point in time
- Handle multiple currencies
- Account for interest payments, principal repayments, dividends

**Use Cases**:

- "What's my cash flow next month?"
- "When do I need to repay this loan?"
- "How much cash will I have available for new opportunities?"

**Documentation**:

- [Primary Goals and Requirements](PRIMARY_GOALS_AND_REQUIREMENTS.md) - Section 2: Cash Flow Modeling

---

### 3. Opportunity Simulation

**Purpose**: What-if analysis for loan usage and optimization

**Key Scenarios**:

- **Loan Consolidation**: Use loan proceeds to consolidate other loans
- **Margin for Box Spreads**: Use loan as collateral for box spread margin
- **Investment Fund Strategy**: Use loan to invest in fund, use fund as collateral for cheaper loan
- **Multi-Instrument Optimization**: Optimal chains (loan → margin → box spread → fund → cheaper loan)

**Features**:

- Interactive what-if analysis
- Real-time simulation as user changes parameters
- Compare multiple scenarios side-by-side
- Show cash flow impact, net benefit, risk metrics

**Documentation**:

- [Primary Goals and Requirements](PRIMARY_GOALS_AND_REQUIREMENTS.md) - Section 3: Opportunity Simulation

---

### 4. Multi-Instrument Relationship Modeling

**Purpose**: Model relationships between instruments (loan → margin → box spread → fund → cheaper loan)

**Relationship Types**:

1. **Collateral Relationships**: Asset can be used as collateral for another
2. **Financing Relationships**: Asset provides financing for another
3. **Cash Flow Relationships**: Asset generates cash flow used by another
4. **Optimization Chains**: Optimal sequence of instrument usage

**Features**:

- Asset relationship graph
- Find optimal chains
- Calculate net benefit of chains
- Visualize relationships
- Real-time updates as positions change

**Documentation**:

- [Synthetic Financing Architecture](SYNTHETIC_FINANCING_ARCHITECTURE.md)

---

### 5. Investment Strategy Framework

**Purpose**: Portfolio allocation, convexity optimization, volatility skew management

**Components**:

- **Portfolio Allocation**: 70-80% core investments (equity + bond ETFs)
- **Convexity Optimization**: Barbell strategy (short-term + long-term bonds)
- **Volatility Skew Management**: Positive-skew assets for risk-adjusted returns
- **Cash Management**: Multi-tier allocation (3-5% immediate, 7-10% spare cash)
- **T-Bill/Bond Ladder**: 10-15% target allocation

**Documentation**:

- [Investment Strategy Framework](INVESTMENT_STRATEGY_FRAMEWORK.md)

---

## Strategy Modules

### Box Spread Strategy ⭐ (Active)

**Allocation**: 7-10% of portfolio (spare cash)
**Purpose**: Synthetic financing via options arbitrage
**Documentation**: [Box Spread Strategy](../strategies/box-spread/README.md)

### Futures Strategy (Planned)

**Allocation**: TBD
**Purpose**: Implied financing rates from futures
**Status**: Design phase

### Bond Strategy (Planned)

**Allocation**: 30-40% of portfolio (core investments)
**Purpose**: Direct financing via bond ETFs
**Status**: Design phase

### Loan Strategy (Planned)

**Allocation**: Variable
**Purpose**: Secured financing via bank/pension loans
**Status**: Design phase

---

## Platform Architecture

### Core Components

```
Synthetic Financing Platform
├── Platform Core
│   ├── Multi-Account Aggregator
│   ├── Cash Flow Modeler
│   ├── Opportunity Simulator
│   ├── Asset Relationship Graph
│   └── Investment Strategy Framework
│
├── Strategy Modules
│   ├── Box Spread Strategy (Active)
│   ├── Futures Strategy (Planned)
│   ├── Bond Strategy (Planned)
│   └── Loan Strategy (Planned)
│
├── Broker Integration
│   ├── IBKR (TWS API, Client Portal API)
│   ├── Alpaca API
│   ├── Tradier API
│   ├── Tastytrade API
│   └── Israeli Brokers (Excel/RTD/DDE/Web scraping)
│
├── Backend Services
│   ├── Rust Backend (agents/backend/)
│   ├── NATS Message Queue
│   └── QuestDB Time-Series Database
│
└── Frontend Applications
    ├── Web PWA (TypeScript/React)
    ├── TUI (Python/Textual)
    └── iPad App (SwiftUI)
```

---

## Multi-Asset Support

### Asset Types

1. **Box Spreads** (Synthetic Financing)
   - Options-based synthetic borrowing/lending
   - Implied interest rate calculation
   - T-bill-equivalent yields

2. **Futures** (Implied Financing)
   - Futures-based financing rates
   - Roll costs and basis risk
   - Delivery and settlement

3. **Bonds/T-Bills** (Direct Financing)
   - Direct lending via bonds
   - T-bill ladder management
   - Coupon and maturity cash flows

4. **Bank Loans** (Direct Financing)
   - Israeli bank loans (Fibi, Discount)
   - Interest rate tracking
   - Payment scheduling

5. **Pension Loans** (Secured Financing)
   - Pension fund secured loans
   - Collateral requirements
   - Repayment terms

6. **ETFs** (Portfolio Allocation)
   - Equity ETFs (40-50% of portfolio)
   - Bond ETFs (30-40% of portfolio)
   - International diversification

---

## Multi-Broker Architecture

### Supported Brokers

- **IBKR**: TWS API (C++), Client Portal API (REST)
- **Alpaca**: REST API (Python)
- **Tradier**: REST API (Python)
- **Tastytrade**: REST API (Python)
- **Israeli Brokers**: Excel/RTD/DDE integration, web scraping
- **Israeli Banks**: Reconciliation file import

### Broker Abstraction

- Unified broker interface
- Broker selection and switching
- Best execution across all sources
- Fallback strategies

---

## Core Documentation

### Architecture & Design

- [Synthetic Financing Architecture](SYNTHETIC_FINANCING_ARCHITECTURE.md) - Multi-asset relationship system
- [Multi-Account Aggregation Design](MULTI_ACCOUNT_AGGREGATION_DESIGN.md) - Account aggregation system
- [Investment Strategy Framework](INVESTMENT_STRATEGY_FRAMEWORK.md) - Portfolio allocation framework

### Goals & Requirements

- [Primary Goals and Requirements](PRIMARY_GOALS_AND_REQUIREMENTS.md) - System objectives

---

## Integration Points

### Box Spread Strategy Integration

- **Cash Management**: Tier 2 spare cash allocation (7-10%)
- **Opportunity Simulation**: "What-if" scenarios for margin usage
- **Multi-Instrument Relationships**: Part of financing chains
- **Risk Calculator**: Uses platform risk assessment

### Strategy Module Pattern

All strategies follow the same integration pattern:

1. **Strategy Engine**: Core calculation logic
2. **Platform Integration**: Cash flow, risk, opportunity simulation
3. **Broker Abstraction**: Works with all supported brokers
4. **Documentation**: Strategy-specific guides in `docs/strategies/<strategy-name>/`

---

## Development Roadmap

### ✅ Completed

- Multi-account aggregation architecture design
- Cash flow modeling system design
- Opportunity simulation framework design
- Investment strategy framework design
- Box spread strategy implementation
- Multi-broker architecture design

### ⏳ In Progress

- Cash flow forecasting implementation
- Opportunity simulator implementation
- Asset relationship graph implementation
- Multi-account aggregator implementation

### 📋 Planned

- Futures strategy module
- Bond strategy module
- Loan strategy module
- Advanced optimization algorithms

---

## See Also

- **[Box Spread Strategy](../strategies/box-spread/README.md)** - Active strategy component
- **[Main README](../research/architecture/../../../README.md)** - Project overview and getting started
- **[Project Status](../research/architecture/../../PROJECT_STATUS.md)** - Implementation status
- **[Project Rename Analysis](../research/architecture/../../PROJECT_RENAME_AND_SPLIT_ANALYSIS.md)** - Rename and split strategy

---

**Last Updated**: 2025-01-27
**Maintained By**: Synthetic Financing Platform Team
