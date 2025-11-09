use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketDataEvent {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait MarketDataSource: Send + Sync {
    async fn next(&self) -> anyhow::Result<MarketDataEvent>;
}
