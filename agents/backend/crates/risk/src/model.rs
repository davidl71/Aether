#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct RiskLimit {
    pub symbol: String,
    pub max_position: i32,
    pub max_notional: f64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct RiskDecision {
    pub allowed: bool,
    pub reason: Option<String>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct RiskViolation {
    pub symbol: String,
    pub details: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct PositionRisk {
    pub position_size: f64,
    pub max_loss: f64,
    pub max_gain: f64,
    pub expected_value: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub leverage: f64,
    pub probability_of_profit: f64,
    pub risk_reward_ratio: f64,
    pub initial_margin: f64,
    pub maintenance_margin: f64,
    pub margin_utilization: f64,
    pub margin_call_risk: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct PortfolioRisk {
    pub total_exposure: f64,
    pub total_delta: f64,
    pub total_gamma: f64,
    pub total_theta: f64,
    pub total_vega: f64,
    pub var_95: f64,
    pub var_99: f64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct BoxSpreadLeg {
    pub net_debit: f64,
    pub strike_width: f64,
}

impl BoxSpreadLeg {
    pub fn new(net_debit: f64, strike_width: f64) -> Self {
        Self {
            net_debit,
            strike_width,
        }
    }

    pub fn get_strike_width(&self) -> f64 {
        self.strike_width
    }
}
