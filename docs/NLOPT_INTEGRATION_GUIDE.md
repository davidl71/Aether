# NLopt Integration Guide

**Date:** 2025-11-18
**Status:** Documentation Prepared (Not Yet Integrated)
**Version:** NLopt Latest Stable

## Overview

NLopt is a library for nonlinear optimization, providing a unified interface for various optimization algorithms. This guide provides step-by-step instructions for integrating NLopt into the IBKR box spread trading application, specifically for convexity optimization and portfolio rebalancing.

## Use Cases in Project

### 1. Convexity Optimization (Barbell Strategy)
- Optimize bond allocation between short-term and long-term maturities
- Maximize portfolio convexity while maintaining duration targets
- Constrained optimization for portfolio allocation

### 2. Portfolio Rebalancing
- Optimize allocation changes to minimize transaction costs
- Rebalance portfolio to target allocations
- Constrained optimization with risk limits

### 3. Spare Cash Allocation
- Optimize allocation between box spreads, T-bills, and bonds
- Maximize yield while maintaining liquidity constraints
- Multi-objective optimization for cash allocation

### 4. Risk-Constrained Optimization
- Optimize portfolio allocation subject to risk limits
- Maximize return while maintaining risk constraints
- Portfolio optimization with Greeks constraints

## Prerequisites

### Required Dependencies

1. **CMake** (Required)
   - Version: 3.15 or higher (project uses 3.21+)

2. **C++ Compiler** (Required)
   - C++11 or higher (project uses C++20)
   - GCC 4.8+, Clang 3.3+, or MSVC 2013+

3. **Math Libraries** (Optional)
   - NLopt can use various math libraries for performance
   - Default implementation works without additional dependencies

### No External Dependencies Required

NLopt is self-contained and does not require Boost or other external libraries (unlike QuantLib).

## CMake Integration

### Option 1: FetchContent (Recommended)

Add to `native/CMakeLists.txt` in the dependencies section:

```cmake
# NLopt via FetchContent
set(NLOPT_REPOSITORY "https://github.com/stevengj/nlopt.git")
fetchcontent_declare(
    NLopt
    GIT_REPOSITORY ${NLOPT_REPOSITORY}
    GIT_TAG v2.7.1  # Use latest stable tag
    GIT_SHALLOW TRUE
)

fetchcontent_makeavailable(NLopt)
```

### Option 2: System Installation

If NLopt is system-installed:

```cmake
find_package(NLopt REQUIRED)
```

### Linking

Add to target link libraries:

```cmake
target_link_libraries(ib_box_spread
    PRIVATE
        ...
        NLopt::nlopt
        ...
)
```

## License Selection

NLopt offers dual licensing:

1. **LGPL License** (Default)
   - Requires linking considerations
   - May require license compatibility review

2. **MIT License** (Alternative)
   - More permissive
   - Recommended for easier integration
   - Set `NLOPT_LICENSE_MIT=ON` in CMake

### Recommended: MIT License

```cmake
set(NLOPT_LICENSE_MIT ON CACHE BOOL "Use MIT license instead of LGPL")
```

## Usage Examples

### 1. Convexity Optimization (Barbell Strategy)

```cpp
#include <nlopt.hpp>
#include <vector>
#include <cmath>

// Objective function: maximize portfolio convexity
double convexity_objective(unsigned n, const double* x, double* grad, void* data)
{
    // x[0] = short-term bond weight
    // x[1] = long-term bond weight

    // Convexity values
    double shortConvexity = 5.0;   // Short-term convexity
    double longConvexity = 150.0;   // Long-term convexity

    // Portfolio convexity = weighted average
    double portfolioConvexity = x[0] * shortConvexity + x[1] * longConvexity;

    // Gradient (if needed)
    if (grad) {
        grad[0] = shortConvexity;
        grad[1] = longConvexity;
    }

    // Minimize negative convexity (maximize convexity)
    return -portfolioConvexity;
}

// Constraint: weights sum to 1.0
double weight_constraint(unsigned n, const double* x, double* grad, void* data)
{
    double sum = x[0] + x[1] - 1.0;

    if (grad) {
        grad[0] = 1.0;
        grad[1] = 1.0;
    }

    return sum;  // Must equal 0
}

// Constraint: duration target
double duration_constraint(unsigned n, const double* x, double* grad, void* data)
{
    double shortDuration = 2.5;   // Short-term duration
    double longDuration = 25.0;    // Long-term duration
    double targetDuration = 10.0;  // Target portfolio duration

    double portfolioDuration = x[0] * shortDuration + x[1] * longDuration;
    double diff = portfolioDuration - targetDuration;

    if (grad) {
        grad[0] = shortDuration;
        grad[1] = longDuration;
    }

    return diff;  // Must equal 0
}

void optimize_barbell_strategy()
{
    // Create optimizer
    nlopt::opt opt(nlopt::LD_LBFGS, 2);  // L-BFGS algorithm, 2 variables

    // Set objective function
    opt.set_min_objective(convexity_objective, nullptr);

    // Set bounds: weights between 0 and 1
    std::vector<double> lb(2, 0.0);  // Lower bounds
    std::vector<double> ub(2, 1.0);  // Upper bounds
    opt.set_lower_bounds(lb);
    opt.set_upper_bounds(ub);

    // Add constraints
    opt.add_equality_constraint(weight_constraint, nullptr, 1e-8);
    opt.add_equality_constraint(duration_constraint, nullptr, 1e-8);

    // Set tolerance
    opt.set_xtol_rel(1e-4);

    // Initial guess: equal weights
    std::vector<double> x(2);
    x[0] = 0.5;  // Short-term weight
    x[1] = 0.5;  // Long-term weight

    // Optimize
    double minf;
    try {
        nlopt::result result = opt.optimize(x, minf);

        if (result < 0) {
            std::cerr << "Optimization failed: " << result << std::endl;
        } else {
            std::cout << "Optimal short-term weight: " << x[0] << std::endl;
            std::cout << "Optimal long-term weight: " << x[1] << std::endl;
            std::cout << "Maximum convexity: " << -minf << std::endl;
        }
    } catch (std::exception& e) {
        std::cerr << "Optimization error: " << e.what() << std::endl;
    }
}
```

