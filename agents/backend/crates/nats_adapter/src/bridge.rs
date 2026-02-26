use std::marker::PhantomData;

use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::client::NatsClient;
use crate::dlq::{DlqService, error_type_from_error};
use crate::error::{NatsAdapterError, Result};
use crate::serde::{deserialize_message, serialize_message};

/// Bridge between Tokio channels and NATS topics
pub struct ChannelBridge<T> {
  client: NatsClient,
  _phantom: PhantomData<T>,
}

impl<T> ChannelBridge<T>
where
  T: Send + Sync + 'static,
{
  /// Create a new channel bridge
  pub fn new(client: NatsClient) -> Self {
    Self {
      client,
      _phantom: PhantomData,
    }
  }

  /// Create a publisher that sends messages from a Tokio channel to NATS
  pub fn create_publisher(
    &self,
    subject: impl Into<String>,
    source: impl Into<String>,
    message_type: impl Into<String>,
  ) -> Publisher<T>
  where
    T: serde::Serialize,
  {
    Publisher::new(
      self.client.clone(),
      subject.into(),
      source.into(),
      message_type.into(),
    )
  }

  /// Create a publisher with DLQ support
  pub fn create_publisher_with_dlq(
    &self,
    subject: impl Into<String>,
    source: impl Into<String>,
    message_type: impl Into<String>,
    dlq_service: DlqService,
  ) -> Publisher<T>
  where
    T: serde::Serialize,
  {
    Publisher::with_dlq(
      self.client.clone(),
      subject.into(),
      source.into(),
      message_type.into(),
      dlq_service,
    )
  }

  /// Create a subscriber that receives messages from NATS and sends to a Tokio channel
  pub fn create_subscriber(
    &self,
    subject: impl Into<String>,
  ) -> Subscriber<T>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    Subscriber::new(self.client.clone(), subject.into())
  }
}

/// Publisher that bridges Tokio channel to NATS
pub struct Publisher<T> {
  client: NatsClient,
  subject: String,
  source: String,
  message_type: String,
  dlq_service: Option<DlqService>,
  _phantom: PhantomData<T>,
}

impl<T> Publisher<T>
where
  T: serde::Serialize + Send + Sync + 'static,
{
  fn new(client: NatsClient, subject: String, source: String, message_type: String) -> Self {
    Self {
      client,
      subject,
      source,
      message_type,
      dlq_service: None,
      _phantom: PhantomData,
    }
  }

  /// Create a publisher with DLQ support
  pub fn with_dlq(
    client: NatsClient,
    subject: String,
    source: String,
    message_type: String,
    dlq_service: DlqService,
  ) -> Self {
    Self {
      client,
      subject,
      source,
      message_type,
      dlq_service: Some(dlq_service),
      _phantom: PhantomData,
    }
  }

  /// Publish a message to NATS with retry logic and DLQ support
  pub async fn publish(&self, payload: T) -> Result<()> {
    // Serialize payload first (before retries)
    let original_payload = serde_json::to_value(&payload)
      .map_err(|e| {
        error!(
          subject = %self.subject,
          error = %e,
          "Failed to serialize message payload"
        );
        NatsAdapterError::Serialization(e)
      })?;

    let bytes = serialize_message(&self.source, &self.message_type, payload)?;

    // If DLQ is enabled, use retry logic
    if let Some(ref dlq_service) = self.dlq_service {
      let config = dlq_service.config();
      let mut last_error = None;

      // Retry loop
      for attempt in 0..=config.max_retries {
        match self
          .client
          .client()
          .publish(self.subject.clone(), bytes.clone())
          .await
        {
          Ok(_) => {
            if attempt > 0 {
              info!(
                subject = %self.subject,
                attempt = attempt,
                "Message published successfully after {} retries",
                attempt
              );
            }
            return Ok(());
          }
          Err(e) => {
            last_error = Some(NatsAdapterError::from(e));

            // If this was the last attempt, send to DLQ
            if attempt >= config.max_retries {
              let error = last_error.unwrap();
              let error_type = error_type_from_error(&error);
              let error_message = format!("{}", error);

              // Send to DLQ (non-blocking, log errors but don't fail)
              if let Err(dlq_err) = dlq_service
                .publish_failed_message(
                  &self.subject,
                  error_type,
                  &error_message,
                  attempt,
                  original_payload.clone(),
                  None,
                )
                .await
              {
                error!(
                  dlq_error = %dlq_err,
                  "Failed to send message to DLQ after publish failure"
                );
              }

              return Err(error);
            }

            // Calculate delay and wait before retry
            let delay = dlq_service.calculate_retry_delay(attempt);
            warn!(
              subject = %self.subject,
              attempt = attempt + 1,
              max_retries = config.max_retries,
              delay_ms = delay.as_millis(),
              error = %last_error.as_ref().unwrap(),
              "Publish failed, retrying in {:?}",
              delay
            );
            tokio::time::sleep(delay).await;
          }
        }
      }

      // Should never reach here, but handle it just in case
      Err(last_error.unwrap_or_else(|| {
        NatsAdapterError::Publish("Unknown error during retry loop".to_string())
      }))
    } else {
      // No DLQ, just try once
      self
        .client
        .client()
        .publish(self.subject.clone(), bytes)
        .await
        .map_err(|e| {
          error!(
            subject = %self.subject,
            error = %e,
            "Failed to publish message to NATS"
          );
          NatsAdapterError::from(e)
        })?;

      Ok(())
    }
  }

  /// Spawn a task that reads from a Tokio channel and publishes to NATS
  pub fn spawn_bridge(
    &self,
    mut rx: mpsc::UnboundedReceiver<T>,
  ) -> tokio::task::JoinHandle<()> {
    let publisher = self.clone();
    tokio::spawn(async move {
      while let Some(payload) = rx.recv().await {
        if let Err(e) = publisher.publish(payload).await {
          error!(error = %e, "Failed to publish message in bridge");
          // Continue processing other messages
        }
      }
      info!(subject = %publisher.subject, "Channel bridge closed");
    })
  }
}

