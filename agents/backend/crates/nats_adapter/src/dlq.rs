//! Dead Letter Queue (DLQ) Service
//!
//! Handles failed messages by sending them to a dead letter queue topic
//! with retry logic and exponential backoff (std-only; matches `backoff` crate
//! behavior for `randomization_factor = 0` and `max_elapsed_time = None`).

use std::time::Duration;

use tracing::{error, warn};
use uuid::Uuid;

use std::sync::Arc;

use crate::client::NatsClient;
use crate::error::{NatsAdapterError, Result};
use crate::serde::encode_envelope;
use crate::topics::dlq;

/// Configuration for DLQ retry behavior
#[derive(Debug, Clone)]
pub struct DlqConfig {
    /// Maximum number of retry attempts before sending to DLQ
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_retry_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_retry_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Enable DLQ publishing (can be disabled for testing)
    pub enabled: bool,
}

impl Default for DlqConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
            enabled: true,
        }
    }
}

/// Dead letter message structure
#[derive(Clone, PartialEq, prost::Message)]
pub struct DeadLetterMessage {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: Option<prost_types::Timestamp>,
    #[prost(string, tag = "3")]
    pub original_topic: String,
    #[prost(string, tag = "4")]
    pub component: String,
    #[prost(string, tag = "5")]
    pub error_type: String,
    #[prost(string, tag = "6")]
    pub error_message: String,
    #[prost(uint32, tag = "7")]
    pub retry_count: u32,
    #[prost(bytes = "vec", tag = "8")]
    pub original_payload: Vec<u8>,
    #[prost(string, optional, tag = "9")]
    pub metadata_json: Option<String>,
}

/// DLQ Service for publishing failed messages
#[derive(Clone)]
pub struct DlqService {
    client: NatsClient,
    config: Arc<DlqConfig>,
    component: String,
}

impl DlqService {
    /// Create a new DLQ service
    pub fn new(client: NatsClient, component: impl Into<String>) -> Self {
        Self {
            client,
            config: Arc::new(DlqConfig::default()),
            component: component.into(),
        }
    }

    /// Create a DLQ service with custom configuration
    pub fn with_config(
        client: NatsClient,
        component: impl Into<String>,
        config: DlqConfig,
    ) -> Self {
        Self {
            client,
            config: Arc::new(config),
            component: component.into(),
        }
    }

    /// Publish a failed message to DLQ
    ///
    /// # Arguments
    /// * `original_topic` - The topic where the message was supposed to be published
    /// * `error_type` - Type of error that occurred
    /// * `error_message` - Human-readable error message
    /// * `retry_count` - Number of retries attempted
    /// * `original_payload` - The original message payload that failed
    /// * `metadata` - Optional additional metadata
    pub async fn publish_failed_message(
        &self,
        original_topic: &str,
        error_type: &str,
        error_message: &str,
        retry_count: u32,
        original_payload: Vec<u8>,
        metadata_json: Option<String>,
    ) -> Result<()> {
        if !self.config.enabled {
            warn!(
              original_topic = %original_topic,
              error_type = %error_type,
              "DLQ disabled, not publishing failed message"
            );
            return Ok(());
        }

        let dlq_message = DeadLetterMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            original_topic: original_topic.to_string(),
            component: self.component.clone(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            retry_count,
            original_payload,
            metadata_json,
        };

        let dlq_topic = dlq::dead_letter(&self.component, error_type);
        let bytes = encode_envelope(&self.component, "DeadLetterMessage", &dlq_message)?;

        self.client
            .client()
            .publish(dlq_topic.clone(), bytes)
            .await
            .map_err(|e| {
                error!(
                  dlq_topic = %dlq_topic,
                  error = %e,
                  "Failed to publish message to DLQ - this is a critical error"
                );
                NatsAdapterError::Publish(format!("DLQ publish failed: {}", e))
            })?;

        warn!(
          dlq_topic = %dlq_topic,
          original_topic = %original_topic,
          retry_count = retry_count,
          "Message sent to DLQ after {} retries",
          retry_count
        );

        Ok(())
    }

    /// Calculate retry delay using exponential backoff (same sequence as
    /// `backoff::ExponentialBackoff` with `randomization_factor = 0` and no max elapsed).
    pub fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        exponential_retry_delay_for_attempt(
            attempt,
            self.config.initial_retry_delay_ms,
            self.config.max_retry_delay_ms,
            self.config.backoff_multiplier,
        )
    }

    /// Get the DLQ configuration
    pub fn config(&self) -> &DlqConfig {
        &self.config
    }
}

/// Exponential retry delay for `attempt` (0-based), matching `backoff` with zero jitter.
pub(crate) fn exponential_retry_delay_for_attempt(
    attempt: u32,
    initial_retry_delay_ms: u64,
    max_retry_delay_ms: u64,
    backoff_multiplier: f64,
) -> Duration {
    let initial = Duration::from_millis(initial_retry_delay_ms);
    let max_d = Duration::from_millis(max_retry_delay_ms);
    let max_s = max_d.as_secs_f64();
    let mut interval_s = initial.as_secs_f64().min(max_s);
    let mut out = initial;
    for _ in 0..=attempt {
        out = Duration::from_secs_f64(interval_s);
        interval_s = (interval_s * backoff_multiplier).min(max_s);
    }
    out
}

/// Extract error type from NatsAdapterError
pub fn error_type_from_error(error: &NatsAdapterError) -> &'static str {
    match error {
        NatsAdapterError::Publish(_) => "publish_error",
        NatsAdapterError::Serialization(_) => "serialization_error",
        NatsAdapterError::Connection(_) => "connection_error",
        NatsAdapterError::Encoding(_) => "serialization_error",
        NatsAdapterError::InvalidSubject(_) => "validation_error",
        _ => "unknown_error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_retry_delay_matches_steps() {
        assert_eq!(
            exponential_retry_delay_for_attempt(0, 100, 5000, 2.0),
            Duration::from_millis(100)
        );
        assert_eq!(
            exponential_retry_delay_for_attempt(1, 100, 5000, 2.0),
            Duration::from_millis(200)
        );
        assert_eq!(
            exponential_retry_delay_for_attempt(2, 100, 5000, 2.0),
            Duration::from_millis(400)
        );
        assert_eq!(
            exponential_retry_delay_for_attempt(3, 100, 5000, 2.0),
            Duration::from_millis(800)
        );
    }

    #[test]
    fn exponential_retry_delay_respects_max() {
        assert_eq!(
            exponential_retry_delay_for_attempt(0, 100, 250, 2.0),
            Duration::from_millis(100)
        );
        assert_eq!(
            exponential_retry_delay_for_attempt(1, 100, 250, 2.0),
            Duration::from_millis(200)
        );
        assert_eq!(
            exponential_retry_delay_for_attempt(2, 100, 250, 2.0),
            Duration::from_millis(250)
        );
        assert_eq!(
            exponential_retry_delay_for_attempt(3, 100, 250, 2.0),
            Duration::from_millis(250)
        );
    }
}
