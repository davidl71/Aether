//! Integration tests for backend service NATS integration
//!
//! Platform topics (market-data, strategy.signal, strategy.decision) use protobuf
//! (NatsEnvelope + payload). These tests use the canonical Publisher/Subscriber
//! bridge types for end-to-end verification.
//!
//! Requires: running NATS server (`./scripts/start_nats.sh`), `cargo build -p backend_service`
//!
//! For api.* request/reply tests: also run one backend_service instance (subscribed to api.*).
//! Run ignored tests: `cargo test -p backend_service --test integration_test -- --ignored`

use std::time::Duration;

use nats_adapter::proto::v1 as pb;
use nats_adapter::{request_json_with_timeout, topics, NatsClient, Publisher, Subscriber};
use serde_json::Value;
use tokio::time::sleep;

/// Platform topic: market-data.tick.* — deserialize protobuf (NatsEnvelope + MarketDataEvent).
#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_market_data_publishing() {
    let client = NatsClient::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let subject = topics::market_data::tick("TEST");

    // Subscribe with proto deserialization (matches C++ publisher format)
    let proto_sub = Subscriber::<pb::MarketDataEvent>::new(client.clone(), subject.clone());
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let _handle = proto_sub
        .spawn_bridge(tx)
        .await
        .expect("Failed to spawn proto subscriber");

    // Publish using proto (same format as C++ nats_client.cpp)
    let publisher = Publisher::new(client.clone(), subject.clone(), "test", "MarketDataEvent");
    let event = pb::MarketDataEvent {
        contract_id: 0,
        symbol: "TEST".to_string(),
        bid: 100.0,
        ask: 100.1,
        last: 100.05,
        volume: 0,
        timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        quote_quality: 0,
        source: "test".to_string(),
        source_priority: 100,
    };
    publisher.publish(&event).await.expect("Failed to publish");

    sleep(Duration::from_millis(100)).await;

    if let Some(received) = rx.recv().await {
        assert_eq!(received.symbol, "TEST");
        assert!((received.bid - 100.0).abs() < 1e-9);
        assert!((received.ask - 100.1).abs() < 1e-9);
    } else {
        panic!("No message received");
    }
}

/// Platform topic: strategy.signal.* — deserialize protobuf (NatsEnvelope + StrategySignal).
#[tokio::test]
#[ignore]
async fn test_strategy_signal_publishing() {
    let client = NatsClient::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let subject = topics::strategy::signal("TEST");

    let proto_sub =
        Subscriber::<pb::StrategySignal>::new(client.clone(), topics::strategy::all_signals());
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let _handle = proto_sub
        .spawn_bridge(tx)
        .await
        .expect("Failed to spawn proto subscriber");

    let publisher = Publisher::new(client.clone(), subject.clone(), "test", "StrategySignal");
    let signal = pb::StrategySignal {
        symbol: "TEST".to_string(),
        price: 100.5,
        timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
    };
    publisher.publish(&signal).await.expect("Failed to publish");

    sleep(Duration::from_millis(100)).await;

    if let Some(received) = rx.recv().await {
        assert_eq!(received.symbol, "TEST");
        assert!((received.price - 100.5).abs() < 1e-9);
    } else {
        panic!("No message received");
    }
}

/// Platform topic: strategy.decision.* — deserialize protobuf (NatsEnvelope + StrategyDecision).
#[tokio::test]
#[ignore]
async fn test_strategy_decision_publishing() {
    let client = NatsClient::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let subject = topics::strategy::decision("TEST");

    let proto_sub =
        Subscriber::<pb::StrategyDecision>::new(client.clone(), topics::strategy::all_decisions());
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let _handle = proto_sub
        .spawn_bridge(tx)
        .await
        .expect("Failed to spawn proto subscriber");

    let publisher = Publisher::new(client.clone(), subject.clone(), "test", "StrategyDecision");
    let decision = pb::StrategyDecision {
        symbol: "TEST".to_string(),
        quantity: 1,
        side: "BUY".to_string(),
        mark: 100.0,
        created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
    };
    publisher
        .publish(&decision)
        .await
        .expect("Failed to publish");

    sleep(Duration::from_millis(100)).await;

    if let Some(received) = rx.recv().await {
        assert_eq!(received.symbol, "TEST");
        assert_eq!(received.quantity, 1);
        assert_eq!(received.side, "BUY");
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

    // Subscribe to all market data with proto deserialization
    let proto_sub =
        Subscriber::<pb::MarketDataEvent>::new(client.clone(), topics::market_data::all());
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let _handle = proto_sub
        .spawn_bridge(tx)
        .await
        .expect("Failed to spawn proto subscriber");

    let publisher_spy = Publisher::new(
        client.clone(),
        topics::market_data::tick("SPY"),
        "test",
        "MarketDataEvent",
    );
    let publisher_xsp = Publisher::new(
        client.clone(),
        topics::market_data::tick("XSP"),
        "test",
        "MarketDataEvent",
    );

    let event_spy = pb::MarketDataEvent {
        contract_id: 0,
        symbol: "SPY".to_string(),
        bid: 100.0,
        ask: 100.1,
        last: 100.05,
        volume: 0,
        timestamp: None,
        quote_quality: 0,
        source: "test".to_string(),
        source_priority: 100,
    };
    let event_xsp = pb::MarketDataEvent {
        contract_id: 0,
        symbol: "XSP".to_string(),
        bid: 50.0,
        ask: 50.1,
        last: 50.05,
        volume: 0,
        timestamp: None,
        quote_quality: 0,
        source: "test".to_string(),
        source_priority: 100,
    };

    publisher_spy
        .publish(&event_spy)
        .await
        .expect("Failed to publish");
    publisher_xsp
        .publish(&event_xsp)
        .await
        .expect("Failed to publish");

    sleep(Duration::from_millis(200)).await;

    let mut received = 0;
    while tokio::time::timeout(Duration::from_millis(500), rx.recv())
        .await
        .unwrap_or(None)
        .is_some()
    {
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

/// api.* request/reply: requires NATS and a running backend_service subscribed to api.*.
/// Run with: `cargo test -p backend_service --test integration_test test_api_request_reply_benchmarks -- --ignored`
#[tokio::test]
#[ignore] // Requires NATS server and backend_service running
async fn test_api_request_reply_benchmarks() {
    let url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let client = NatsClient::connect(&url)
        .await
        .expect("Failed to connect to NATS");

    let response: Value = request_json_with_timeout(
        &client,
        "api.finance_rates.benchmarks",
        &(),
        Duration::from_secs(5),
    )
    .await
    .expect("api.finance_rates.benchmarks request/reply");

    // Assert response shape (SOFR + Treasury; timestamps present)
    let sofr_ts = response
        .get("sofr")
        .and_then(|v| v.get("timestamp"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let treasury_ts = response
        .get("treasury")
        .and_then(|v| v.get("timestamp"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        !sofr_ts.is_empty() || !treasury_ts.is_empty(),
        "benchmarks response should have at least one non-empty timestamp"
    );
}
