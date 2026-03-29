//! Finance rates / yield curve / benchmarks exposed via NATS `api.finance_rates.*`.
//! Curve sources are layered as cached KV, Yahoo option-chain data, then synthetic placeholders.
//! Source labels are preserved when available so the TUI can explain whether a curve came from
//! `tws`, `yahoo`, `synthetic`, or an explicit URL fallback. Proto KV entries remain supported
//! for older writers, but the read model should stay source-first rather than engine-first.
//! Benchmark rates are resolved independently through FMP, FRED, and New York Fed endpoints.
//! Keep this module read-model oriented; it is not an execution or broker-engine boundary.
//!
//! Layout: `types` (serde DTOs), `curve` (opportunity aggregation / synthetic curves),
//! `benchmarks` (HTTP/FRED/FMP), `comparison` (compare + yield-curve orchestration).

mod benchmarks;
mod comparison;
mod curve;
mod types;

#[cfg(test)]
mod tests;

pub use benchmarks::{get_sofr_rates, get_treasury_rates};
pub use comparison::{compare_rates, yield_curve_comparison};
pub use curve::{build_curve, build_synthetic_curve, extract_rate};
pub use types::{
    BenchmarkCurvePointResponse, BenchmarkRateResponse, BenchmarksResponse, BoxSpreadInput,
    CompareRequest, ComparisonResponse, CurvePointResponse, CurveQuery, CurveRequest,
    CurveResponse, RatePointResponse, SofrBenchmarksResponse, SofrOvernightResponse,
    SpreadPointResponse, TreasuryBenchmarksResponse, YieldCurveComparisonRequest,
    YieldCurveComparisonResponse,
};
