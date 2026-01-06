# Implementation Plan from Research (Option A)

**Date**: 2025-12-30
**Status**: Ready for Implementation
**Source**: 10 Completed Research Tasks (T-142 through T-151)

---

## Executive Summary

**Research Complete**: All 10 research tasks (T-142 through T-151) have been completed with comprehensive findings.

**Implementation Tasks Ready**: 10 implementation tasks are ready to proceed, with research dependencies satisfied.

**Parallelization Opportunities**: Multiple tasks can be executed in parallel for maximum efficiency.

---

## Research → Implementation Mapping

### Broker Integration (3 tasks)

| Research Task | Status | Implementation Task | Dependencies | Ready? |
|--------------|--------|---------------------|--------------|--------|
| **T-142** | ✅ Done | **T-35**: Implement Alpaca API adapter | T-32, T-34, T-142 | ✅ Yes |
| **T-143** | ✅ Done | **T-36**: Implement IB Client Portal adapter | T-33, T-34, T-143 | ✅ Yes |
| **T-144** | ✅ Done | **T-37**: Implement broker selection/switching | T-34, T-35, T-36, T-144 | ⏳ After T-35, T-36 |

**Research Findings → Implementation Guidance:**

**T-142 (Alpaca):**

- Use API key authentication (APCA-API-KEY-ID, APCA-API-SECRET-KEY headers)
- Implement rate limit monitoring (200 req/min trading, 10,000 req/min market data)
- Use exponential backoff for 429 errors (Retry-After header)
- Support both paper (`paper-api.alpaca.markets`) and live (`api.alpaca.markets`) endpoints
- Follow existing broker adapter pattern (similar to IBKR Portal client)

**T-143 (IB Client Portal):**

- Use OAuth 1.0a for individual accounts (OAuth 2.0 for institutional)
- Implement automatic session token renewal
- Require Client Portal Gateway for local access
- Follow existing IBKR Portal client patterns (`python/integration/ibkr_portal_client.py`)
- Handle gateway connection errors gracefully

**T-144 (Broker Selection):**

- Implement broker factory pattern for dynamic selection
- Use unified interface (BrokerInterface already exists)
- Normalize data models across brokers
- Implement fallback mechanisms (primary → secondary → tertiary)
- Support performance-based selection (algo wheel pattern)

---

### Data Import (3 tasks)

| Research Task | Status | Implementation Task | Dependencies | Ready? |
|--------------|--------|---------------------|--------------|--------|
| **T-145** | ✅ Done | **T-63**: Implement Excel static file import | T-62, T-145 | ⏳ After T-62 |
| **T-146** | ✅ Done | **T-64**: Implement Excel RTD/DDE connectors | T-62, T-63, T-146 | ⏳ After T-63 |
| **T-147** | ✅ Done | **T-65**: Implement web scraping for Israeli brokers | T-62, T-147 | ⏳ After T-62 |

**Research Findings → Implementation Guidance:**

**T-145 (Excel/CSV Import):**

- Use pandas for Excel/CSV parsing (flexible, widely used)
- Use Pydantic for data validation (type safety, clear errors)
- Implement field mapping configuration (broker-specific formats)
- Handle large files with chunking (pandas chunksize)
- Support .xlsx, .xls, .csv, .tsv formats
- Provide clear error messages for invalid data

**T-146 (RTD/DDE):**

- Use xlwings for Excel RTD access (recommended, cross-platform)
- Use pywin32 for DDE support (Windows-only, legacy systems)
- RTD requires broker-provided RTD server or Excel add-in
- Windows-only limitation must be clearly documented
- Implement polling via named ranges in Excel workbook
- Handle connection failures with retry logic
- Consider alternative: Web scraping or API if RTD/DDE unavailable

**T-147 (Web Scraping):**

- Use Playwright for new projects (faster, more reliable, built-in browsers)
- Use Selenium if existing codebase or specific requirements
- Use BeautifulSoup for HTML parsing after JavaScript execution
- Implement session persistence to avoid repeated logins
- Respect rate limits and broker ToS
- Handle CAPTCHA with manual intervention or services
- Use Page Object Model for maintainability
- Implement retry logic with exponential backoff
- Prefer APIs over scraping when available

---

### Greeks & Risk (2 tasks)

| Research Task | Status | Implementation Task | Dependencies | Ready? |
|--------------|--------|---------------------|--------------|--------|
| **T-148** | ✅ Done | **T-67**: Implement non-option Greeks | T-66, T-148 | ⏳ After T-66 |
| **T-149** | ✅ Done | **T-68**: Implement portfolio Greeks aggregation | T-66, T-67, T-149 | ⏳ After T-67 |

