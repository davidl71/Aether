//! NATS Integration Module
//!
//! Provides NATS publishing capabilities for backend service components.
//! Runs in parallel to existing Tokio channels for gradual migration.

use std::collections::HashMap;
use std::sync::Arc;

use nats_adapter::proto::v1 as pb;
use nats_adapter::{topics, DlqService, NatsClient, Publisher};
use strategy::{model::TradeSide, Decision as StrategyDecisionModel, StrategySignal};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// NATS integration state
pub struct NatsIntegration {
    client: Option<Arc<NatsClient>>,
    market_data_publishers: Arc<RwLock<HashMap<String, Arc<Publisher<pb::MarketDataEvent>>>>>,
    strategy_signal_publishers: Arc<RwLock<HashMap<String, Arc<Publisher<pb::StrategySignal>>>>>,
    strategy_decision_publishers:
        Arc<RwLock<HashMap<String, Arc<Publisher<pb::StrategyDecision>>>>>,
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
        Some(Self {
            client: Some(client),
            market_data_publishers: Arc::new(RwLock::new(HashMap::new())),
            strategy_signal_publishers: Arc::new(RwLock::new(HashMap::new())),
            strategy_decision_publishers: Arc::new(RwLock::new(HashMap::new())),
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
                        Arc::new(Publisher::new(
                            client.as_ref().clone(),
                            topics::market_data::tick(symbol),
                            "backend",
                            "MarketDataEvent",
                        ))
                    })
                    .clone()
            };

            let event = pb::MarketDataEvent {
                contract_id: 0,
                symbol: symbol.to_string(),
                bid,
                ask,
                last: (bid + ask) * 0.5,
                volume: 0,
                timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
                quote_quality: 0,
            };

            if let Err(e) = publisher.publish(&event).await {
                error!(error = %e, symbol = %symbol, "Failed to publish market data to NATS");
            }
        }
    }

    /// Publish strategy signal (non-blocking, logs errors)
    pub async fn publish_strategy_signal(&self, signal: &StrategySignal) {
        if let Some(ref client) = self.client {
            let publisher = {
                let mut publishers = self.strategy_signal_publishers.write().await;
                publishers
                    .entry(signal.symbol.clone())
                    .or_insert_with(|| {
                        Arc::new(Publisher::with_dlq(
                            client.as_ref().clone(),
                            topics::strategy::signal(&signal.symbol),
                            "backend",
                            "StrategySignal",
                            DlqService::new(client.as_ref().clone(), "backend"),
                        ))
                    })
                    .clone()
            };

            let proto_signal = pb::StrategySignal {
                symbol: signal.symbol.clone(),
                price: signal.price,
                timestamp: Some(prost_types::Timestamp {
                    seconds: signal.timestamp.timestamp(),
                    nanos: signal.timestamp.timestamp_subsec_nanos() as i32,
                }),
            };
            if let Err(e) = publisher.publish(&proto_signal).await {
                error!(error = %e, symbol = %signal.symbol, "Failed to publish strategy signal to NATS");
            }
        }
    }

    /// Publish strategy decision (non-blocking, logs errors)
    pub async fn publish_strategy_decision(&self, decision: &StrategyDecisionModel) {
        if let Some(ref client) = self.client {
            let publisher = {
                let mut publishers = self.strategy_decision_publishers.write().await;
                publishers
                    .entry(decision.symbol.clone())
                    .or_insert_with(|| {
                        Arc::new(Publisher::with_dlq(
                            client.as_ref().clone(),
                            topics::strategy::decision(&decision.symbol),
                            "backend",
                            "StrategyDecision",
                            DlqService::new(client.as_ref().clone(), "backend"),
                        ))
                    })
                    .clone()
            };

            let proto_decision = pb::StrategyDecision {
                symbol: decision.symbol.clone(),
                quantity: decision.quantity,
                side: match decision.side {
                    TradeSide::Buy => "BUY".to_string(),
                    TradeSide::Sell => "SELL".to_string(),
                },
                mark: 0.0,
                created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            };
            if let Err(e) = publisher.publish(&proto_decision).await {
                error!(
                  error = %e,
                  symbol = %decision.symbol,
                  "Failed to publish strategy decision to NATS"
                );
            }
        }
    }

    /// Return the underlying NATS client, if connected.
    pub fn client(&self) -> Option<Arc<NatsClient>> {
        self.client.clone()
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
            market_data_publishers: Arc::new(RwLock::new(HashMap::new())),
            strategy_signal_publishers: Arc::new(RwLock::new(HashMap::new())),
            strategy_decision_publishers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
