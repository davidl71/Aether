use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use chrono::Utc;
use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::{sync::Mutex, time::sleep};

use crate::{MarketDataEvent, MarketDataEventBuilder, MarketDataSource};

struct MockState {
    baselines: HashMap<String, f64>,
    rng: StdRng,
}

pub struct MockMarketDataSource {
    symbols: Vec<String>,
    interval: Duration,
    state: Mutex<MockState>,
}

impl MockMarketDataSource {
    pub fn new<I, S>(symbols: I, interval: Duration) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let symbols_vec: Vec<String> = symbols.into_iter().map(Into::into).collect();
        let baselines = symbols_vec
            .iter()
            .map(|symbol| (symbol.clone(), 100.0))
            .collect();

        Self {
            symbols: symbols_vec,
            interval,
            state: Mutex::new(MockState {
                baselines,
                rng: StdRng::from_entropy(),
            }),
        }
    }
}

#[async_trait]
impl MarketDataSource for MockMarketDataSource {
    async fn next(&self) -> anyhow::Result<MarketDataEvent> {
        sleep(self.interval).await;

        let mut state = self.state.lock().await;
        let symbol = if self.symbols.is_empty() {
            "XSP".to_string()
        } else {
            let idx = state.rng.gen_range(0..self.symbols.len());
            self.symbols[idx].clone()
        };

        let drift = state.rng.gen_range(-0.75..0.75);
        let spread = state.rng.gen_range(0.02..0.08);
        let entry = state.baselines.entry(symbol.clone()).or_insert(100.0);
        *entry = (*entry + drift).max(1.0);

        let price = *entry;
        let bid = (price - (spread / 2.0)).max(0.01);
        let ask = bid + spread;

        let event = MarketDataEventBuilder::default()
            .symbol(symbol)
            .bid(bid)
            .ask(ask)
            .timestamp(Utc::now())
            .build()?;
        Ok(event)
    }
}
