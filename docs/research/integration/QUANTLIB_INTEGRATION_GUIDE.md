# QuantLib Integration Guide

**Date:** 2025-11-18
**Status:** Documentation Prepared (Not Yet Integrated)
**Version:** QuantLib Latest Stable

## Overview

QuantLib is an industry-standard open-source C++ library for quantitative finance, providing comprehensive tools for derivatives pricing, risk management, portfolio optimization, and yield curve modeling. This guide provides step-by-step instructions for integrating QuantLib into the IBKR box spread trading application.

## Use Cases in Project

### 1. Enhanced Option Pricing

- Replace or enhance existing Black-Scholes implementation
- Support for European, American, and exotic options
- Volatility surface modeling

### 2. Greeks Calculations

- Comprehensive Greeks (Delta, Gamma, Vega, Theta, Rho)
- Portfolio-level Greeks aggregation
- Risk sensitivity analysis

### 3. Volatility Modeling

- Implied volatility calculations
- Volatility skew analysis
- Volatility surface construction

### 4. Yield Curve Construction

- Risk-free rate estimation for option pricing
- T-bill rate interpolation
- Yield curve construction for cash allocation decisions

### 5. Risk Management

- Value at Risk (VaR) calculations
- Portfolio risk metrics
- Stress testing capabilities

## Prerequisites

### Required Dependencies

1. **Boost Libraries** (Required)
   - `boost::date_time` - Date and time handling
   - `boost::filesystem` - File system operations
   - `boost::system` - System error codes
   - Version: Boost 1.67 or higher

2. **CMake** (Required)
   - Version: 3.15 or higher (project uses 3.21+)

3. **C++ Compiler** (Required)
   - C++17 or higher (project uses C++20)
   - GCC 7+, Clang 5+, or MSVC 2017+

### Installing Boost

#### macOS (Homebrew)

```bash
brew install boost
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get install libboost-all-dev
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install boost-devel
```

#### Manual Installation

If system package manager is not available:

1. Download Boost from <https://www.boost.org/>
2. Extract and build:

```bash
./bootstrap.sh
./b2 --with-date_time --with-filesystem --with-system
sudo ./b2 install
```

## CMake Integration

### Option 1: FetchContent (Recommended)

Add to `native/CMakeLists.txt` in the dependencies section:

```cmake
# Find Boost (required for QuantLib)
find_package(Boost REQUIRED COMPONENTS date_time filesystem system)

# QuantLib via FetchContent
set(QUANTLIB_REPOSITORY "https://github.com/lballabio/quantlib.git")
fetchcontent_declare(
    QuantLib
    GIT_REPOSITORY ${QUANTLIB_REPOSITORY}
    GIT_TAG QuantLib-v1.32  # Use latest stable tag
    GIT_SHALLOW TRUE
)

fetchcontent_makeavailable(QuantLib)
```

### Option 2: System Installation

If QuantLib is system-installed:

```cmake
find_package(Boost REQUIRED COMPONENTS date_time filesystem system)
find_package(QuantLib REQUIRED)
```

### Linking

Add to target link libraries:

```cmake
target_link_libraries(ib_box_spread
    PRIVATE
        ...
        QuantLib::QuantLib
        Boost::date_time
        Boost::filesystem
        Boost::system
        ...
)
```

## Usage Examples

### 1. Option Pricing (Black-Scholes)