impl<T> Clone for Publisher<T> {
  fn clone(&self) -> Self {
    Self {
      client: self.client.clone(),
      subject: self.subject.clone(),
      source: self.source.clone(),
      message_type: self.message_type.clone(),
      dlq_service: self.dlq_service.clone(),
      _phantom: PhantomData,
    }
  }
}

/// Subscriber that bridges NATS to Tokio channel
pub struct Subscriber<T> {
  client: NatsClient,
  subject: String,
  _phantom: PhantomData<T>,
}

impl<T> Subscriber<T>
where
  T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
{
  fn new(client: NatsClient, subject: String) -> Self {
    Self {
      client,
      subject,
      _phantom: PhantomData,
    }
  }

  /// Spawn a task that subscribes to NATS and sends messages to a Tokio channel
  pub async fn spawn_bridge(
    &self,
    tx: mpsc::UnboundedSender<T>,
  ) -> Result<tokio::task::JoinHandle<()>> {
    let subject = self.subject.clone();
    let mut subscriber = self
      .client
      .client()
      .subscribe(subject.clone())
      .await
      .map_err(|e| {
        error!(
          subject = %subject,
          error = %e,
          "Failed to subscribe to NATS subject"
        );
        NatsAdapterError::from(e)
      })?;

    info!(subject = %subject, "Subscribed to NATS subject");

    let handle = tokio::spawn(async move {
      while let Some(msg) = subscriber.next().await {
        match deserialize_message::<T>(&msg.payload) {
          Ok(nats_msg) => {
            if let Err(e) = tx.send(nats_msg.payload) {
              warn!(
                error = %e,
                "Failed to send message to channel, channel closed"
              );
              break;
            }
          }
          Err(e) => {
            error!(
              error = %e,
              subject = %subject,
              "Failed to deserialize NATS message"
            );
            // Continue processing other messages
          }
        }
      }
      info!(subject = %subject, "NATS subscription closed");
    });

    Ok(handle)
  }
}

// ---------------------------------------------------------------------------
// Proto-native publisher / subscriber (prost::Message types)
// ---------------------------------------------------------------------------

/// Publisher that encodes payloads as protobuf (no JSON envelope).
pub struct ProtoPublisher<T: prost::Message> {
  client: NatsClient,
  subject: String,
  source: String,
  message_type: String,
  _phantom: PhantomData<T>,
}

impl<T> ProtoPublisher<T>
where
  T: prost::Message + Send + Sync + 'static,
{
  pub fn new(
    client: NatsClient,
    subject: impl Into<String>,
    source: impl Into<String>,
    message_type: impl Into<String>,
  ) -> Self {
    Self {
      client,
      subject: subject.into(),
      source: source.into(),
      message_type: message_type.into(),
      _phantom: PhantomData,
    }
  }

  pub async fn publish(&self, payload: &T) -> Result<()> {
    let bytes =
      crate::serde::encode_envelope(&self.source, &self.message_type, payload)?;
    self
      .client
      .client()
      .publish(self.subject.clone(), bytes)
      .await
      .map_err(|e| {
        error!(subject = %self.subject, error = %e, "proto publish failed");
        NatsAdapterError::from(e)
      })
  }
}

impl<T: prost::Message> Clone for ProtoPublisher<T> {
  fn clone(&self) -> Self {
    Self {
      client: self.client.clone(),
      subject: self.subject.clone(),
      source: self.source.clone(),
      message_type: self.message_type.clone(),
      _phantom: PhantomData,
    }
  }
}

/// Subscriber that decodes protobuf payloads from a NatsEnvelope.
pub struct ProtoSubscriber<T: prost::Message + Default> {
  client: NatsClient,
  subject: String,
  _phantom: PhantomData<T>,
}

impl<T> ProtoSubscriber<T>
where
  T: prost::Message + Default + Send + Sync + 'static,
{
  pub fn new(client: NatsClient, subject: impl Into<String>) -> Self {
    Self {
      client,
      subject: subject.into(),
      _phantom: PhantomData,
    }
  }

  pub async fn spawn_bridge(
    &self,
    tx: mpsc::UnboundedSender<T>,
  ) -> Result<tokio::task::JoinHandle<()>> {
    let subject = self.subject.clone();
    let mut subscriber = self
      .client
      .client()
      .subscribe(subject.clone())
      .await
      .map_err(|e| {
        error!(subject = %subject, error = %e, "proto subscribe failed");
        NatsAdapterError::from(e)
      })?;

    info!(subject = %subject, "proto subscriber active");

    let handle = tokio::spawn(async move {
      while let Some(msg) = subscriber.next().await {
        match crate::serde::extract_proto_payload::<T>(&msg.payload) {
          Ok(payload) => {
            if tx.send(payload).is_err() {
              warn!("proto subscriber channel closed");
              break;
            }
          }
          Err(e) => {
            error!(error = %e, subject = %subject, "proto decode failed");
          }
        }
      }
      info!(subject = %subject, "proto subscription closed");
    });

    Ok(handle)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
  struct TestMessage {
    value: i32,
  }

  // Note: These tests require a running NATS server
  // They should be run as integration tests with a test server
}