**Research Findings → Implementation Guidance:**

**T-148 (Non-Option Greeks):**

- Stocks/ETFs: Delta = 1.0 (1:1 price movement), Gamma = 0, Vega = 0, Theta = 0
- Bonds: Rho = -Duration × Price (dollar duration), Gamma = Convexity, Delta = 0
- Currency: Delta = FX rate sensitivity, other Greeks = 0
- Cash: All Greeks = 0
- Implement in `native/src/risk_calculator.cpp` (replace stub `calculate_non_option_greeks()`)

**T-149 (Portfolio Aggregation):**

- Portfolio Greek = Σ(Position Greek × Quantity × Multiplier × FX Rate)
- Convert foreign positions to USD before aggregation
- Implement in `native/src/risk_calculator.cpp` (replace stub `calculate_aggregate_greeks()`)
- Use Eigen VectorXd for aggregation (already in codebase)
- Set risk limits based on aggregated Greeks
- Use Greeks for rebalancing triggers

---

### Cash Flow (2 tasks)

| Research Task | Status | Implementation Task | Dependencies | Ready? |
|--------------|--------|---------------------|--------------|--------|
| **T-150** | ✅ Done | **T-70**: Implement cash flow calculations | T-69, T-150 | ⏳ After T-69 |
| **T-151** | ✅ Done | **T-71**: Integrate cash flow with backend/strategy | T-69, T-70, T-151 | ⏳ After T-70 |

**Research Findings → Implementation Guidance:**

**T-150 (Cash Flow Calculation):**

- Loan payments: Principal + Interest, SHIR-based (variable), CPI-linked (fixed + CPI adjustment)
- Option expiration: Intrinsic value at expiration (ITM = cash flow, OTM = 0), box spreads guarantee payout
- Bond coupons: Periodic payments (semi-annual most common), principal at maturity
- Dividends: Use ORATS API for dividend schedules (already integrated)
- Create CashFlowCalculator class for all cash flow types
- Generate timeline sorted by date, aggregated by period
- Convert to USD for unified view
- Calculate cumulative balance for liquidity planning

**T-151 (Cash Flow Integration):**

- Add `cash_flow_timeline` field to SystemSnapshot
- Generate timeline sorted by date, aggregated by period
- Cumulative balance projection for liquidity planning
- Cash flow-based rebalancing triggers
- API endpoints: GET /cash-flows, GET /cash-flow-timeline
- Integrate with PortfolioAllocationManager
- Multi-currency, multi-account cash flow aggregation

---

## Implementation Execution Plan

### Phase 1: Broker Integration (Can Start Now)

**Parallel Execution:**

- **T-35**: Implement Alpaca API adapter (8-12 hours)
- **T-36**: Implement IB Client Portal adapter (8-12 hours)

**Sequential:**

- **T-37**: Implement broker selection/switching (6-8 hours) - After T-35, T-36

**Total Time**: 14-20 hours (parallel) vs 22-32 hours (sequential)
**Time Savings**: ~35-40%

**Prerequisites:**

- T-32, T-33, T-34 must be complete (check status)
- Research T-142, T-143, T-144 complete ✅

---

### Phase 2: Data Import (After T-62)

**Sequential:**

- **T-63**: Implement Excel static file import (6-8 hours) - After T-62
- **T-64**: Implement Excel RTD/DDE connectors (8-10 hours) - After T-63
- **T-65**: Implement web scraping (8-10 hours) - After T-62 (can parallel with T-64)

**Total Time**: 14-18 hours (T-64 and T-65 parallel) vs 22-28 hours (sequential)
**Time Savings**: ~35-40%

**Prerequisites:**

- T-62 must be complete (check status)
- Research T-145, T-146, T-147 complete ✅

---

### Phase 3: Greeks & Risk (After T-66)

**Sequential:**

- **T-67**: Implement non-option Greeks (6-8 hours) - After T-66
- **T-68**: Implement portfolio Greeks aggregation (8-10 hours) - After T-67

**Total Time**: 14-18 hours (sequential, no parallelization)
**Time Savings**: N/A (sequential dependency)

**Prerequisites:**

- T-66 must be complete (check status)
- Research T-148, T-149 complete ✅

---

### Phase 4: Cash Flow (After T-69)

**Sequential:**

- **T-70**: Implement cash flow calculations (8-10 hours) - After T-69
- **T-71**: Integrate cash flow with backend/strategy (6-8 hours) - After T-70

**Total Time**: 14-18 hours (sequential, no parallelization)
**Time Savings**: N/A (sequential dependency)

**Prerequisites:**

- T-69 must be complete (check status)
- Research T-150, T-151 complete ✅

---

## Immediate Next Steps

