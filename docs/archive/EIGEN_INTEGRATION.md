# Eigen Integration Guide

**Date:** 2025-11-18
**Status:** Integrated ✅
**Version:** Eigen 3.4.0

## Overview

Eigen is a C++ template library for linear algebra, providing efficient matrix and vector operations. It has been integrated into the IBKR box spread trading application to support portfolio optimization calculations, convexity optimization, and matrix operations required by the investment strategy framework.

## Integration Details

### CMake Configuration

Eigen is integrated via CMake `FetchContent` in `native/CMakeLists.txt`:

```cmake
set(EIGEN3_REPOSITORY "https://gitlab.com/libeigen/eigen.git")
fetchcontent_declare(
    Eigen3
    GIT_REPOSITORY ${EIGEN3_REPOSITORY}
    GIT_TAG 3.4.0
    GIT_SHALLOW TRUE
)

fetchcontent_makeavailable(nlohmann_json spdlog CLI11 ftxui Eigen3)
```

### Linking

Eigen is linked to the main executable and test targets:

```cmake
target_link_libraries(ib_box_spread
    PRIVATE
        ...
        Eigen3::Eigen
        ...
)
```

**Note:** Eigen is header-only, so no actual linking occurs. The `Eigen3::Eigen` target provides proper include directory configuration.

## Usage

### Basic Includes

```cpp

#include <Eigen/Dense>  // For dense matrices and vectors
#include <Eigen/Sparse> // For sparse matrices (if needed)
```

### Common Use Cases

#### 1. Portfolio Allocation Matrix Operations

```cpp

#include <Eigen/Dense>

// Portfolio weights vector (must sum to 1.0)
Eigen::VectorXd weights(3);
weights << 0.4, 0.3, 0.3;

// Verify weights sum to 1.0
double sum = weights.sum();
assert(std::abs(sum - 1.0) < 1e-10);

// Covariance matrix for risk calculation
Eigen::MatrixXd covariance(3, 3);
covariance << 0.04, 0.02, 0.01,
              0.02, 0.03, 0.015,
              0.01, 0.015, 0.025;

// Portfolio variance: w^T * C * w
double portfolio_variance = weights.transpose() * covariance * weights;
```

#### 2. Convexity Optimization (Barbell Strategy)

```cpp

#include <Eigen/Dense>

// Bond durations for convexity calculation
Eigen::VectorXd durations(2);
durations << 2.5,  // Short-term bonds (years)
             25.0; // Long-term bonds (years)

// Portfolio weights for barbell strategy
Eigen::VectorXd barbell_weights(2);
barbell_weights << 0.5, 0.5;  // 50% short, 50% long

// Weighted average duration
double avg_duration = barbell_weights.dot(durations);

// Convexity calculation (simplified)
Eigen::VectorXd convexities(2);
convexities << 5.0,   // Short-term convexity
               150.0; // Long-term convexity

double portfolio_convexity = barbell_weights.dot(convexities);
```

#### 3. Linear System Solving (Portfolio Optimization)

```cpp

#include <Eigen/Dense>

// Solve: A * x = b
// Where A is constraint matrix, b is target values
Eigen::MatrixXd A(2, 2);
A << 1, 1,      // Constraint: weights sum to 1
     2, 3;      // Constraint: weighted duration target

Eigen::VectorXd b(2);
b << 1.0,       // Sum constraint
     10.0;      // Duration target

// Solve using QR decomposition
Eigen::VectorXd x = A.colPivHouseholderQr().solve(b);
```

#### 4. Matrix Operations for Risk Metrics

```cpp

#include <Eigen/Dense>

// Correlation matrix
Eigen::MatrixXd correlation(3, 3);
correlation << 1.0, 0.5, 0.3,
               0.5, 1.0, 0.4,
               0.3, 0.4, 1.0;

// Volatility vector
Eigen::VectorXd volatilities(3);
volatilities << 0.20, 0.15, 0.18;

// Convert to covariance matrix: C = diag(σ) * R * diag(σ)
Eigen::MatrixXd covariance = volatilities.asDiagonal() *
                             correlation *
                             volatilities.asDiagonal();
```

## Testing

Integration tests are located in `native/tests/eigen_integration_test.cpp`:

- Basic matrix operations
- Matrix multiplication
- Vector operations
- Linear system solving
- C++20 compatibility verification

Run tests with:

```bash
ctest --test-dir build --output-on-failure
```

## Performance Considerations

### Expression Templates

Eigen uses expression templates to optimize operations:

```cpp
// This is optimized at compile-time, no temporary matrices created
Eigen::MatrixXd result = A * B + C * D;
```

### Lazy Evaluation

Eigen evaluates expressions lazily:

```cpp
// No computation until result is assigned
auto expression = A.transpose() * B;
Eigen::MatrixXd result = expression;  // Computation happens here
```

### Memory Alignment

Eigen matrices are aligned for optimal performance. Use `Eigen::aligned_allocator` for STL containers:

```cpp
std::vector<Eigen::VectorXd, Eigen::aligned_allocator<Eigen::VectorXd>> vectors;
```

## C++20 Compatibility

Eigen 3.4.0 is fully compatible with C++20. The integration test verifies:

- Lambda functions with Eigen types
- `auto` type deduction
- Modern C++ features

## Integration with Investment Strategy Framework

Eigen is used in the following components (planned):

1. **PortfolioAllocationManager** (`docs/INVESTMENT_STRATEGY_FRAMEWORK.md`)
   - Matrix operations for allocation calculations
   - Covariance matrix calculations
   - Risk metric computations

2. **ConvexityCalculator**
   - Barbell strategy optimization
   - Duration and convexity calculations
   - Portfolio convexity aggregation

3. **SpareCashAllocator**
   - Rate comparison matrix operations
   - Allocation optimization calculations

## Dependencies

- **None:** Eigen is header-only, no external dependencies
- **CMake:** Uses FetchContent for automatic download
- **C++ Standard:** C++03+ (fully compatible with C++20)

## License

Eigen is licensed under MPL2 (Mozilla Public License 2.0), which is permissive and compatible with the project.

## References

- **Eigen Website:** <https://eigen.tuxfamily.org/>
- **Eigen Documentation:** <https://eigen.tuxfamily.org/dox/>
- **Getting Started:** <https://eigen.tuxfamily.org/dox/GettingStarted.html>
- **GitLab Repository:** <https://gitlab.com/libeigen/eigen>

## Next Steps

1. ✅ Eigen integrated and tested
2. ⏳ Implement portfolio allocation matrix operations
3. ⏳ Implement convexity optimization calculations
4. ⏳ Add Eigen-based risk metric calculations

---

**Document Status:** Integration Complete ✅
**Last Updated:** 2025-11-18
