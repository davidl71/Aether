# Financial Data Sources and Databases Research

**Date**: 2025-01-27
**Sources**: Multiple financial data APIs and time-series databases
**Purpose**: Comprehensive analysis of financial data sources and databases for box spread arbitrage system

---

## **MANDATORY RESEARCH COMPLETED** ✅

### **Local Codebase Analysis:**

**Existing Data Infrastructure:**

1. **Time-Series Database**: QuestDB (partially integrated)
   - Location: `python/integration/questdb_client.py`
   - Protocol: InfluxDB Line Protocol (ILP) on port 9009
   - Tables: `quotes`, `trades`
   - Status: Currently used for historical market data only

2. **Market Data Providers**:
   - **dxFeed**: C++ APIs, FIX protocol, options analytics (primary)
   - **ORATS**: REST API, extensive options analytics
   - **Alpha Vantage**: REST API, free tier (5 calls/min)
   - **Finnhub**: REST/WebSocket API, free tier (60 calls/min)
   - **Massive.com**: Historical data, S3-compatible interface

3. **Storage Architecture** (from `BACKEND_DATA_STORAGE_ARCHITECTURE.md`):
   - **Layer 1**: In-memory cache (Rust `Arc<RwLock<SystemSnapshot>>`)
   - **Layer 2**: Operational database (PostgreSQL/SQLite proposed)
   - **Layer 3**: Time-series database (QuestDB)
   - **Layer 4**: Configuration storage (TOML files)

**Code References:**

```21:27:docs/research/architecture/BACKEND_DATA_STORAGE_ARCHITECTURE.md
2. **QuestDB (Time-Series Database):**
   - **Status:** Partially integrated
   - **Purpose:** Historical market data (quotes, trades)
   - **Protocol:** InfluxDB Line Protocol (ILP) on port 9009
   - **Client:** `python/integration/questdb_client.py`
   - **Tables:** `quotes`, `trades`
   - **Limitation:** Not yet used for positions, orders, or portfolio data
```

---

### **Internet Research (2025):**

