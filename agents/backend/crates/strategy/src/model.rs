use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategySignal {
    pub symbol: String,
    pub price: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Decision {
    pub symbol: String,
    pub quantity: i32,
    pub side: TradeSide,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}
