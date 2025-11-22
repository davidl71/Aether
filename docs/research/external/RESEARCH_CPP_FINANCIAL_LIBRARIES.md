# C++ Financial Libraries Research

**Date:** 2025-11-18
**Status:** Research Complete
**Project:** IBKR Box Spread Trading Application

## Executive Summary

This document analyzes 10 C++ financial software resources for potential integration with the IBKR box spread trading application. The analysis focuses on relevance to box spread trading, compatibility with C++20, integration complexity, and alignment with existing project architecture.

**Key Findings:**

- **QuantLib** and **Eigen** are highest priority for integration
- **Option Pricer (GitHub)** provides lighter-weight alternative to QuantLib
- **NLopt** useful for convexity optimization in portfolio allocation
- FIX engines and charting libraries are low priority for current CLI/TUI focus
- Most libraries are C++17+ compatible, suitable for C++20 project

---

## Resource Analysis

### 1. QuantLib (<https://www.quantlib.org/>)

**Type:** Open-source C++ library
**License:** BSD 3-Clause
**Status:** Actively maintained (2025)

**Description:**
QuantLib is the industry-standard C++ library for quantitative finance, providing comprehensive tools for derivatives pricing, risk management, portfolio optimization, and yield curve modeling.

**Key Features:**

- Black-Scholes, binomial trees, Monte Carlo methods
- Greeks calculations (Delta, Gamma, Vega, Theta, Rho)
- Volatility surfaces and term structure modeling
- Yield curve construction and interpolation
- Portfolio optimization and risk metrics
- Calendar and day-count conventions

**Relevance to Box Spread Trading:**

- ✅ **High:** Could replace or enhance existing option pricing logic
- ✅ **Greeks Calculations:** Comprehensive Greeks for portfolio risk management
- ✅ **Volatility Modeling:** Useful for volatility skew analysis in investment strategy
- ✅ **Yield Curves:** Supports T-bill rate calculations and cash allocation decisions

**Technical Compatibility:**

- **C++ Standard:** C++17+ (compatible with C++20)
- **CMake Support:** Yes, well-integrated with CMake
- **Dependencies:** Boost (date_time, filesystem, system), may require additional math libraries
- **Build Complexity:** Moderate (large library, but well-documented build process)

**Integration Considerations:**

- **Pros:**
  - Industry standard, widely used and tested
  - Comprehensive documentation and examples
  - Active community and maintenance
  - BSD license (permissive)
- **Cons:**
  - Large library size (may increase build time)
  - Boost dependency (adds complexity)
  - May be overkill if only need basic option pricing

**Use Cases in Project:**

1. Enhanced option pricing for box spreads (replace current Black-Scholes implementation)
2. Portfolio Greeks aggregation (see `docs/PORTFOLIO_GREEKS_SYSTEM.md`)
3. Volatility surface modeling for skew analysis
4. Yield curve construction for T-bill rate comparisons
5. Risk metrics calculation for portfolio management

**Recommendation:** **HIGH PRIORITY** - Integrate for enhanced option pricing and Greeks calculations

---

### 2. Option Pricer (GitHub: anthonymakarewicz/option-pricer)

**Type:** Open-source C++ library
**License:** Need to verify (check repository)
**Status:** Need to verify activity (2025)

**Description:**
High-performance C++ library specifically designed for option pricing, implementing European, American, and exotic options with efficient algorithms.

**Key Features:**

- Black-Scholes implementation
- Binomial tree methods
- Monte Carlo simulation
- American option pricing
- Exotic options support

**Relevance to Box Spread Trading:**

- ✅ **High:** Directly applicable to box spread option pricing
- ✅ **Performance:** Optimized for real-time trading systems
- ✅ **Simplicity:** Lighter weight than QuantLib

**Technical Compatibility:**

- **C++ Standard:** Need to verify (likely C++17+)
- **CMake Support:** Need to verify repository structure
- **Dependencies:** Likely minimal (verify in repository)
- **Build Complexity:** Likely low (smaller library)

**Integration Considerations:**

