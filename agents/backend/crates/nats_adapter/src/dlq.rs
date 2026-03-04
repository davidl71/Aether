//! Dead Letter Queue (DLQ) Service
//!
//! Handles failed messages by sending them to a dead letter queue topic
//! with retry logic and exponential backoff.

use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use uuid::Uuid;

use std::sync::Arc;

use crate::client::NatsClient;
use crate::error::{NatsAdapterError, Result};
use crate::serde::serialize_message;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterMessage {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub original_topic: String,
    pub component: String,
    pub error_type: String,
    pub error_message: String,
    pub retry_count: u32,
    pub original_payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
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
        original_payload: serde_json::Value,
        metadata: Option<serde_json::Value>,
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
            timestamp: Utc::now(),
            original_topic: original_topic.to_string(),
            component: self.component.clone(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            retry_count,
            original_payload,
            metadata,
        };

        let dlq_topic = dlq::dead_letter(&self.component, error_type);
        let bytes = serialize_message(&self.component, "DeadLetterMessage", &dlq_message)?;

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

    /// Calculate retry delay using exponential backoff
    pub fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.config.initial_retry_delay_ms as f64
            * self.config.backoff_multiplier.powi(attempt as i32))
        .min(self.config.max_retry_delay_ms as f64) as u64;

        Duration::from_millis(delay_ms)
    }

    /// Get the DLQ configuration
    pub fn config(&self) -> &DlqConfig {
        &self.config
    }
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
