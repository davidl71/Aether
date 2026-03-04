use std::sync::Arc;
use std::time::Duration;

use api::{Alert, SharedSnapshot};
use market_data::{
  MarketDataEvent, MarketDataIngestor, MarketDataSource, MockMarketDataSource,
  PolygonMarketDataSource,
};
use strategy::StrategySignal;
use tokio::sync::{mpsc::UnboundedSender, watch};
use tokio::time::sleep;
use tracing::warn;

use crate::config::{self, MarketDataSettings};
use crate::nats_integration;

pub fn spawn(
  settings: &MarketDataSettings,
  state: SharedSnapshot,
  strategy_signal: UnboundedSender<StrategySignal>,
  strategy_toggle: watch::Receiver<bool>,
  nats: Arc<Option<nats_integration::NatsIntegration>>,
) -> anyhow::Result<()> {
  let symbols = if settings.symbols.is_empty() {
    config::default_market_symbols()
  } else {
    settings.symbols.clone()
  };
  let interval = Duration::from_millis(settings.poll_interval_ms.max(10));

  match settings.provider.as_str() {
    "polygon" => {
      let polygon_cfg = settings
        .polygon
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("polygon provider selected but polygon settings missing"))?;
      let api_key = config::resolve_polygon_api_key(polygon_cfg)?;
      let base_url = polygon_cfg.base_url.as_deref();
      let source = PolygonMarketDataSource::new(symbols, interval, api_key, base_url)
        .map_err(|e| anyhow::anyhow!("failed to create polygon market data source: {e}"))?;
      spawn_loop(source, state, strategy_signal, strategy_toggle, nats);
    }
    provider => {
      if provider != "mock" {
        warn!(%provider, "unknown market data provider, falling back to mock");
      }
      let source = MockMarketDataSource::new(symbols, interval);
      spawn_loop(source, state, strategy_signal, strategy_toggle, nats);
    }
  }

  Ok(())
}

fn spawn_loop<S>(
  source: S,
  state: SharedSnapshot,
  strategy_signal: UnboundedSender<StrategySignal>,
  strategy_toggle: watch::Receiver<bool>,
  nats: Arc<Option<nats_integration::NatsIntegration>>,
) where
  S: MarketDataSource + Send + Sync + 'static,
{
  tokio::spawn(async move {
    let ingestor = MarketDataIngestor::new(source);

    loop {
      match ingestor.poll().await {
        Ok(event) => {
          let running = *strategy_toggle.borrow();
          handle_event(&state, &strategy_signal, &event, running, nats.as_ref().as_ref()).await;
        }
        Err(err) => {
          warn!(%err, "market data poll failed, retrying");
          sleep(Duration::from_secs(1)).await;
        }
      }
    }
  });
}

async fn handle_event(
  state: &SharedSnapshot,
  strategy_signal: &UnboundedSender<StrategySignal>,
  event: &MarketDataEvent,
  running: bool,
  nats: Option<&nats_integration::NatsIntegration>,
) {
  let spread = event.ask - event.bid;
  let mid = (event.bid + event.ask) * 0.5;

  if let Some(n) = nats {
    n.publish_market_data(&event.symbol, event.bid, event.ask).await;
  }

  {
    let mut snapshot = state.write().await;
    snapshot.apply_market_event(event);

    if spread > 0.4 {
      snapshot.alerts.push(Alert::warning(format!(
        "Wide spread detected on {}: {:.2}", event.symbol, spread
      )));
    }
    while snapshot.alerts.len() > 32 {
      snapshot.alerts.remove(0);
    }
  }

  if !running {
    return;
  }

  let signal = StrategySignal {
    symbol: event.symbol.clone(),
    price: mid,
    timestamp: event.timestamp,
  };

  if let Some(n) = nats {
    n.publish_strategy_signal(&signal).await;
  }

  if let Err(err) = strategy_signal.send(signal) {
    warn!(%err, "failed to queue strategy signal");
  }
}