### 2. Portfolio Rebalancing Optimization

```cpp
#include <nlopt.hpp>
#include <vector>

// Objective: minimize transaction costs while rebalancing
double rebalance_objective(unsigned n, const double* x, double* grad, void* data)
{
    // x = new allocation weights
    // data = old allocation weights

    double* oldWeights = static_cast<double*>(data);
    double cost = 0.0;

    // Transaction cost = sum of absolute changes
    for (unsigned i = 0; i < n; ++i) {
        double change = std::abs(x[i] - oldWeights[i]);
        cost += change * 0.001;  // 10 bps transaction cost
    }

    if (grad) {
        for (unsigned i = 0; i < n; ++i) {
            grad[i] = (x[i] > oldWeights[i] ? 1.0 : -1.0) * 0.001;
        }
    }

    return cost;
}

// Constraint: weights sum to 1.0
double sum_constraint(unsigned n, const double* x, double* grad, void* data)
{
    double sum = 0.0;
    for (unsigned i = 0; i < n; ++i) {
        sum += x[i];
    }

    if (grad) {
        for (unsigned i = 0; i < n; ++i) {
            grad[i] = 1.0;
        }
    }

    return sum - 1.0;  // Must equal 0
}

void optimize_rebalancing(const std::vector<double>& oldWeights,
                          const std::vector<double>& targetWeights)
{
    unsigned n = oldWeights.size();

    // Create optimizer
    nlopt::opt opt(nlopt::LD_SLSQP, n);  // SLSQP for constrained optimization

    // Set objective function
    opt.set_min_objective(rebalance_objective,
                         const_cast<double*>(oldWeights.data()));

    // Set bounds: weights between 0 and 1
    std::vector<double> lb(n, 0.0);
    std::vector<double> ub(n, 1.0);
    opt.set_lower_bounds(lb);
    opt.set_upper_bounds(ub);

    // Add constraint: weights sum to 1.0
    opt.add_equality_constraint(sum_constraint, nullptr, 1e-8);

    // Initial guess: target weights
    std::vector<double> x = targetWeights;

    // Optimize
    double minf;
    nlopt::result result = opt.optimize(x, minf);

    // Use optimized weights for rebalancing
    // ...
}
```

### 3. Spare Cash Allocation Optimization

