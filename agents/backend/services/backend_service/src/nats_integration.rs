//! NATS Integration Module
//!
//! Provides NATS publishing capabilities for backend service components.
//! Runs in parallel to existing Tokio channels for gradual migration.

use std::collections::HashMap;
use std::sync::Arc;

use nats_adapter::{ChannelBridge, DlqService, NatsClient, ProtoPublisher, Publisher, topics};
use nats_adapter::proto::v1 as pb;
use strategy::{Decision as StrategyDecisionModel, StrategySignal};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// NATS integration state
pub struct NatsIntegration {
  client: Option<Arc<NatsClient>>,
  strategy_signal_publisher: Option<Arc<Publisher<StrategySignal>>>,
  strategy_decision_publisher: Option<Arc<Publisher<StrategyDecisionModel>>>,
  market_data_publishers: Arc<RwLock<HashMap<String, Arc<ProtoPublisher<pb::MarketDataEvent>>>>>,
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

    // Create DLQ service for failed messages
    let dlq_service = DlqService::new(client_clone.clone(), "backend");

    // Create publishers for strategy signals and decisions
    // We need separate bridges because ChannelBridge is generic over the message type
    let signal_bridge: ChannelBridge<StrategySignal> = ChannelBridge::new(client_clone.clone());
    let decision_bridge: ChannelBridge<StrategyDecisionModel> = ChannelBridge::new(client_clone.clone());

    let strategy_signal_pub = signal_bridge.create_publisher_with_dlq(
      topics::strategy::all_signals(),
      "backend",
      "StrategySignal",
      dlq_service.clone(),
    );
    let strategy_decision_pub = decision_bridge.create_publisher_with_dlq(
      topics::strategy::all_decisions(),
      "backend",
      "StrategyDecision",
      dlq_service.clone(),
    );

    Some(Self {
      client: Some(client),
      strategy_signal_publisher: Some(Arc::new(strategy_signal_pub)),
      strategy_decision_publisher: Some(Arc::new(strategy_decision_pub)),
      market_data_publishers: Arc::new(RwLock::new(HashMap::new())),
    })
  }

  /// Publish market data tick as protobuf (non-blocking, logs errors)
  pub async fn publish_market_data(&self, symbol: &str, bid: f64, ask: f64) {
    if let Some(ref client) = self.client {
      let publisher = {
        let mut publishers = self.market_data_publishers.write().await;
        publishers
          .entry(symbol.to_string())
          .or_insert_with(|| {
            Arc::new(ProtoPublisher::new(
              client.as_ref().clone(),
              topics::market_data::tick(symbol),
              "backend",
              "MarketDataEvent",
            ))
          })
          .clone()
      };

      let event = pb::MarketDataEvent {
        symbol: symbol.to_string(),
        bid,
        ask,
        last: (bid + ask) * 0.5,
        volume: 0,
        timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
      };

      if let Err(e) = publisher.publish(&event).await {
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

  /// Check NATS client connection health
  ///
  /// Attempts to flush the connection to verify it's alive.
  /// Returns "ok" if connected and flush succeeds, "degraded" if connected but flush fails,
  /// or "unavailable" if not connected.
  #[allow(dead_code)]
  pub async fn check_connection_health(&self) -> String {
    if let Some(ref client) = self.client {
      // Try to flush with a short timeout to verify connection
      match tokio::time::timeout(
        std::time::Duration::from_millis(500),
        client.as_ref().flush(),
      )
      .await
      {
        Ok(Ok(_)) => "ok".to_string(),
        Ok(Err(e)) => {
          warn!(error = %e, "NATS client flush failed");
          "degraded".to_string()
        }
        Err(_) => {
          warn!("NATS client flush timed out");
          "timeout".to_string()
        }
      }
    } else {
      "unavailable".to_string()
    }
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
