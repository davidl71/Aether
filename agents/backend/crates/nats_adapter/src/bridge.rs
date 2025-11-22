use std::marker::PhantomData;

use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::client::NatsClient;
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
      _phantom: PhantomData,
    }
  }

  /// Publish a message to NATS
  pub async fn publish(&self, payload: T) -> Result<()> {
    let bytes = serialize_message(&self.source, &self.message_type, payload)?;

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