- **Pros:**
  - Focused on option pricing (no unnecessary features)
  - Likely easier to integrate than QuantLib
  - Performance-optimized
- **Cons:**
  - Less comprehensive than QuantLib
  - May lack advanced features (Greeks, volatility surfaces)
  - Need to verify license compatibility

**Use Cases in Project:**

1. Alternative to QuantLib for basic option pricing
2. Performance-critical pricing calculations
3. Lightweight option pricing module

**Recommendation:** **MEDIUM PRIORITY** - Evaluate as lighter alternative to QuantLib

---

### 3. C++ for Quants (<https://cppforquants.com/>)

**Type:** Educational resource / library overview
**License:** N/A (website)
**Status:** Active (2025)

**Description:**
Comprehensive guide and resource platform for C++ quantitative finance development, covering essential libraries, best practices, and implementation patterns.

**Key Resources:**

- Library overviews (QuantLib, Eigen, Boost, NLopt)
- Tutorials and code examples
- Best practices for quant development
- Performance optimization techniques

**Relevance to Box Spread Trading:**

- ✅ **High:** Educational resource for library selection
- ✅ **Eigen:** Linear algebra for portfolio optimization
- ✅ **NLopt:** Optimization for convexity calculations
- ✅ **Best Practices:** Industry patterns and conventions

**Technical Compatibility:**

- **C++ Standard:** Varies by library (all C++17+ compatible)
- **CMake Support:** Varies by library
- **Dependencies:** Library-specific

**Integration Considerations:**

- **Pros:**
  - Comprehensive library ecosystem overview
  - Practical examples and patterns
  - Industry best practices
- **Cons:**
  - Not a library itself (resource only)
  - Need to integrate individual libraries separately

**Use Cases in Project:**

1. Reference for library selection decisions
2. Implementation patterns for portfolio optimization
3. Best practices for quantitative finance development

**Recommendation:** **REFERENCE ONLY** - Use as guide for library selection, not direct integration

---

### 4. Eigen (Linear Algebra Library)

**Type:** Open-source C++ template library
**License:** MPL2 (Mozilla Public License 2.0)
**Status:** Actively maintained (2025)

**Description:**
High-performance C++ template library for linear algebra, providing efficient matrix and vector operations, numerical solvers, and related algorithms.

**Key Features:**

- Matrix and vector operations
- Linear system solvers
- Eigenvalue/eigenvector calculations
- Sparse matrix support
- Expression templates for optimization

**Relevance to Box Spread Trading:**

- ✅ **High:** Essential for portfolio optimization calculations
- ✅ **Convexity Optimization:** Matrix operations for barbell strategy calculations
- ✅ **Portfolio Allocation:** Linear algebra for allocation algorithms

**Technical Compatibility:**

- **C++ Standard:** C++03+ (fully compatible with C++20)
- **CMake Support:** Yes, excellent CMake integration
- **Dependencies:** None (header-only or minimal)
- **Build Complexity:** Low (header-only option available)

**Integration Considerations:**

- **Pros:**
  - Header-only option (no linking required)
  - Excellent performance (expression templates)
  - Well-documented and widely used
  - MPL2 license (permissive)
- **Cons:**
  - Template-heavy (may increase compile time)
  - Learning curve for advanced features

**Use Cases in Project:**

