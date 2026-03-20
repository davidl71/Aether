use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, derive_builder::Builder)]
#[builder(setter(into, strip_option), default)]
pub struct MarketDataEvent {
    #[builder(default = "0")]
    pub contract_id: i64,
    #[builder(setter(into))]
    pub symbol: String,
    #[builder(default = "0.0")]
    pub bid: f64,
    #[builder(default = "0.0")]
    pub ask: f64,
    #[builder(default = "0.0")]
    pub last: f64,
    #[builder(default = "0")]
    pub volume: i64,
    #[builder(default = "Utc::now()")]
    pub timestamp: DateTime<Utc>,
    #[builder(default = "0")]
    pub quote_quality: u32,
}

impl Default for MarketDataEvent {
    fn default() -> Self {
        Self {
            contract_id: 0,
            symbol: String::new(),
            bid: 0.0,
            ask: 0.0,
            last: 0.0,
            volume: 0,
            timestamp: Utc::now(),
            quote_quality: 0,
        }
    }
}

#[async_trait::async_trait]
pub trait MarketDataSource: Send + Sync {
    async fn next(&self) -> anyhow::Result<MarketDataEvent>;
}
