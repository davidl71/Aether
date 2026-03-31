//! Request/response DTOs and wire enums for `api::finance_rates` (NATS / JSON read model).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct BoxSpreadInput {
    pub symbol: String,
    pub expiry: String,
    pub days_to_expiry: i32,
    pub strike_width: f64,
    #[serde(default)]
    pub strike_low: Option<f64>,
    #[serde(default)]
    pub strike_high: Option<f64>,
    pub buy_implied_rate: f64,
    pub sell_implied_rate: f64,
    pub net_debit: f64,
    pub net_credit: f64,
    pub liquidity_score: f64,
    pub spread_id: Option<String>,
    #[serde(default)]
    pub convenience_yield: Option<f64>,
    #[serde(default)]
    pub delayed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatePointResponse {
    pub symbol: String,
    pub expiry: String,
    pub days_to_expiry: i32,
    pub strike_width: f64,
    pub buy_implied_rate: f64,
    pub sell_implied_rate: f64,
    pub mid_rate: f64,
    pub net_debit: f64,
    pub net_credit: f64,
    pub liquidity_score: f64,
    pub timestamp: String,
    pub spread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strike_low: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strike_high: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub convenience_yield: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveResponse {
    pub symbol: String,
    pub points: Vec<RatePointResponse>,
    pub timestamp: String,
    pub strike_width: Option<f64>,
    pub point_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlying_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResponse {
    pub dte: i32,
    pub box_spread_rate: f64,
    pub benchmark_rate: f64,
    pub benchmark_type: String,
    pub spread_bps: f64,
    pub liquidity_score: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRateResponse {
    pub tenor: String,
    pub rate: f64,
    pub days_to_expiry: Option<i32>,
    pub timestamp: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofrBenchmarksResponse {
    pub overnight: SofrOvernightResponse,
    pub term_rates: Vec<BenchmarkRateResponse>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofrOvernightResponse {
    pub rate: Option<f64>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryBenchmarksResponse {
    pub rates: Vec<BenchmarkRateResponse>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarksResponse {
    pub sofr: SofrBenchmarksResponse,
    pub treasury: TreasuryBenchmarksResponse,
    /// Israeli overnight rate (Bank of Israel). Populated when backend has a SHIR data source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shir: Option<f64>,
    #[serde(default)]
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurveQuery {
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CurveRequest {
    Opportunities(Vec<Value>),
    Named {
        opportunities: Vec<Value>,
        symbol: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CompareRequest {
    Opportunities(Vec<Value>),
    Named {
        opportunities: Vec<Value>,
        symbol: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct YieldCurveComparisonRequest {
    pub box_spread_rates: HashMap<String, HashMap<String, f64>>,
    pub treasury_rates: Option<HashMap<String, f64>>,
    pub sofr_rates: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub fetch_live: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct YieldCurveComparisonResponse {
    pub symbols: Vec<String>,
    pub tenor_count: usize,
    pub treasury_points: usize,
    pub sofr_points: usize,
    pub spread_points: usize,
    pub box_spread_wins: usize,
    pub benchmark_wins: usize,
    pub ties: usize,
    pub generated: String,
    pub box_curves: HashMap<String, Vec<CurvePointResponse>>,
    pub treasury_curve: Vec<BenchmarkCurvePointResponse>,
    pub sofr_curve: Vec<BenchmarkCurvePointResponse>,
    pub spreads: Vec<SpreadPointResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurvePointResponse {
    pub dte: i32,
    pub rate: f64,
    pub liquidity: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkCurvePointResponse {
    pub dte: i32,
    pub rate: f64,
    pub tenor: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpreadPointResponse {
    pub dte: i32,
    pub tenor: String,
    pub box_rate: f64,
    pub benchmark_rate: f64,
    pub benchmark_source: String,
    pub spread_bps: f64,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub(crate) struct BenchmarkRate {
    pub rate_type: String,
    pub tenor: String,
    pub days_to_expiry: Option<i32>,
    pub rate: f64,
    pub timestamp: String,
    pub source: String,
}