🔗 **[ClickHouse Quick Start Guide](https://clickhouse.com/docs/get-started/quick-start)**

- **Found via web search:** ClickHouse official documentation for quick start
- **Key Insights:**
  - High-performance, open-source columnar database for real-time analytical processing
  - Offers both cloud and open-source solutions
  - Supports multiple data ingestion methods: CDC, SQL console, local client, file upload
  - Optimized for analytical queries on large datasets

- **Applicable to Task:**
  - Potential alternative to QuestDB for time-series analytics
  - Better suited for analytical workloads vs. real-time streaming
  - Cloud option available (ClickHouse Cloud on AWS/GCP/Azure)

🔗 **[Financial Modeling Prep (FMP) API Documentation](https://site.financialmodelingprep.com/developer/docs)**

- **Found via web search:** FMP comprehensive API documentation
- **Key Insights:**
  - **Comprehensive Coverage**: Over 100 API endpoints
  - **Data Types**: Real-time stock prices, historical data, financial statements (income, balance sheet, cash flow), earnings transcripts, analyst estimates, price targets
  - **Features**:
    - Stock symbol search across global markets
    - Bulk data APIs for efficient retrieval
    - DCF valuations, financial scores, ratings
    - ETF holdings, insider trades, Form 13F
    - Options data available
  - **Auth**: API key required (`?apikey=YOUR_API_KEY`)
  - **Free Tier**: Limited (requires API key registration)

- **Applicable to Task:**
  - **Complementary to existing providers**: More comprehensive than Alpha Vantage/Finnhub
  - **Financial statements**: Detailed financial data for fundamental analysis
  - **Bulk APIs**: Efficient for batch operations
  - **Options data**: Supports box spread strategy analysis
  - **Use Case**: Cross-validation with TWS API data, fundamental analysis, research

🔗 **[Daloopa: Pros & Cons of Open Source Financial Data](https://daloopa.com/blog/analyst-best-practices/pros-cons-open-source-financial-data-for-analysis)**

- **Found via web search:** Daloopa blog on open-source financial data analysis
- **Key Insights:**
  - **Pros of Open Source Data**:
    - Accessibility and cost-effectiveness
    - Community-driven improvements
    - Transparency and auditability
    - Flexibility for customization
  - **Cons of Open Source Data**:
    - Data quality and reliability concerns
    - Maintenance and security challenges
    - Limited support compared to commercial providers
    - Potential gaps in coverage
  - **Best Practices**: Validate data quality, implement data governance, use multiple sources for cross-validation

- **Applicable to Task:**
  - **Strategy**: Use open-source data for research/backtesting, commercial data for live trading
  - **Validation**: Cross-validate data from multiple sources (TWS + FMP + Alpha Vantage)
  - **Risk Management**: Understand limitations of open-source financial data

🔗 **[QuestDB Market Data Features](https://questdb.com/market-data/)**

- **Found via web search:** QuestDB specialized features for capital markets
- **Key Insights:**
  - **Capital Markets Optimization**: Purpose-built for financial data
  - **Features**:
    - Real-time data streaming (InfluxDB Line Protocol, PostgreSQL wire protocol)
    - Efficient storage for tick data
    - Pre- and post-trade analysis capabilities
    - Backtesting support
    - Exchange surveillance features
  - **Performance**: Optimized for rapid data ingestion and near-instant queries
  - **Extended SQL**: Time-series SQL extensions for financial queries

- **Applicable to Task:**
  - **Current Integration**: Already partially integrated (`python/integration/questdb_client.py`)
  - **Strengths**: Optimized for capital markets use case
  - **Recommendation**: Continue using QuestDB for time-series market data storage

🔗 **[QuestDB Documentation](https://questdb.com/docs/)**

- **Found via web search:** QuestDB comprehensive documentation
- **Key Insights:**
  - **Installation**: Multiple deployment options (Docker, Homebrew, binaries)
  - **Protocols**: InfluxDB Line Protocol (ILP), PostgreSQL wire protocol, HTTP REST API
  - **SQL Extensions**: Time-series specific SQL features (SAMPLE BY, LATEST ON, ASOF JOIN)
  - **Performance**: Sub-millisecond queries on billions of rows
  - **Integration**: Python, Java, Node.js, Go clients available

- **Applicable to Task:**
  - **Enhancement Opportunity**: Expand QuestDB usage beyond quotes/trades
  - **Potential Use**: Store positions history, orders history, Greeks snapshots
  - **Integration**: Already using ILP protocol, could leverage PostgreSQL protocol for easier Rust integration

🔗 **[FinanceDatabase by JerBouma (GitHub)](https://github.com/JerBouma/FinanceDatabase)**

- **Found via web search:** Python library providing comprehensive financial instrument database
- **Key Insights:**
  - **Scope**: Over 300,000 financial symbols
  - **Coverage**:
    - Equities (stocks)
    - ETFs
    - Funds (mutual funds)
    - Indices
    - Currencies
    - Cryptocurrencies
    - Money markets
  - **Purpose**: Research, backtesting, symbol lookup
  - **Format**: Python library, structured data access
  - **License**: Open-source (check specific license)

- **Applicable to Task:**
  - **Symbol Lookup**: Comprehensive symbol database for research
  - **Backtesting**: Historical symbol reference for backtesting strategies
  - **Research**: Valuable for exploring available instruments
  - **Limitation**: Static reference data, not real-time market data

🔗 **[FinanceToolkit by JerBouma (GitHub)](https://github.com/JerBouma/FinanceToolkit)**

- **Found via web search:** Open-source Python toolkit for transparent financial analysis
- **Key Insights:**
  - **Comprehensive Metrics**: 150+ financial ratios, indicators, and performance measurements
  - **Transparency**: All calculation methods are open-source and documented
  - **Data Provider**: Uses Financial Modeling Prep (FMP) API as primary data source
  - **Coverage**:
    - Financial ratios (liquidity, profitability, efficiency, leverage, etc.)
    - Risk metrics (Sharpe, Sortino, Calmar, Information Ratio, Max Drawdown)
    - Technical indicators (RSI, MACD, Bollinger Bands, etc.)
    - Performance metrics (ROI, ROE, ROA, etc.)
    - Fundamental analysis (income statements, balance sheets, cash flow)
  - **Features**:
    - Multiple asset classes: Equities, options, currencies, crypto, ETFs, indices
    - Quarterly and annual financial statements
    - TTM (Trailing Twelve Months) calculations
    - Growth metrics with lag options
    - Caching support (`use_cached_data=True`)
    - Multi-ticker analysis
  - **Complementary to FinanceDatabase**: Works together for comprehensive analysis

- **Applicable to Task:**
  - **Risk Metrics Cross-Validation**: Project already has C++ risk calculator (Sharpe, Sortino, Calmar, Information Ratio) - FinanceToolkit can validate calculations
  - **Fundamental Analysis**: Analyze underlying securities for box spread opportunities
  - **Performance Analysis**: Calculate portfolio-level risk-adjusted returns
  - **Research & Backtesting**: Comprehensive financial analysis for strategy development
  - **Transparent Calculations**: All methods are documented - useful for understanding methodology

- **Integration Priority**: Medium (complementary to existing risk calculator)

---

### **Library Assessment:**

**1. ClickHouse vs QuestDB vs PostgreSQL (TimescaleDB)**

| Feature | ClickHouse | QuestDB | PostgreSQL + TimescaleDB |
|---------|-----------|---------|-------------------------|
| **Use Case** | Analytical workloads | Real-time time-series | Operational + time-series hybrid |
| **Performance** | Very fast analytical queries | Optimized for tick data | Good for mixed workloads |
| **Integration** | Multiple protocols | ILP, PostgreSQL wire | PostgreSQL native |
| **Capital Markets** | General-purpose | Purpose-built | General-purpose with extension |
| **Recommendation** | ❌ Not needed (overlap with QuestDB) | ✅ Continue using | ✅ For Layer 2 (operational DB) |

**2. Financial Modeling Prep vs Existing Providers**

| Provider | Free Tier | Options Data | Financial Statements | Bulk APIs | Best For |
|----------|-----------|--------------|---------------------|-----------|----------|
| **FMP** | Limited | ✅ Yes | ✅ Comprehensive | ✅ Yes | Financial analysis, research |
| **Alpha Vantage** | 5 calls/min | ⚠️ Basic | ⚠️ Limited | ❌ No | Technical indicators |
| **Finnhub** | 60 calls/min | ⚠️ Basic | ⚠️ Limited | ❌ No | Real-time quotes, sentiment |
| **dxFeed** | Paid | ✅ Full | ❌ No | ✅ Yes | Live trading, C++ integration |
| **ORATS** | Paid | ✅ Extensive | ❌ No | ⚠️ Limited | Options analytics |

**Recommendation**: **FMP as complementary data source** for:

- Financial statement analysis
- Cross-validation with TWS data
- Research and backtesting
- Fundamental analysis

---

### **Synthesis & Recommendation:**

#### **Database Strategy:**

1. **Continue Using QuestDB** (Time-Series Database - Layer 3)
   - **Rationale**: Already integrated, purpose-built for capital markets
   - **Action**: Expand usage to positions history, orders history, Greeks snapshots
   - **No Change**: Keep QuestDB for market data time-series storage

2. **ClickHouse**: **Not Recommended**
   - **Rationale**: Overlaps with QuestDB functionality
   - **QuestDB is better suited** for real-time market data
   - **ClickHouse better for** analytical workloads (already have QuestDB)

3. **PostgreSQL/SQLite** (Operational Database - Layer 2)
   - **Rationale**: Already proposed in architecture
   - **Action**: Proceed with PostgreSQL for structured data (positions, orders, portfolio)

#### **Market Data Provider Strategy:**

1. **Add Financial Modeling Prep (FMP)** as **Tier 2 Data Source**
   - **Purpose**: Financial statements, fundamental analysis, cross-validation
   - **Use Cases**:
     - Research and backtesting
     - Financial statement analysis
     - Cross-validation with TWS API data
     - Bulk data retrieval for analysis
   - **Integration Priority**: Medium (complementary, not critical)

2. **Keep Existing Providers**:
   - **dxFeed**: Primary for live trading (C++ APIs, options analytics)
   - **ORATS**: Secondary for options-specific analytics
   - **Alpha Vantage/Finnhub**: Free tier for validation
   - **TWS API**: Primary broker data source

3. **FinanceDatabase**: **Reference Data Only**
   - **Purpose**: Symbol lookup, research, backtesting reference
   - **Integration**: Minimal (Python library for research tools)
   - **Not for**: Real-time trading or market data

4. **FinanceToolkit**: **Financial Analysis & Risk Metrics** ✅ **NEW**
   - **Purpose**: Comprehensive financial ratios, risk metrics, fundamental analysis
   - **Features**: 150+ financial ratios, technical indicators, risk metrics (Sharpe, Sortino, Calmar, etc.)
   - **Integration**: Python library using FMP API
   - **Use Cases**:
     - Fundamental analysis of underlying securities
     - Cross-validation of risk metrics (complements existing C++ risk calculator)
     - Performance analysis and backtesting
     - Research and strategy development
   - **Integration Priority**: Medium (complementary to existing risk calculator)

#### **Open Source Data Strategy:**

**From Daloopa Analysis:**

- **Use Open Source for**: Research, backtesting, validation
- **Use Commercial for**: Live trading, critical decisions
- **Cross-Validation**: Always validate open-source data against commercial sources
- **Risk Management**: Understand limitations and implement data quality checks

---

## Integration Recommendations

### **Priority 1: Expand QuestDB Usage** (High Impact, Low Effort)

**Current State**: QuestDB only stores `quotes` and `trades`

**Recommended Expansion**:

```python

# Additional QuestDB tables to add:
# 1. positions_history - Time-series position snapshots
# 2. orders_history - Time-series order status updates
# 3. greeks_snapshots - Portfolio Greeks over time
# 4. market_data_snapshots - Consolidated market data snapshots
```

**Benefits**:

- Historical analysis of positions, orders, Greeks
- Real-time monitoring and alerting
- Backtesting capabilities
- Leverages existing infrastructure

### **Priority 2: Add FMP API Integration** (Medium Impact, Medium Effort)

**Integration Points**:

1. **Financial Statements Module**:
   - Income statements, balance sheets, cash flow statements
   - Use for fundamental analysis of underlying securities

2. **Cross-Validation Service**:
   - Compare TWS API data with FMP data
   - Identify discrepancies or data quality issues

3. **Research Tools**:
   - Bulk data retrieval for analysis
   - Historical financial data for backtesting

**Implementation**:

```python

# New module: python/integration/fmp_client.py
# - Financial statements API
# - Bulk data APIs
# - Cross-validation utilities
```

### **Priority 3: FinanceDatabase Integration** (Low Impact, Low Effort)

**Integration Points**:

1. **Symbol Lookup Tool**:
   - Comprehensive symbol search
   - Research and backtesting reference

2. **Market Data Research**:
   - Explore available instruments
   - Historical symbol reference

**Implementation**:

```python

# Add to research tools: python/research/symbol_lookup.py
# - Use FinanceDatabase for symbol exploration
# - Reference data for backtesting
```

### **Priority 4: FinanceToolkit Integration** (Medium Impact, Low Effort)

**Integration Points**:

1. **Risk Metrics Cross-Validation**:
   - Project already has C++ risk calculator (`native/src/risk_calculator.cpp`)
   - Existing metrics: Sharpe, Sortino, Calmar, Information Ratio, Max Drawdown
   - FinanceToolkit provides same metrics - use for validation
   - Compare results to ensure calculation accuracy

2. **Fundamental Analysis Module**:
   - Analyze underlying securities for box spread opportunities
   - Financial ratios (liquidity, profitability, efficiency, leverage)
   - Use for screening securities before box spread analysis

3. **Performance Analysis**:
   - Portfolio-level risk-adjusted returns
   - Calculate TTM metrics for performance tracking
   - Growth metrics with lag options

4. **Research Tools**:
   - Comprehensive financial analysis for strategy development
   - Backtesting support with cached data
   - Multi-ticker analysis for sector/industry comparisons

**Implementation**:

```python

# New module: python/integration/finance_toolkit_client.py
# - Risk metrics validation against C++ calculator
# - Fundamental analysis for underlying securities
# - Performance analysis for portfolio tracking

# Example usage:

from financetoolkit import Toolkit

# Initialize with FMP API key (shared with FMP client)

toolkit = Toolkit(
    tickers=['SPY', 'QQQ'],  # Underlying securities
    api_key="YOUR_FMP_API_KEY",
    quarterly=True,
    start_date="2020-01-01"
)

# Cross-validate risk metrics

sharpe_ratio = toolkit.ratios.get_sharpe_ratio(trailing=252)
sortino_ratio = toolkit.ratios.get_sortino_ratio(trailing=252)

# Compare with C++ calculator results for validation
```

**Benefits**:

- **Transparency**: All calculation methods are documented (open-source)
- **Comprehensive**: 150+ ratios vs. ~10 in existing risk calculator
- **Validation**: Cross-validate existing C++ risk calculations
- **Fundamental Analysis**: Analyze underlying securities (beyond options pricing)
- **Research**: Extensive financial analysis capabilities

---

## Summary

| Resource | Recommendation | Priority | Integration Effort |
|----------|---------------|----------|-------------------|
| **QuestDB** | ✅ Continue & Expand | High | Low (already integrated) |
| **ClickHouse** | ❌ Not Needed | N/A | N/A |
| **FMP API** | ✅ Add as Tier 2 Source | Medium | Medium |
| **FinanceDatabase** | ✅ Add to Research Tools | Low | Low |
| **FinanceToolkit** | ✅ Add for Risk Validation & Analysis | Medium | Low |
| **Daloopa Insights** | ✅ Adopt Strategy | Medium | Low (documentation) |

**Key Decision**: **Do not replace QuestDB with ClickHouse** - QuestDB is better suited for real-time market data. Instead, **expand QuestDB usage** and **add FMP API** for complementary data needs.

---

## Next Steps

1. **Expand QuestDB Schema**: Add tables for positions_history, orders_history, greeks_snapshots
2. **Integrate FMP API**: Create client module for financial statements and bulk data
3. **Add FinanceDatabase**: Include in research tools for symbol lookup
4. **Add FinanceToolkit**: Create client module for risk metrics validation and fundamental analysis
5. **Document Strategy**: Update architecture docs with multi-source data validation approach

---

## References

- [ClickHouse Quick Start](https://clickhouse.com/docs/get-started/quick-start)
- [Financial Modeling Prep API Docs](https://site.financialmodelingprep.com/developer/docs)
- [QuestDB Market Data](https://questdb.com/market-data/)
- [QuestDB Documentation](https://questdb.com/docs/)
- [FinanceDatabase GitHub](https://github.com/JerBouma/FinanceDatabase)
- Existing: `docs/research/architecture/BACKEND_DATA_STORAGE_ARCHITECTURE.md`
- Existing: `docs/indices/MARKET_DATA_INDEX.md`
