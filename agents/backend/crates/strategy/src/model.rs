#[derive(Clone, Debug)]
pub struct StrategySignal {
    pub symbol: String,
    pub price: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
pub struct Decision {
    pub symbol: String,
    pub quantity: i32,
    pub side: TradeSide,
}

#[derive(Clone, Debug)]
pub enum TradeSide {
    Buy,
    Sell,
}