### 1. Verify Prerequisites

**Check Status of Foundation Tasks:**

- **T-32, T-33, T-34**: Broker infrastructure tasks (required for T-35, T-36, T-37)
- **T-62**: Position import system design (required for T-63, T-64, T-65)
- **T-66**: Portfolio Greeks calculation system design (required for T-67, T-68)
- **T-69**: Cash flow forecasting system design (required for T-70, T-71)

### 2. Start Broker Integration (If Prerequisites Met)

**Recommended First Steps:**

1. Verify T-32, T-33, T-34 are complete
2. Start T-35 and T-36 in parallel
3. After completion, start T-37

### 3. Continue with Other Phases

**As Prerequisites Complete:**

- Phase 2: After T-62 complete
- Phase 3: After T-66 complete
- Phase 4: After T-69 complete

---

## Implementation Task Details

### T-35: Implement Alpaca API Adapter

**Research-Based Implementation:**

- Use API key authentication (store in environment variables)
- Implement rate limit monitoring via response headers
- Use exponential backoff for 429 errors
- Support both paper and live trading endpoints
- Follow existing broker adapter pattern

**Files to Create/Modify:**

- `native/src/brokers/alpaca_adapter.cpp` (implementation)
- `native/include/brokers/alpaca_adapter.h` (header)
- Configuration: Environment variables for API keys

**Testing:**

- Unit tests for authentication
- Rate limit handling tests
- Error handling tests
- Integration tests with paper trading

---

### T-36: Implement IB Client Portal Adapter

**Research-Based Implementation:**

- Use OAuth 1.0a for individual accounts
- Implement automatic session token renewal
- Require Client Portal Gateway for local access
- Follow existing IBKR Portal client patterns

**Files to Create/Modify:**

- `native/src/brokers/ib_client_portal_adapter.cpp` (implementation)
- `native/include/brokers/ib_client_portal_adapter.h` (header)
- Use existing `python/integration/ibkr_portal_client.py` as reference

**Testing:**

- OAuth flow tests
- Session management tests
- Gateway connection tests
- Error handling tests

---

### T-37: Implement Broker Selection/Switching

**Research-Based Implementation:**

- Implement broker factory pattern
- Use unified interface (BrokerInterface)
- Normalize data models across brokers
- Implement fallback mechanisms

**Files to Create/Modify:**

- `native/src/brokers/broker_factory.cpp` (factory pattern)
- `native/src/brokers/broker_manager.cpp` (selection/switching)
- `native/include/brokers/broker_factory.h`
- `native/include/brokers/broker_manager.h`

**Testing:**

- Factory pattern tests
- Selection logic tests
- Fallback mechanism tests
- Switching tests

---

### T-63: Implement Excel Static File Import

**Research-Based Implementation:**

- Use pandas for Excel/CSV parsing
- Use Pydantic for data validation
- Implement field mapping configuration
- Handle large files with chunking

**Files to Create/Modify:**

- `python/integration/excel_file_importer.py` (new)
- Configuration: Field mapping JSON

**Testing:**

- File format tests (.xlsx, .xls, .csv, .tsv)
- Field mapping tests
- Validation tests
- Large file handling tests

---

### T-64: Implement Excel RTD/DDE Connectors

**Research-Based Implementation:**

- Use xlwings for Excel RTD access
- Use pywin32 for DDE support
- Windows-only limitation documented
- Implement polling via named ranges

**Files to Create/Modify:**

- `python/integration/excel_rtd_client.py` (new)
- `python/integration/excel_dde_client.py` (new)
- Documentation: Windows-only requirements

**Testing:**

- RTD connection tests (Windows only)
- DDE connection tests (Windows only)
- Polling tests
- Error handling tests

---

### T-65: Implement Web Scraping for Israeli Brokers

**Research-Based Implementation:**

- Use Playwright for browser automation
- Use BeautifulSoup for HTML parsing
- Implement session persistence
- Respect rate limits and ToS

**Files to Create/Modify:**

- `python/integration/web_scraper.py` (new)
- Configuration: Broker-specific selectors

**Testing:**

- Login automation tests
- Position extraction tests
- Session management tests
- Error handling tests

---

### T-67: Implement Non-Option Greeks

**Research-Based Implementation:**

- Stocks/ETFs: Delta = 1.0, others = 0
- Bonds: Rho = -Duration × Price, Gamma = Convexity
- Currency: Delta = FX sensitivity
- Cash: All Greeks = 0

**Files to Modify:**

- `native/src/risk_calculator.cpp` (replace stub `calculate_non_option_greeks()`)

**Testing:**

- Stock Greeks tests
- Bond Greeks tests
- Currency Greeks tests
- Edge case tests

