//! NATS Integration Module
//!
//! Provides NATS publishing capabilities for backend service components.
//! Runs in parallel to existing Tokio channels for gradual migration.

use std::collections::HashMap;
use std::sync::Arc;

use nats_adapter::{ChannelBridge, NatsClient, Publisher, topics};
use strategy::{Decision as StrategyDecisionModel, StrategySignal};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// NATS integration state
pub struct NatsIntegration {
  client: Option<Arc<NatsClient>>,
  strategy_signal_publisher: Option<Arc<Publisher<StrategySignal>>>,
  strategy_decision_publisher: Option<Arc<Publisher<StrategyDecisionModel>>>,
  // Cache publishers per symbol for market data
  market_data_publishers: Arc<RwLock<HashMap<String, Arc<Publisher<MarketDataTick>>>>>,
}

/// Market data tick message for NATS
#[derive(serde::Serialize, Clone, Debug)]
struct MarketDataTick {
  symbol: String,
  bid: f64,
  ask: f64,
  timestamp: chrono::DateTime<chrono::Utc>,
}

impl NatsIntegration {
  /// Initialize NATS integration
  ///
  /// Returns None if NATS is unavailable (graceful degradation)
  pub async fn new(nats_url: Option<String>) -> Option<Self> {
    let url = nats_url.unwrap_or_else(|| "nats://localhost:4222".to_string());

    let client = match NatsClient::connect(&url).await {
      Ok(c) => {
        info!(url = %url, "NATS client connected");
        c
      }
      Err(e) => {
        warn!(error = %e, url = %url, "Failed to connect to NATS, continuing without NATS integration");
        return None;
      }
    };

    let client = Arc::new(client);
    let client_clone = client.as_ref().clone();

    // Create publishers for strategy signals and decisions
    // We need separate bridges because ChannelBridge is generic over the message type
    let signal_bridge: ChannelBridge<StrategySignal> = ChannelBridge::new(client_clone.clone());
    let decision_bridge: ChannelBridge<StrategyDecisionModel> = ChannelBridge::new(client_clone);

    let strategy_signal_pub = signal_bridge.create_publisher(
      topics::strategy::all_signals(),
      "backend",
      "StrategySignal",
    );
    let strategy_decision_pub = decision_bridge.create_publisher(
      topics::strategy::all_decisions(),
      "backend",
      "StrategyDecision",
    );

    Some(Self {
      client: Some(client),
      strategy_signal_publisher: Some(Arc::new(strategy_signal_pub)),
      strategy_decision_publisher: Some(Arc::new(strategy_decision_pub)),
      market_data_publishers: Arc::new(RwLock::new(HashMap::new())),
    })
  }

  /// Publish market data tick (non-blocking, logs errors)
  pub async fn publish_market_data(&self, symbol: &str, bid: f64, ask: f64) {
    if let Some(ref client) = self.client {
      // Get or create publisher for this symbol
      let publisher = {
        let mut publishers = self.market_data_publishers.write().await;
        publishers
          .entry(symbol.to_string())
          .or_insert_with(|| {
            let bridge = ChannelBridge::new(client.as_ref().clone());
            Arc::new(bridge.create_publisher(
              topics::market_data::tick(symbol),
              "backend",
              "MarketDataTick",
            ))
          })
          .clone()
      };

      let tick = MarketDataTick {
        symbol: symbol.to_string(),
        bid,
        ask,
        timestamp: chrono::Utc::now(),
      };

      if let Err(e) = publisher.publish(tick).await {
        error!(error = %e, symbol = %symbol, "Failed to publish market data to NATS");
      }
    }
  }

  /// Publish strategy signal (non-blocking, logs errors)
  pub async fn publish_strategy_signal(&self, signal: &StrategySignal) {
    if let Some(ref publisher) = self.strategy_signal_publisher {
      if let Err(e) = publisher.publish(signal.clone()).await {
        error!(error = %e, symbol = %signal.symbol, "Failed to publish strategy signal to NATS");
      }
    }
  }

  /// Publish strategy decision (non-blocking, logs errors)
  pub async fn publish_strategy_decision(&self, decision: &StrategyDecisionModel) {
    if let Some(ref publisher) = self.strategy_decision_publisher {
      if let Err(e) = publisher.publish(decision.clone()).await {
        error!(
          error = %e,
          symbol = %decision.symbol,
          "Failed to publish strategy decision to NATS"
        );
      }
    }
  }

  /// Check if NATS integration is active
  pub fn is_active(&self) -> bool {
    self.client.is_some()
  }
}

impl Default for NatsIntegration {
  fn default() -> Self {
    Self {
      client: None,
      strategy_signal_publisher: None,
      strategy_decision_publisher: None,
      market_data_publishers: Arc::new(RwLock::new(HashMap::new())),
    }
  }
}
