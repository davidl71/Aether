//! Integration tests for backend service NATS integration
//!
//! These tests require:
//! - Running NATS server: `./scripts/start_nats.sh`
//! - Backend service compiled: `cargo build -p backend_service`

use std::time::Duration;

use futures::StreamExt;
use nats_adapter::{topics, ChannelBridge, NatsClient};
use serde::{Deserialize, Serialize};
use strategy::{Decision as StrategyDecisionModel, StrategySignal, TradeSide};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MarketDataTick {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_market_data_publishing() {
    let client = NatsClient::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let bridge = ChannelBridge::new(client.clone());
    let subject = topics::market_data::tick("TEST");

    // Subscribe to verify
    let mut subscriber = client
        .client()
        .subscribe(&subject)
        .await
        .expect("Failed to subscribe");

    // Create publisher
    let publisher = bridge.create_publisher(&subject, "test", "MarketDataTick");

    // Publish test message
    let tick = MarketDataTick {
        symbol: "TEST".to_string(),
        bid: 100.0,
        ask: 100.1,
        timestamp: chrono::Utc::now(),
    };

    publisher
        .publish(tick.clone())
        .await
        .expect("Failed to publish");

    // Wait for message
    sleep(Duration::from_millis(100)).await;

    // Receive and verify
    if let Some(msg) = subscriber.next().await {
        let payload: serde_json::Value =
            serde_json::from_slice(&msg.payload).expect("Failed to deserialize");
        assert_eq!(payload["payload"]["symbol"], "TEST");
    } else {
        panic!("No message received");
    }
}

#[tokio::test]
#[ignore]
async fn test_strategy_signal_publishing() {
    let client = NatsClient::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let bridge = ChannelBridge::new(client.clone());
    let subject = topics::strategy::signal("TEST");

    // Subscribe to wildcard
    let mut subscriber = client
        .client()
        .subscribe(topics::strategy::all_signals())
        .await
        .expect("Failed to subscribe");

    // Create publisher
    let publisher = bridge.create_publisher(&subject, "test", "StrategySignal");

    // Publish test signal
    let signal = StrategySignal {
        symbol: "TEST".to_string(),
        price: 100.5,
        timestamp: chrono::Utc::now(),
    };

    publisher
        .publish(signal.clone())
        .await
        .expect("Failed to publish");

    // Wait for message
    sleep(Duration::from_millis(100)).await;

    // Receive and verify
    if let Some(msg) = subscriber.next().await {
        let payload: serde_json::Value =
            serde_json::from_slice(&msg.payload).expect("Failed to deserialize");
        assert_eq!(payload["payload"]["symbol"], "TEST");
    } else {
        panic!("No message received");
    }
}

#[tokio::test]
#[ignore]
async fn test_strategy_decision_publishing() {
    let client = NatsClient::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let bridge = ChannelBridge::new(client.clone());
    let subject = topics::strategy::decision("TEST");

    // Subscribe to wildcard
    let mut subscriber = client
        .client()
        .subscribe(topics::strategy::all_decisions())
        .await
        .expect("Failed to subscribe");

    // Create publisher
    let publisher = bridge.create_publisher(&subject, "test", "StrategyDecision");

    // Publish test decision
    let decision = StrategyDecisionModel {
        symbol: "TEST".to_string(),
        quantity: 1,
        side: TradeSide::Buy,
    };

    publisher
        .publish(decision.clone())
        .await
        .expect("Failed to publish");

    // Wait for message
    sleep(Duration::from_millis(100)).await;

    // Receive and verify
    if let Some(msg) = subscriber.next().await {
        let payload: serde_json::Value =
            serde_json::from_slice(&msg.payload).expect("Failed to deserialize");
        assert_eq!(payload["payload"]["symbol"], "TEST");
    } else {
        panic!("No message received");
    }
}

#[tokio::test]
#[ignore]
async fn test_topic_validation() {
    use nats_adapter::topics;

    // Test valid topics
    assert!(topics::validate_topic("market-data.tick.SPY").is_ok());
    assert!(topics::validate_topic("strategy.signal.>").is_ok());
    assert!(topics::validate_topic("orders.status.123").is_ok());

    // Test invalid topics
    assert!(topics::validate_topic("").is_err());
    assert!(topics::validate_topic(".invalid").is_err());
    assert!(topics::validate_topic("invalid..topic").is_err());
}

#[tokio::test]
#[ignore]
async fn test_wildcard_subscriptions() {
    let client = NatsClient::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let bridge = ChannelBridge::new(client.clone());

    // Subscribe to all market data
    let mut subscriber = client
        .client()
        .subscribe(topics::market_data::all())
        .await
        .expect("Failed to subscribe");

    // Publish to specific symbol
    let publisher_spy =
        bridge.create_publisher(topics::market_data::tick("SPY"), "test", "MarketDataTick");
    let publisher_xsp =
        bridge.create_publisher(topics::market_data::tick("XSP"), "test", "MarketDataTick");

    let tick_spy = MarketDataTick {
        symbol: "SPY".to_string(),
        bid: 100.0,
        ask: 100.1,
        timestamp: chrono::Utc::now(),
    };

    let tick_xsp = MarketDataTick {
        symbol: "XSP".to_string(),
        bid: 50.0,
        ask: 50.1,
        timestamp: chrono::Utc::now(),
    };

    publisher_spy
        .publish(tick_spy)
        .await
        .expect("Failed to publish");
    publisher_xsp
        .publish(tick_xsp)
        .await
        .expect("Failed to publish");

    // Wait for messages
    sleep(Duration::from_millis(200)).await;

    // Should receive both messages
    let mut received = 0;
    while let Some(_msg) = subscriber.next().await {
        received += 1;
        if received >= 2 {
            break;
        }
    }

    assert!(
        received >= 2,
        "Expected at least 2 messages, got {}",
        received
    );
}