---

### T-68: Implement Portfolio Greeks Aggregation

**Research-Based Implementation:**

- Portfolio Greek = Σ(Position Greek × Quantity × Multiplier × FX Rate)
- Convert foreign positions to USD
- Use Eigen VectorXd for aggregation

**Files to Modify:**

- `native/src/risk_calculator.cpp` (replace stub `calculate_aggregate_greeks()`)

**Testing:**

- Aggregation tests
- Currency conversion tests
- Multi-asset tests
- Edge case tests

---

### T-70: Implement Cash Flow Calculations

**Research-Based Implementation:**

- Loan payments: SHIR-based, CPI-linked
- Option expiration: Intrinsic value
- Bond coupons: Periodic payments
- Dividends: ORATS API integration

**Files to Create/Modify:**

- `python/integration/cash_flow_calculator.py` (new)
- Integration with existing loan/option/bond systems

**Testing:**

- Loan payment tests
- Option expiration tests
- Bond coupon tests
- Dividend tests
- Timeline generation tests

---

### T-71: Integrate Cash Flow with Backend/Strategy

**Research-Based Implementation:**

- Add `cash_flow_timeline` to SystemSnapshot
- Generate timeline sorted by date
- Cumulative balance projection
- API endpoints for cash flow data

**Files to Create/Modify:**

- `agents/backend/crates/api/src/state.rs` (add cash_flow_timeline)
- `agents/backend/crates/api/src/routes.rs` (add endpoints)
- Integration with PortfolioAllocationManager

**Testing:**

- Timeline generation tests
- API endpoint tests
- Integration tests
- Performance tests

---

## Parallelization Summary

| Phase | Tasks | Parallel? | Time (Parallel) | Time (Sequential) | Savings |
|-------|-------|-----------|-----------------|-------------------|---------|
| **Phase 1** | T-35, T-36, T-37 | T-35∥T-36 → T-37 | 14-20 hours | 22-32 hours | 35-40% |
| **Phase 2** | T-63, T-64, T-65 | T-63 → T-64∥T-65 | 14-18 hours | 22-28 hours | 35-40% |
| **Phase 3** | T-67, T-68 | Sequential | 14-18 hours | 14-18 hours | 0% |
| **Phase 4** | T-70, T-71 | Sequential | 14-18 hours | 14-18 hours | 0% |

**Total Time Savings**: ~20-30 hours (35-40% reduction in Phases 1 & 2)

---

## Risk Assessment

### High-Risk Areas

1. **Windows-Only Dependencies (T-64)**:
   - RTD/DDE require Windows COM automation
   - **Mitigation**: Document clearly, provide alternative (web scraping or API)

2. **Web Scraping Legal Issues (T-65)**:
   - Must respect broker ToS and rate limits
   - **Mitigation**: Prefer APIs when available, implement rate limiting, respect robots.txt

3. **Broker API Changes**:
   - Alpaca and IB Client Portal APIs may change
   - **Mitigation**: Use official SDKs when possible, implement version checking

### Medium-Risk Areas

1. **Currency Conversion Accuracy (T-68)**:
   - FX rates must be accurate for portfolio aggregation
   - **Mitigation**: Use reliable FX rate source, cache rates appropriately

2. **Cash Flow Calculation Complexity (T-70)**:
   - Multiple asset types with different calculation methods
   - **Mitigation**: Comprehensive testing, use existing design documents

---

## Success Criteria

### Phase 1: Broker Integration

- ✅ Both adapters functional and tested
- ✅ Broker switching working
- ✅ Unified interface operational
- ✅ Rate limiting and error handling robust

### Phase 2: Data Import

- ✅ All three import methods functional
- ✅ Data validation working
- ✅ Integration with portfolio system
- ✅ Windows-only limitations documented

### Phase 3: Greeks & Risk

- ✅ Greeks calculated for all asset types
- ✅ Portfolio-level aggregation working
- ✅ Risk metrics functional
- ✅ Currency conversion accurate

### Phase 4: Cash Flow

- ✅ All cash flow types calculated
- ✅ Backend integration complete
- ✅ Strategy integration working
- ✅ Timeline generation accurate

---

## Next Actions

1. **Verify Prerequisites**: Check status of T-32, T-33, T-34, T-62, T-66, T-69
2. **Start Phase 1**: Begin T-35 and T-36 in parallel (if prerequisites met)
3. **Monitor Progress**: Track implementation against research findings
4. **Update Dependencies**: Ensure all task dependencies are correctly set

---

**Last Updated**: 2025-12-30
**Status**: Ready for Implementation
**Research Complete**: ✅ All 10 research tasks (T-142 through T-151)
