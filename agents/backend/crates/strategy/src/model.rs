//! Core strategy DTOs: signals and discrete decisions.

/// Snapshot of a signal (e.g. indicative price level) for a symbol at a point in time.
#[derive(Clone, Debug)]
pub struct StrategySignal {
    /// Underlying or tradable symbol.
    pub symbol: String,
    /// Observed or indicative price.
    pub price: f64,
    /// When the signal was generated.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A proposed order-style action (quantity and side) for a symbol.
#[derive(Clone, Debug)]
pub struct Decision {
    /// Target symbol.
    pub symbol: String,
    /// Signed quantity intent (convention matches caller).
    pub quantity: i32,
    /// Buy or sell.
    pub side: TradeSide,
}

/// Trade direction for a [`Decision`].
#[derive(Clone, Debug)]
pub enum TradeSide {
    /// Long / buy side.
    Buy,
    /// Short / sell side.
    Sell,
}