1. Portfolio allocation matrix calculations (see `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
2. Convexity optimization for barbell strategy
3. Risk metric calculations (covariance matrices, correlation)
4. Optimization algorithms (portfolio rebalancing)

**Recommendation:** **HIGH PRIORITY** - Essential for portfolio optimization features

---

### 5. NLopt (Nonlinear Optimization)

**Type:** Open-source C++ library
**License:** LGPL or MIT (dual license)
**Status:** Actively maintained (2025)

**Description:**
Library for nonlinear optimization, providing a common interface for various optimization algorithms (gradient-based, derivative-free, global optimization).

**Key Features:**

- Multiple optimization algorithms (L-BFGS, Nelder-Mead, etc.)
- Constrained and unconstrained optimization
- Global optimization methods
- Gradient-based and derivative-free options

**Relevance to Box Spread Trading:**

- ✅ **Medium-High:** Useful for convexity optimization
- ✅ **Portfolio Rebalancing:** Optimization algorithms for allocation decisions
- ✅ **Risk Optimization:** Constrained optimization for risk limits

**Technical Compatibility:**

- **C++ Standard:** C++11+ (compatible with C++20)
- **CMake Support:** Yes
- **Dependencies:** Minimal (may require math libraries)
- **Build Complexity:** Low to moderate

**Integration Considerations:**

- **Pros:**
  - Comprehensive optimization algorithms
  - Well-documented
  - Dual license (LGPL or MIT)
- **Cons:**
  - May be overkill for simple optimization tasks
  - Learning curve for algorithm selection

**Use Cases in Project:**

1. Convexity optimization for barbell strategy (see `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
2. Portfolio rebalancing optimization
3. Spare cash allocation optimization
4. Risk-constrained portfolio optimization

**Recommendation:** **MEDIUM PRIORITY** - Useful for advanced optimization features

---

### 6. OnixS C++ FIX Engine (<https://www.onixs.biz/cpp-fix-engine.html>)

**Type:** Commercial C++ library
**License:** Commercial (paid license required)
**Status:** Active (2025)

**Description:**
High-performance C++ implementation of the Financial Information eXchange (FIX) protocol, enabling direct market access and electronic trading.

**Key Features:**

- FIX protocol 4.0-5.0 support
- Low-latency message handling
- Session management
- Market data and order execution
- Multi-threaded architecture

**Relevance to Box Spread Trading:**

- ⚠️ **Low:** TWS API already integrated, FIX engine redundant unless expanding to other brokers
- ⚠️ **Alternative Trading:** Could enable direct market access (bypass TWS API)
- ⚠️ **Latency:** Lower latency than TWS API, but TWS API sufficient for box spreads

**Technical Compatibility:**

- **C++ Standard:** C++11+ (compatible with C++20)
- **CMake Support:** Yes (commercial libraries typically support CMake)
- **Dependencies:** Platform-specific (verify)
- **Build Complexity:** Low (commercial library, pre-built or well-documented)

**Integration Considerations:**

- **Pros:**
  - Lower latency than TWS API
  - Direct market access
  - Industry-standard protocol
- **Cons:**
  - Commercial license cost
  - Redundant with existing TWS API integration
  - Additional complexity (FIX protocol learning curve)
  - May require broker-specific FIX connectivity setup

**Use Cases in Project:**

1. Alternative to TWS API for direct market access (future consideration)
2. Multi-broker support (if expanding beyond IBKR)
3. High-frequency trading (if latency becomes critical)

**Recommendation:** **LOW PRIORITY** - Only consider if expanding beyond TWS API or requiring lower latency

---

### 7. StockChartX C++ (<https://www.modulusfe.com/products/stock-chart-library/stockchartx-cpp/>)

**Type:** Commercial C++ library
**License:** Commercial (paid license required)
**Status:** Active (2025)

**Description:**
C++ library for creating financial charts and technical analysis visualization, providing chart types, technical indicators, and real-time data display.

**Key Features:**

- Multiple chart types (candlestick, bar, line)
- 80+ technical indicators
- Real-time data updates
- Customizable styling
- Cross-platform support

**Relevance to Box Spread Trading:**

- ⚠️ **Low:** Current project focuses on CLI/TUI, not GUI visualization
- ⚠️ **Future Consideration:** Only if adding GUI or web interface
- ⚠️ **Not Critical:** Technical analysis not core to box spread strategy

**Technical Compatibility:**

- **C++ Standard:** Need to verify (likely C++11+)
- **CMake Support:** Likely yes (commercial library)
- **Dependencies:** Graphics libraries (verify)
- **Build Complexity:** Low to moderate

**Integration Considerations:**

- **Pros:**
  - Comprehensive charting capabilities
  - Real-time updates
  - Professional appearance
- **Cons:**
  - Commercial license cost
  - Not needed for CLI/TUI focus
  - Adds GUI dependencies
  - May require graphics framework (Qt, etc.)

**Use Cases in Project:**

1. Future GUI development (if expanding beyond CLI/TUI)
2. Web interface visualization (if adding web frontend)
3. Technical analysis tools (if adding charting features)

**Recommendation:** **LOW PRIORITY** - Only consider if adding GUI or web interface

---

### 8. Fetching Stock Data in C++ (Medium Article)

**Type:** Educational article / tutorial
**License:** N/A (article)
**Status:** Published (need to verify date)

**Description:**
Tutorial article demonstrating how to fetch fundamental and technical stock data in C++ using APIs (specifically Financial Modeling Prep API).

**Key Topics:**

- HTTP client implementation in C++
- JSON parsing and data extraction
- API authentication patterns
- Data structure design
- Error handling

**Relevance to Box Spread Trading:**

- ✅ **Medium:** Useful patterns for additional data sources
- ✅ **API Integration:** Patterns for integrating external data APIs
- ✅ **Data Fetching:** Techniques for market data retrieval

**Technical Compatibility:**

- **C++ Standard:** Varies by implementation (article examples)
- **CMake Support:** N/A (article, not library)
- **Dependencies:** HTTP client library, JSON parser (nlohmann/json already in project)

**Integration Considerations:**

- **Pros:**
  - Practical implementation patterns
  - Real-world examples
  - API integration best practices
- **Cons:**
  - Not a library (educational resource)
  - May reference outdated patterns
  - Need to adapt to project's existing HTTP client (if any)

**Use Cases in Project:**

1. Reference for integrating additional data sources
2. API integration patterns
3. Data fetching implementation examples

**Recommendation:** **REFERENCE ONLY** - Use as implementation guide, not direct integration

---

### 9. UnoAPI Quantitative Finance Interface (<https://unoapi.org/20-quant-finance/qfi.html>)

**Type:** Modern C++ parallel computing framework
**License:** Need to verify (likely Apache 2.0 or similar)
**Status:** Active (2025)

**Description:**
Modern parallel C++ programming framework for quantitative finance using SYCL/oneAPI, focusing on GPU acceleration and heterogeneous computing.

**Key Features:**

- SYCL/oneAPI for parallel computing
- GPU acceleration for financial calculations
- Heterogeneous computing (CPU + GPU)
- Performance optimization for large-scale calculations

**Relevance to Box Spread Trading:**

- ⚠️ **Low:** Premature optimization for current box spread focus
- ⚠️ **Future Consideration:** Only if performance becomes bottleneck
- ⚠️ **Complexity:** Adds GPU dependencies and SYCL learning curve

**Technical Compatibility:**

- **C++ Standard:** C++17+ (SYCL/oneAPI requirements)
- **CMake Support:** Yes (oneAPI toolkits provide CMake support)
- **Dependencies:** Intel oneAPI toolkit, GPU hardware, SYCL runtime
- **Build Complexity:** High (requires GPU setup, oneAPI installation)

**Integration Considerations:**

- **Pros:**
  - Significant performance gains for large-scale calculations
  - Modern parallel computing approach
  - Industry-leading performance optimization
- **Cons:**
  - High complexity (GPU programming, SYCL)
  - Hardware requirements (GPU)
  - Premature optimization for box spread trading
  - Learning curve for parallel computing

**Use Cases in Project:**

1. Portfolio optimization acceleration (if portfolio becomes very large)
2. Monte Carlo simulation acceleration (if adding advanced pricing)
3. Greeks calculation acceleration (if calculating for many positions)
4. Risk metric calculations (if processing large datasets)

**Recommendation:** **LOW PRIORITY** - Only consider if performance becomes critical bottleneck

---

### 10. Quantum Zeitgeist - C++ Financial Software Article

**Type:** Educational article
**License:** N/A (article)
**Status:** Published 2024

**Description:**
Article discussing C++ advantages in financial software development, covering performance benefits, algorithmic trading strategies, backtesting, and quantitative finance modeling.

**Key Topics:**

- C++ performance advantages in finance
- Multi-core processing benefits
- Static typing advantages
- Industry library references (QuantLib, OpenGamma)
- Algorithmic trading patterns

**Relevance to Box Spread Trading:**

- ✅ **Medium:** Validates C++ choice for trading applications
- ✅ **Best Practices:** Industry patterns and conventions
- ✅ **Context:** Understanding C++ role in finance industry

**Technical Compatibility:**

- **C++ Standard:** N/A (article, not library)
- **CMake Support:** N/A
- **Dependencies:** N/A

**Integration Considerations:**

- **Pros:**
  - Validates project's technology choices
  - Industry context and best practices
  - Performance optimization insights
- **Cons:**
  - Not a library (educational resource)
  - May reference general patterns (not specific implementations)

**Use Cases in Project:**

1. Validation of C++ technology choice
2. Industry best practices reference
3. Performance optimization insights

**Recommendation:** **REFERENCE ONLY** - Use as validation and best practices guide

---

## Integration Status

| Library | Status | Documentation | Integration Date |
|---------|--------|---------------|------------------|
| **Eigen** | ✅ **Integrated** | `docs/EIGEN_INTEGRATION.md` | 2025-11-18 |
| **QuantLib** | 📋 **Documentation Prepared** | `docs/QUANTLIB_INTEGRATION_GUIDE.md` | 2025-11-18 |
| **NLopt** | 📋 **Documentation Prepared** | `docs/NLOPT_INTEGRATION_GUIDE.md` | 2025-11-18 |
| **Option Pricer** | ⏳ **Pending** | - | - |

## Integration Priority Matrix

| Library | Priority | Use Case | Integration Complexity | License | Status |
|---------|----------|----------|----------------------|---------|--------|
| **QuantLib** | **HIGH** | Option pricing, Greeks, risk metrics | Moderate | BSD 3-Clause | 📋 Docs Ready |
| **Eigen** | **HIGH** | Portfolio optimization, linear algebra | Low | MPL2 | ✅ Integrated |
| **Option Pricer** | **MEDIUM** | Lightweight option pricing | Low | Verify | ⏳ Pending |
| **NLopt** | **MEDIUM** | Convexity optimization, portfolio rebalancing | Low-Moderate | LGPL/MIT | 📋 Docs Ready |
| **OnixS FIX** | **LOW** | Alternative to TWS API (future) | Moderate | Commercial |
| **StockChartX** | **LOW** | GUI visualization (future) | Low-Moderate | Commercial |
| **UnoAPI/SYCL** | **LOW** | Performance optimization (future) | High | Verify |
| **C++ for Quants** | **REFERENCE** | Library selection guide | N/A | N/A |
| **Medium Article** | **REFERENCE** | API integration patterns | N/A | N/A |
| **Quantum Zeitgeist** | **REFERENCE** | Best practices validation | N/A | N/A |

---

## Recommended Integration Strategy

### Phase 1: Core Libraries (Immediate)

**1. Eigen (Linear Algebra)**

- **Rationale:** Essential for portfolio optimization calculations
- **Integration:** Header-only option (minimal integration complexity)
- **Use Cases:** Portfolio allocation matrix operations, convexity calculations
- **CMake Integration:** Add via `FetchContent` or `find_package`

**2. QuantLib (Option Pricing & Risk)**

- **Rationale:** Industry standard, comprehensive features
- **Integration:** Add via CMake `FetchContent` or system installation
- **Use Cases:** Enhanced option pricing, Greeks calculations, volatility modeling
- **Dependencies:** Boost (date_time, filesystem, system) - may need to add Boost

### Phase 2: Optimization (Near-term)

**3. NLopt (Optimization)**

- **Rationale:** Useful for convexity optimization and portfolio rebalancing
- **Integration:** Add via CMake `FetchContent` or system installation
- **Use Cases:** Barbell strategy optimization, spare cash allocation optimization
- **Dependencies:** Minimal (verify)

### Phase 3: Alternatives (Evaluate)

**4. Option Pricer (GitHub)**

- **Rationale:** Lighter alternative to QuantLib if QuantLib proves too complex
- **Integration:** Evaluate as fallback option
- **Use Cases:** Basic option pricing if QuantLib integration fails
- **Dependencies:** Verify license and dependencies

### Phase 4: Future Considerations

**5. OnixS FIX Engine**

- **Rationale:** Only if expanding beyond TWS API or requiring lower latency
- **Integration:** Commercial license required, evaluate cost-benefit
- **Use Cases:** Multi-broker support, direct market access

**6. StockChartX**

- **Rationale:** Only if adding GUI or web interface
- **Integration:** Commercial license required, evaluate need
- **Use Cases:** Visualization, technical analysis tools

**7. UnoAPI/SYCL**

- **Rationale:** Only if performance becomes critical bottleneck
- **Integration:** High complexity, requires GPU hardware
- **Use Cases:** Large-scale portfolio optimization, Monte Carlo acceleration

---

## CMake Integration Examples

### Eigen (Header-Only)

```cmake
# Option 1: FetchContent (recommended)
include(FetchContent)
FetchContent_Declare(
    Eigen3
    GIT_REPOSITORY https://gitlab.com/libeigen/eigen.git
    GIT_TAG 3.4.0
)
FetchContent_MakeAvailable(Eigen3)

# Option 2: find_package (if system-installed)
find_package(Eigen3 REQUIRED)

target_link_libraries(ib_box_spread PRIVATE Eigen3::Eigen)
```

### QuantLib

```cmake
# Option 1: FetchContent
include(FetchContent)
FetchContent_Declare(
    QuantLib
    GIT_REPOSITORY https://github.com/lballabio/QuantLib.git
    GIT_TAG QuantLib-v1.32
)
FetchContent_MakeAvailable(QuantLib)

# Option 2: find_package (if system-installed)
find_package(QuantLib REQUIRED)

target_link_libraries(ib_box_spread PRIVATE QuantLib::QuantLib)
```

### NLopt

```cmake
# Option 1: FetchContent
include(FetchContent)
FetchContent_Declare(
    NLopt
    GIT_REPOSITORY https://github.com/stevengj/nlopt.git
    GIT_TAG v2.7.1
)
FetchContent_MakeAvailable(NLopt)

# Option 2: find_package (if system-installed)
find_package(NLopt REQUIRED)

target_link_libraries(ib_box_spread PRIVATE NLopt::nlopt)
```

---

## Compatibility Assessment

### C++20 Compatibility

All recommended libraries are compatible with C++20:

- **QuantLib:** C++17+ (fully compatible)
- **Eigen:** C++03+ (fully compatible)
- **NLopt:** C++11+ (fully compatible)
- **Option Pricer:** Need to verify (likely C++17+)

### Existing Project Dependencies

Current project dependencies:

- TWS API (Protocol Buffers, Intel Decimal, Abseil)
- nlohmann/json
- spdlog
- CLI11
- Catch2

**Compatibility Notes:**

- QuantLib may require Boost (adds dependency, but Boost is widely used)
- Eigen is header-only (no linking conflicts)
- NLopt is standalone (no conflicts)
- All libraries use CMake (compatible with existing build system)

### Build System Integration

Project uses:

- CMake 3.21+
- C++20 standard
- Ninja generator (optional)
- Universal binary builds (macOS)

**Integration Strategy:**

- Use `FetchContent` for automatic dependency management
- Or use system-installed libraries via `find_package`
- Maintain compatibility with existing build presets

---

## License Compatibility

### Project License

Need to verify project license (check LICENSE file or repository).

### Library Licenses

| Library | License | Compatibility Notes |
|---------|---------|---------------------|
| QuantLib | BSD 3-Clause | ✅ Permissive, compatible with any project license |
| Eigen | MPL2 | ✅ Permissive, compatible with most licenses |
| NLopt | LGPL or MIT | ✅ MIT option is permissive; LGPL requires linking considerations |
| Option Pricer | Verify | ⚠️ Need to check repository |
| OnixS FIX | Commercial | ⚠️ Commercial license, evaluate cost |
| StockChartX | Commercial | ⚠️ Commercial license, evaluate cost |

**Recommendation:** QuantLib, Eigen, and NLopt (MIT option) are all permissive licenses compatible with most project licenses.

---

## Performance Considerations

### QuantLib

- **Performance:** Excellent for complex calculations, may be slower for simple operations
- **Memory:** Moderate (large library, but efficient)
- **Compile Time:** May increase due to template usage

### Eigen

- **Performance:** Excellent (expression templates optimize operations)
- **Memory:** Low (header-only, efficient)
- **Compile Time:** May increase due to template-heavy code

### NLopt

- **Performance:** Good (optimized algorithms)
- **Memory:** Low to moderate
- **Compile Time:** Low (C library with C++ wrapper)

### Option Pricer

- **Performance:** Need to verify (likely optimized for speed)
- **Memory:** Likely low (focused library)
- **Compile Time:** Likely low

---

## Risk Assessment

### Integration Risks

**High Risk:**

- QuantLib: Large library, Boost dependency, potential build complexity
- UnoAPI/SYCL: High complexity, GPU requirements, learning curve

**Medium Risk:**

- NLopt: Algorithm selection complexity, may be overkill for simple tasks
- Option Pricer: Need to verify license and maintenance status

**Low Risk:**

- Eigen: Header-only, minimal integration complexity
- Reference resources: No integration risk (educational only)

### Mitigation Strategies

1. **Start with Eigen:** Lowest risk, highest value for portfolio optimization
2. **Evaluate QuantLib:** Test integration in separate branch before committing
3. **Consider Option Pricer:** Evaluate as lighter alternative if QuantLib proves complex
4. **Defer Commercial Libraries:** Only consider if clear business case exists
5. **Performance Testing:** Benchmark library performance before full integration

---

## Next Steps

### Immediate Actions

1. **Evaluate Eigen Integration:**
   - Add Eigen via `FetchContent` in CMakeLists.txt
   - Test with portfolio allocation calculations
   - Verify C++20 compatibility

2. **Research QuantLib Integration:**
   - Check Boost dependency requirements
   - Evaluate build complexity
   - Test option pricing integration

3. **Verify Option Pricer:**
   - Check GitHub repository license
   - Verify maintenance status
   - Evaluate as QuantLib alternative

### Short-term Actions

1. **NLopt Evaluation:**
   - Test optimization algorithms
   - Evaluate for convexity optimization use case
   - Compare with simpler optimization approaches

2. **Documentation Updates:**
   - Update `docs/API_DOCUMENTATION_INDEX.md` with new libraries
   - Document integration patterns
   - Add usage examples

### Long-term Considerations

1. **Commercial Libraries:**
   - Evaluate OnixS FIX if expanding beyond TWS API
   - Consider StockChartX if adding GUI
   - Assess UnoAPI/SYCL if performance becomes critical

---

## References

1. **QuantLib:** <https://www.quantlib.org/>
2. **Eigen:** <https://eigen.tuxfamily.org/>
3. **NLopt:** <https://nlopt.readthedocs.io/>
4. **Option Pricer:** <https://github.com/anthonymakarewicz/option-pricer>
5. **C++ for Quants:** <https://cppforquants.com/>
6. **OnixS FIX Engine:** <https://www.onixs.biz/cpp-fix-engine.html>
7. **StockChartX:** <https://www.modulusfe.com/products/stock-chart-library/stockchartx-cpp/>
8. **UnoAPI QFI:** <https://unoapi.org/20-quant-finance/qfi.html>
9. **Quantum Zeitgeist:** <https://quantumzeitgeist.com/c-financial-software/>
10. **Project Documentation:** `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`, `docs/PORTFOLIO_GREEKS_SYSTEM.md`

---

**Document Status:** Research Complete ✅
**Next Review:** After initial library integration testing
**Maintained By:** Development Team
