//! Integration tests for NATS adapter
//!
//! These tests require a running NATS server.
//! Start NATS server before running: `./scripts/start_nats.sh`

use bytes::Bytes;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use nats_adapter::{
  bridge::{ChannelBridge, Publisher, Subscriber},
  client::{default_connect_options, NatsClient},
  serde::NatsMessage,
};

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

  // Subscribe
  let mut subscriber = client
    .client()
    .subscribe(subject)
    .await
    .expect("Failed to subscribe");

  // Publish
  let payload = Bytes::from("test message");
  client
    .client()
    .publish(subject, payload.clone())
    .await
    .expect("Failed to publish");

  // Receive
  let message = subscriber.next().await.expect("No message received");
  assert_eq!(message.payload, payload);
}

#[tokio::test]
#[ignore]
async fn test_channel_bridge_publisher() {
  let client = NatsClient::connect("nats://localhost:4222").await.unwrap();
  let bridge = ChannelBridge::new(client.clone());
  let subject = "test.bridge.publisher";

  // Create publisher
  let publisher = bridge.create_publisher(subject, "test-source", "TestMessage");

  // Subscribe to verify
  let mut subscriber = client
    .client()
    .subscribe(subject)
    .await
    .expect("Failed to subscribe");

  // Publish via bridge
  let payload = TestPayload {
    value: 42,
    text: "bridge test".to_string(),
  };
  publisher.publish(payload.clone()).await.expect("Failed to publish");

  // Receive and verify
  let message = subscriber.next().await.expect("No message received");
  let nats_msg: NatsMessage<TestPayload> =
    serde_json::from_slice(&message.payload).expect("Failed to deserialize");
  assert_eq!(nats_msg.payload, payload);
}

#[tokio::test]
#[ignore]
async fn test_channel_bridge_subscriber() {
  let client = NatsClient::connect("nats://localhost:4222").await.unwrap();
  let bridge = ChannelBridge::new(client.clone());
  let subject = "test.bridge.subscriber";

  // Create subscriber bridge
  let (tx, mut rx) = mpsc::unbounded_channel();
  let subscriber = bridge.create_subscriber(subject);
  let _handle = subscriber
    .spawn_bridge(tx)
    .expect("Failed to spawn subscriber bridge");

  // Wait for subscription to be ready
  sleep(Duration::from_millis(100)).await;

  // Publish message
  let payload = TestPayload {
    value: 100,
    text: "subscriber test".to_string(),
  };
  let message = NatsMessage::new("test-source", "TestMessage", payload.clone());
  let bytes = message.to_bytes().expect("Failed to serialize");
  client
    .client()
    .publish(subject, bytes)
    .await
    .expect("Failed to publish");

  // Receive via channel
  let received = rx.recv().await.expect("No message received");
  assert_eq!(received, payload);
}

#[tokio::test]
#[ignore]
async fn test_channel_bridge_full_loop() {
  let client = NatsClient::connect("nats://localhost:4222").await.unwrap();
  let bridge = ChannelBridge::new(client.clone());
  let subject = "test.bridge.full_loop";

  // Create channel for publishing
  let (pub_tx, pub_rx) = mpsc::unbounded_channel();
  let publisher = bridge.create_publisher(subject, "test-source", "TestMessage");
  let _pub_handle = publisher.spawn_bridge(pub_rx);

  // Create channel for subscribing
  let (sub_tx, mut sub_rx) = mpsc::unbounded_channel();
  let subscriber = bridge.create_subscriber(subject);
  let _sub_handle = subscriber
    .spawn_bridge(sub_tx)
    .expect("Failed to spawn subscriber bridge");

  // Wait for subscription to be ready
  sleep(Duration::from_millis(100)).await;

  // Send message through channel -> NATS -> channel
  let payload = TestPayload {
    value: 200,
    text: "full loop test".to_string(),
  };
  pub_tx.send(payload.clone()).expect("Failed to send");

  // Receive via subscriber channel
  let received = sub_rx.recv().await.expect("No message received");
  assert_eq!(received, payload);
}
