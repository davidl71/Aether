// constants.h - Named constants for trading, risk, and margin calculations.
// Centralises literals so intent is documented and updates stay in one place.
// Do not scatter numeric literals through source files — add them here instead.
#pragma once

namespace constants {

// ============================================================================
// Options market conventions
// ============================================================================

/// Standard equity option contract size (shares per contract).
inline constexpr double kOptionsContractMultiplier = 100.0;

/// Annualisation factor: trading days used to convert daily vol to annual.
inline constexpr double kTradingDaysPerYear = 252.0;

/// Calendar days per year — used in Black-Scholes time-to-expiry conversion.
inline constexpr double kCalendarDaysPerYear = 365.0;

// ============================================================================
// Default market parameters (fallbacks when live data is unavailable)
// ============================================================================

/// Approximate Fed Funds effective rate (2025). Used when no live rate feed.
inline constexpr double kDefaultRiskFreeRate = 0.045;

/// Default implied volatility when market data is absent.
inline constexpr double kDefaultImpliedVolatility = 0.20;

/// Initial IV guess for Newton-Raphson / Black-Scholes solver (30% annualised).
inline constexpr double kIvNewtonSeed = 0.30;

// ============================================================================
// Implied-volatility clamp bounds
// ============================================================================

/// Floor applied to aggregated IV — prevents near-zero/degenerate inputs.
inline constexpr double kIvClampLow = 0.05;

/// Cap applied to aggregated IV — rejects clearly erroneous outliers.
inline constexpr double kIvClampHigh = 1.50;

// ============================================================================
// Reg-T margin percentages  (FINRA Rule 4210)
// ============================================================================

/// 20% of underlying — base Reg-T requirement for uncovered short options.
inline constexpr double kRegTMarginPct = 0.20;

/// 10% floor — minimum Reg-T requirement regardless of OTM amount.
inline constexpr double kRegTMarginFloorPct = 0.10;

// ============================================================================
// Margin ratios
// ============================================================================

/// Maintenance margin = 75% of initial margin (standard broker convention).
inline constexpr double kMaintenanceMarginRatio = 0.75;

/// Conservative portfolio-margin multiplier applied on top of Reg-T.
inline constexpr double kPortfolioMarginMultiplier = 0.60;

// ============================================================================
// Numerical stability guards
// ============================================================================

/// Vega below this threshold causes the IV solver to abort the iteration.
inline constexpr double kVegaEpsilon = 1e-10;

/// Std-dev below this threshold treats the asset as non-volatile (correlation
/// undefined → set to identity).
inline constexpr double kStdDevEpsilon = 1e-12;

// ============================================================================
// Correlation heuristics (sign-based, no historical data)
// ============================================================================

/// Both returns positive or both negative → moderate positive correlation.
inline constexpr double kCorrelationSameDirection = 0.70;

/// Returns move in opposite directions → lower correlation.
inline constexpr double kCorrelationOppositeDir = 0.30;

/// No price data available for either leg → neutral fallback.
inline constexpr double kCorrelationFallback = 0.50;

// ============================================================================
// Risk monitoring thresholds
// ============================================================================

/// Fraction of max_total_exposure that triggers a Warning-level alert.
inline constexpr double kExposureWarningFraction = 0.90;

/// Order efficiency ratio below this triggers a low-efficiency warning.
inline constexpr double kOrderEfficiencyFloor = 0.05;

// ============================================================================
// Box spread execution
// ============================================================================

/// Minimum execution_probability for an opportunity to be considered actionable.
inline constexpr double kMinExecutionProbability = 0.70;

// ============================================================================
// VaR scaling (simple fractional approximation — pending full implementation)
// ============================================================================

inline constexpr double kVar95ScalingFactor = 0.05;
inline constexpr double kVar99ScalingFactor = 0.10;

} // namespace constants