```cpp
#include <ql/quantlib.hpp>
#include <ql/instruments/vanillaoption.hpp>
#include <ql/pricingengines/vanilla/analyticeuropeanengine.hpp>
#include <ql/processes/blackscholesmertonprocess.hpp>
#include <ql/termstructures/volatility/equityfx/blackconstantvol.hpp>
#include <ql/termstructures/yield/flatforward.hpp>

using namespace QuantLib;

// Option parameters
Real spot = 100.0;           // Current stock price
Real strike = 105.0;         // Strike price
Rate riskFreeRate = 0.05;    // Risk-free rate (5%)
Volatility volatility = 0.20; // Volatility (20%)
Time maturity = 0.25;        // Time to expiration (3 months)
Option::Type type = Option::Call;

// Create option
Date today = Date::todaysDate();
Date expiration = today + static_cast<int>(maturity * 365);

DayCounter dayCounter = Actual365Fixed();
Calendar calendar = TARGET();

Handle<Quote> spotHandle(ext::shared_ptr<Quote>(new SimpleQuote(spot)));
Handle<YieldTermStructure> rateHandle(
    ext::shared_ptr<YieldTermStructure>(
        new FlatForward(today, riskFreeRate, dayCounter)
    )
);
Handle<BlackVolTermStructure> volHandle(
    ext::shared_ptr<BlackVolTermStructure>(
        new BlackConstantVol(today, calendar, volatility, dayCounter)
    )
);

ext::shared_ptr<BlackScholesMertonProcess> process(
    new BlackScholesMertonProcess(spotHandle, rateHandle, volHandle)
);

ext::shared_ptr<PricingEngine> engine(
    new AnalyticEuropeanEngine(process)
);

ext::shared_ptr<StrikedTypePayoff> payoff(
    new PlainVanillaPayoff(type, strike)
);

ext::shared_ptr<Exercise> exercise(
    new EuropeanExercise(expiration)
);

VanillaOption option(payoff, exercise);
option.setPricingEngine(engine);

// Calculate option price
Real optionPrice = option.NPV();
std::cout << "Option Price: " << optionPrice << std::endl;
```

### 2. Greeks Calculation

```cpp
// After setting up option (see example above)

// Calculate Greeks
Real delta = option.delta();
Real gamma = option.gamma();
Real vega = option.vega();
Real theta = option.theta();
Real rho = option.rho();

std::cout << "Delta: " << delta << std::endl;
std::cout << "Gamma: " << gamma << std::endl;
std::cout << "Vega: " << vega << std::endl;
std::cout << "Theta: " << theta << std::endl;
std::cout << "Rho: " << rho << std::endl;
```

### 3. Implied Volatility

```cpp
#include <ql/instruments/vanillaoption.hpp>
#include <ql/pricingengines/vanilla/analyticeuropeanengine.hpp>
#include <ql/volatility/blackformula.hpp>

// Market option price
Real marketPrice = 5.0;

// Calculate implied volatility
Real impliedVol = 0.0;
try {
    impliedVol = blackFormulaImpliedStdDev(
        Option::Call,
        strike,
        spot,
        marketPrice,
        riskFreeRate,
        maturity
    ) / std::sqrt(maturity);

    std::cout << "Implied Volatility: " << impliedVol << std::endl;
} catch (const std::exception& e) {
    std::cerr << "Error calculating implied volatility: " << e.what() << std::endl;
}
```

### 4. Yield Curve Construction

```cpp
#include <ql/termstructures/yield/flatforward.hpp>
#include <ql/termstructures/yield/interpolatedzerocurve.hpp>

// T-bill rates for yield curve
std::vector<Date> dates = {Date(1, 1, 2025), Date(1, 4, 2025), Date(1, 7, 2025)};
std::vector<Rate> rates = {0.04, 0.045, 0.05}; // 3-month, 6-month, 9-month

DayCounter dayCounter = Actual365Fixed();
Calendar calendar = TARGET();

// Create interpolated yield curve
ext::shared_ptr<YieldTermStructure> yieldCurve(
    new InterpolatedZeroCurve<Linear>(
        dates, rates, dayCounter, calendar
    )
);

// Get risk-free rate for option pricing
Date today = Date::todaysDate();
Rate riskFreeRate = yieldCurve->zeroRate(today + 90, dayCounter, Continuous);
```

### 5. Box Spread Pricing

