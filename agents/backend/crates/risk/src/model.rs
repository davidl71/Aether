use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskLimit {
    pub symbol: String,
    pub max_position: i32,
    pub max_notional: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskDecision {
    pub allowed: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskViolation {
    pub symbol: String,
    pub details: String,
}
