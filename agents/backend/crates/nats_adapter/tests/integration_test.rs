//! Integration tests for NATS adapter
//!
//! These tests require a running NATS server.
//! Start NATS server before running: `./scripts/start_nats.sh`

use bytes::Bytes;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use nats_adapter::{bridge::ChannelBridge, client::NatsClient, serde::NatsMessage};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestPayload {
    value: i32,
    text: String,
}

#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_connect() {
    let client = NatsClient::connect("nats://localhost:4222").await;
    assert!(client.is_ok());
    let client = client.unwrap();
    assert!(client.is_connected());
    client.close().await;
}

#[tokio::test]
#[ignore]
async fn test_publish_subscribe() {
    let client = NatsClient::connect("nats://localhost:4222").await.unwrap();
    let subject = "test.publish_subscribe";

    let mut subscriber = client
        .client()
        .subscribe(subject.to_string())
        .await
        .expect("Failed to subscribe");

    let payload = Bytes::from("test message");
    client
        .client()
        .publish(subject.to_string(), payload.clone())
        .await
        .expect("Failed to publish");

    let message = subscriber.next().await.expect("No message received");
    assert_eq!(message.payload, payload);
}

#[tokio::test]
#[ignore]
async fn test_channel_bridge_publisher() {
    let client = NatsClient::connect("nats://localhost:4222").await.unwrap();
    let bridge = ChannelBridge::new(client.clone());
    let subject = "test.bridge.publisher";

    let publisher = bridge.create_publisher(subject, "test-source", "TestMessage");

    let mut subscriber = client
        .client()
        .subscribe(subject.to_string())
        .await
        .expect("Failed to subscribe");

    let payload = TestPayload {
        value: 42,
        text: "bridge test".to_string(),
    };
    publisher
        .publish(payload.clone())
        .await
        .expect("Failed to publish");

    let message = subscriber.next().await.expect("No message received");
    let nats_msg: NatsMessage<TestPayload> =
        serde_json::from_slice(&message.payload).expect("Failed to deserialize");
    assert_eq!(nats_msg.payload, payload);
}

#[tokio::test]
#[ignore]
async fn test_channel_bridge_subscriber() {
    let client = NatsClient::connect("nats://localhost:4222").await.unwrap();
    let bridge: ChannelBridge<TestPayload> = ChannelBridge::new(client.clone());
    let subject = "test.bridge.subscriber";

    let (tx, mut rx) = mpsc::unbounded_channel();
    let subscriber = bridge.create_subscriber(subject);
    let _handle = subscriber
        .spawn_bridge(tx)
        .await
        .expect("Failed to spawn subscriber bridge");

    sleep(Duration::from_millis(100)).await;

    let payload = TestPayload {
        value: 100,
        text: "subscriber test".to_string(),
    };
    let message = NatsMessage::new("test-source", "TestMessage", payload.clone());
    let bytes = message.to_bytes().expect("Failed to serialize");
    client
        .client()
        .publish(subject.to_string(), bytes)
        .await
        .expect("Failed to publish");

    let received = rx.recv().await.expect("No message received");
    assert_eq!(received, payload);
}

#[tokio::test]
#[ignore]
async fn test_channel_bridge_full_loop() {
    let client = NatsClient::connect("nats://localhost:4222").await.unwrap();
    let bridge = ChannelBridge::new(client.clone());
    let subject = "test.bridge.full_loop";

    let (pub_tx, pub_rx) = mpsc::unbounded_channel();
    let publisher = bridge.create_publisher(subject, "test-source", "TestMessage");
    let _pub_handle = publisher.spawn_bridge(pub_rx);

    let (sub_tx, mut sub_rx) = mpsc::unbounded_channel();
    let subscriber = bridge.create_subscriber(subject);
    let _sub_handle = subscriber
        .spawn_bridge(sub_tx)
        .await
        .expect("Failed to spawn subscriber bridge");

    sleep(Duration::from_millis(100)).await;

    let payload = TestPayload {
        value: 200,
        text: "full loop test".to_string(),
    };
    pub_tx.send(payload.clone()).expect("Failed to send");

    let received = sub_rx.recv().await.expect("No message received");
    assert_eq!(received, payload);
}
