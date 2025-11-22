use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{NatsAdapterError, Result};

/// Wrapper for NATS messages with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsMessage<T> {
  pub id: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub source: String,
  #[serde(rename = "type")]
  pub message_type: String,
  pub payload: T,
}

impl<T> NatsMessage<T>
where
  T: Serialize,
{
  /// Create a new NATS message
  pub fn new(source: impl Into<String>, message_type: impl Into<String>, payload: T) -> Self {
    Self {
      id: Uuid::new_v4().to_string(),
      timestamp: chrono::Utc::now(),
      source: source.into(),
      message_type: message_type.into(),
      payload,
    }
  }

  /// Serialize message to JSON bytes
  pub fn to_bytes(&self) -> Result<Bytes> {
    let json = serde_json::to_vec(self).map_err(NatsAdapterError::Serialization)?;
    Ok(Bytes::from(json))
  }
}

impl<T> NatsMessage<T>
where
  T: for<'de> Deserialize<'de>,
{
  /// Deserialize message from JSON bytes
  pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
    serde_json::from_slice(bytes).map_err(NatsAdapterError::Serialization)
  }
}

/// Serialize any serializable type to NATS message bytes
pub fn serialize_message<T>(
  source: impl Into<String>,
  message_type: impl Into<String>,
  payload: T,
) -> Result<Bytes>
where
  T: Serialize,
{
  let message = NatsMessage::new(source, message_type, payload);
  message.to_bytes()
}

/// Deserialize NATS message bytes to type
pub fn deserialize_message<T>(bytes: &[u8]) -> Result<NatsMessage<T>>
where
  T: for<'de> Deserialize<'de>,
{
  NatsMessage::from_bytes(bytes)
}

/// Extract payload from NATS message
pub fn extract_payload<T>(bytes: &[u8]) -> Result<T>
where
  T: for<'de> Deserialize<'de>,
{
  let message: NatsMessage<T> = deserialize_message(bytes)?;
  Ok(message.payload)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
  struct TestPayload {
    value: i32,
    text: String,
  }

  #[test]
  fn test_serialize_deserialize() {
    let payload = TestPayload {
      value: 42,
      text: "test".to_string(),
    };

    let message = NatsMessage::new("test-source", "TestMessage", payload.clone());
    let bytes = message.to_bytes().unwrap();
    let deserialized: NatsMessage<TestPayload> = NatsMessage::from_bytes(&bytes).unwrap();

    assert_eq!(deserialized.payload, payload);
    assert_eq!(deserialized.source, "test-source");
    assert_eq!(deserialized.message_type, "TestMessage");
  }

  #[test]
  fn test_extract_payload() {
    let payload = TestPayload {
      value: 100,
      text: "extract".to_string(),
    };

    let message = NatsMessage::new("source", "Type", payload.clone());
    let bytes = message.to_bytes().unwrap();
    let extracted: TestPayload = extract_payload(&bytes).unwrap();

    assert_eq!(extracted, payload);
  }
}