```cpp
// Box spread = Long call + Short call + Long put + Short put
// All at same expiration, different strikes

// Calculate individual option prices
Real longCallPrice = calculateOptionPrice(spot, strike1, riskFreeRate, volatility, maturity, Option::Call);
Real shortCallPrice = calculateOptionPrice(spot, strike2, riskFreeRate, volatility, maturity, Option::Call);
Real longPutPrice = calculateOptionPrice(spot, strike2, riskFreeRate, volatility, maturity, Option::Put);
Real shortPutPrice = calculateOptionPrice(spot, strike1, riskFreeRate, volatility, maturity, Option::Put);

// Box spread price (net debit for long box, net credit for short box)
Real boxSpreadPrice = longCallPrice - shortCallPrice + longPutPrice - shortPutPrice;

// Box spread value at expiration = strike width
Real strikeWidth = strike2 - strike1;
Real arbitrageProfit = strikeWidth - boxSpreadPrice;
```

## Integration Checklist

### Pre-Integration

- [ ] Install Boost libraries (date_time, filesystem, system)
- [ ] Verify Boost version (1.67+)
- [ ] Verify CMake version (3.15+)
- [ ] Verify C++ compiler (C++17+)

### CMake Integration

- [ ] Add Boost find_package to CMakeLists.txt
- [ ] Add QuantLib FetchContent declaration
- [ ] Add QuantLib to fetchcontent_makeavailable
- [ ] Add QuantLib to target_link_libraries
- [ ] Add Boost components to target_link_libraries

### Testing

- [ ] Create basic option pricing test
- [ ] Test Greeks calculations
- [ ] Verify build succeeds
- [ ] Run integration tests

### Documentation

- [ ] Update API_DOCUMENTATION_INDEX.md
- [ ] Document usage patterns
- [ ] Add code examples

## Build Complexity Considerations

### Compile Time

- QuantLib is a large library and may increase compile time
- Consider using precompiled headers if available
- Use ccache or sccache for faster rebuilds

### Binary Size

- QuantLib adds significant binary size
- Consider static vs. shared library linking
- Use compiler optimization flags

### Dependencies

- Boost adds additional dependencies
- Ensure all Boost components are available
- Consider Boost version compatibility

## License Compatibility

- **QuantLib License:** BSD 3-Clause (permissive)
- **Boost License:** Boost Software License (permissive)
- **Compatibility:** Both licenses are compatible with most project licenses

## Performance Considerations

### Optimization

- QuantLib supports various pricing engines (analytical, numerical, Monte Carlo)
- Choose appropriate engine for use case
- Analytical engines are fastest for simple options

### Memory

- QuantLib uses smart pointers for memory management
- Consider object lifetime and shared ownership
- Use appropriate QuantLib types (Real, Rate, etc.)

## Troubleshooting

### Common Issues

1. **Boost Not Found**
   - Verify Boost installation
   - Set `BOOST_ROOT` environment variable
   - Use `-DBoost_ROOT=/path/to/boost` in CMake

2. **QuantLib Build Fails**
   - Check Boost version compatibility
   - Verify C++ standard (C++17+)
   - Check compiler compatibility

3. **Link Errors**
   - Verify all Boost components are linked
   - Check QuantLib library path
   - Verify namespace targets (QuantLib::QuantLib)

## References

- **QuantLib Website:** <https://www.quantlib.org/>
- **QuantLib GitHub:** <https://github.com/lballabio/quantlib>
- **QuantLib Documentation:** <https://www.quantlib.org/documentation.shtml>
- **Boost Website:** <https://www.boost.org/>
- **Project Research:** `docs/RESEARCH_CPP_FINANCIAL_LIBRARIES.md`

## Next Steps

1. ⏳ Install Boost dependencies
2. ⏳ Integrate QuantLib via CMake
3. ⏳ Create option pricing wrapper
4. ⏳ Implement Greeks calculations
5. ⏳ Integrate with investment strategy framework

---

**Document Status:** Documentation Prepared (Integration Pending)
**Last Updated:** 2025-11-18