```cpp
#include <nlopt.hpp>
#include <vector>

// Objective: maximize yield on spare cash
double yield_objective(unsigned n, const double* x, double* grad, void* data)
{
    // x[0] = box spread allocation
    // x[1] = T-bill allocation
    // x[2] = short bond allocation

    // Yields (annualized)
    double boxSpreadYield = 0.052;   // 5.2%
    double tbillYield = 0.050;       // 5.0%
    double bondYield = 0.048;        // 4.8%

    // Portfolio yield = weighted average
    double portfolioYield = x[0] * boxSpreadYield +
                           x[1] * tbillYield +
                           x[2] * bondYield;

    if (grad) {
        grad[0] = boxSpreadYield;
        grad[1] = tbillYield;
        grad[2] = bondYield;
    }

    // Minimize negative yield (maximize yield)
    return -portfolioYield;
}

// Constraint: liquidity requirement (box spreads less liquid)
double liquidity_constraint(unsigned n, const double* x, double* grad, void* data)
{
    // Maximum 70% in box spreads (liquidity constraint)
    double maxBoxSpread = 0.70;
    double boxSpreadAllocation = x[0];

    if (grad) {
        grad[0] = 1.0;
        grad[1] = 0.0;
        grad[2] = 0.0;
    }

    return boxSpreadAllocation - maxBoxSpread;  // Must be <= 0
}

void optimize_cash_allocation()
{
    unsigned n = 3;  // box spread, T-bill, bond

    // Create optimizer
    nlopt::opt opt(nlopt::LD_MMA, n);  // MMA for inequality constraints

    // Set objective function
    opt.set_min_objective(yield_objective, nullptr);

    // Set bounds: allocations between 0 and 1
    std::vector<double> lb(n, 0.0);
    std::vector<double> ub(n, 1.0);
    opt.set_lower_bounds(lb);
    opt.set_upper_bounds(ub);

    // Add constraint: allocations sum to 1.0
    opt.add_equality_constraint(sum_constraint, nullptr, 1e-8);

    // Add constraint: liquidity requirement
    opt.add_inequality_constraint(liquidity_constraint, nullptr, 1e-8);

    // Initial guess: equal allocation
    std::vector<double> x(n, 1.0 / n);

    // Optimize
    double minf;
    nlopt::result result = opt.optimize(x, minf);

    // Use optimized allocation
    // ...
}
```

## Algorithm Selection Guide

### For Convexity Optimization
- **LD_LBFGS** (Recommended): Limited-memory BFGS, good for smooth objectives
- **LD_SLSQP**: Sequential quadratic programming, good for constraints
- **LN_COBYLA**: Derivative-free, good if gradients unavailable

### For Portfolio Rebalancing
- **LD_SLSQP** (Recommended): Good for equality constraints
- **LD_MMA**: Method of moving asymptotes, good for inequality constraints
- **GN_ISRES**: Global optimization, if local optima are a concern

### For Cash Allocation
- **LD_MMA** (Recommended): Good for inequality constraints (liquidity limits)
- **LD_SLSQP**: Good for mixed equality/inequality constraints
- **LN_BOBYQA**: Derivative-free, if yield functions are noisy

## Integration Checklist

### Pre-Integration
- [ ] Verify CMake version (3.15+)
- [ ] Verify C++ compiler (C++11+)
- [ ] Decide on license (MIT recommended)

### CMake Integration
- [ ] Add NLopt FetchContent declaration
- [ ] Set NLOPT_LICENSE_MIT=ON (if using MIT license)
- [ ] Add NLopt to fetchcontent_makeavailable
- [ ] Add NLopt to target_link_libraries

### Testing
- [ ] Create convexity optimization test
- [ ] Test portfolio rebalancing optimization
- [ ] Verify build succeeds
- [ ] Run integration tests

### Documentation
- [ ] Update API_DOCUMENTATION_INDEX.md
- [ ] Document algorithm selection
- [ ] Add usage examples

## Performance Considerations

### Algorithm Performance
- Gradient-based algorithms (LD_*) are faster but require gradients
- Derivative-free algorithms (LN_*) are slower but more robust
- Global algorithms (GN_*) are slowest but find global optima

### Optimization Tips
- Provide good initial guesses
- Set appropriate tolerances (xtol_rel, ftol_rel)
- Use gradient-based algorithms when possible
- Consider algorithm-specific parameters

## Troubleshooting

### Common Issues

1. **Optimization Fails**
   - Check constraint feasibility
   - Verify initial guess is within bounds
   - Try different algorithm
   - Adjust tolerances

2. **Build Errors**
   - Verify CMake version
   - Check C++ standard (C++11+)
   - Verify license selection

3. **Link Errors**
   - Verify NLopt library path
   - Check namespace targets (NLopt::nlopt)
   - Verify license compatibility

## References

- **NLopt Website:** https://nlopt.readthedocs.io/
- **NLopt GitHub:** https://github.com/stevengj/nlopt
- **NLopt C++ Reference:** https://nlopt.readthedocs.io/en/stable/NLopt_C-plus-plus_Reference/
- **Algorithm Guide:** https://nlopt.readthedocs.io/en/stable/NLopt_Algorithms/
- **Project Research:** `docs/RESEARCH_CPP_FINANCIAL_LIBRARIES.md`

## Next Steps

1. ⏳ Integrate NLopt via CMake
2. ⏳ Implement convexity optimization
3. ⏳ Add portfolio rebalancing optimization
4. ⏳ Integrate with investment strategy framework

---

**Document Status:** Documentation Prepared (Integration Pending)
**Last Updated:** 2025-11-18
