use bytes::Bytes;
use prost::Message as ProstMessage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{NatsAdapterError, Result};
use crate::proto::v1::NatsEnvelope;

/// Wrapper for NATS messages with metadata (JSON path, kept for backward compat)
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
  pub fn new(source: impl Into<String>, message_type: impl Into<String>, payload: T) -> Self {
    Self {
      id: Uuid::new_v4().to_string(),
      timestamp: chrono::Utc::now(),
      source: source.into(),
      message_type: message_type.into(),
      payload,
    }
  }

  pub fn to_bytes(&self) -> Result<Bytes> {
    let json = serde_json::to_vec(self).map_err(NatsAdapterError::Serialization)?;
    Ok(Bytes::from(json))
  }
}

impl<T> NatsMessage<T>
where
  T: for<'de> Deserialize<'de>,
{
  pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
    serde_json::from_slice(bytes).map_err(NatsAdapterError::Serialization)
  }
}

// ---------------------------------------------------------------------------
// JSON helpers (legacy path)
// ---------------------------------------------------------------------------

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

pub fn deserialize_message<T>(bytes: &[u8]) -> Result<NatsMessage<T>>
where
  T: for<'de> Deserialize<'de>,
{
  NatsMessage::from_bytes(bytes)
}

pub fn extract_payload<T>(bytes: &[u8]) -> Result<T>
where
  T: for<'de> Deserialize<'de>,
{
  let message: NatsMessage<T> = deserialize_message(bytes)?;
  Ok(message.payload)
}

// ---------------------------------------------------------------------------
// Protobuf helpers (preferred path for wire encoding)
// ---------------------------------------------------------------------------

/// Encode a proto message directly (no envelope).
pub fn encode_proto<T: ProstMessage>(msg: &T) -> Result<Bytes> {
  Ok(Bytes::from(msg.encode_to_vec()))
}

/// Decode a proto message directly (no envelope).
pub fn decode_proto<T: ProstMessage + Default>(bytes: &[u8]) -> Result<T> {
  T::decode(bytes).map_err(NatsAdapterError::ProtoDecode)
}

/// Wrap a proto payload inside a `NatsEnvelope` and encode.
pub fn encode_envelope<T: ProstMessage>(
  source: &str,
  message_type: &str,
  payload: &T,
) -> Result<Bytes> {
  let envelope = NatsEnvelope {
    id: Uuid::new_v4().to_string(),
    timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
    source: source.into(),
    message_type: message_type.into(),
    payload: payload.encode_to_vec(),
  };
  Ok(Bytes::from(envelope.encode_to_vec()))
}

/// Decode a `NatsEnvelope` and extract the inner proto payload.
pub fn decode_envelope<T: ProstMessage + Default>(bytes: &[u8]) -> Result<(NatsEnvelope, T)> {
  let envelope = NatsEnvelope::decode(bytes).map_err(NatsAdapterError::ProtoDecode)?;
  let payload = T::decode(envelope.payload.as_slice()).map_err(NatsAdapterError::ProtoDecode)?;
  Ok((envelope, payload))
}

/// Decode a `NatsEnvelope` and return just the payload.
pub fn extract_proto_payload<T: ProstMessage + Default>(bytes: &[u8]) -> Result<T> {
  let (_, payload) = decode_envelope(bytes)?;
  Ok(payload)
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

  #[test]
  fn test_proto_encode_decode_market_data() {
    use crate::proto::v1::MarketDataEvent;

    let event = MarketDataEvent {
      symbol: "SPX".into(),
      bid: 4500.25,
      ask: 4500.75,
      last: 4500.50,
      volume: 1_000_000,
      timestamp: None,
    };

    let bytes = encode_proto(&event).unwrap();
    let decoded: MarketDataEvent = decode_proto(&bytes).unwrap();

    assert_eq!(decoded.symbol, "SPX");
    assert!((decoded.bid - 4500.25).abs() < f64::EPSILON);
    assert!((decoded.ask - 4500.75).abs() < f64::EPSILON);
  }

  #[test]
  fn test_proto_envelope_round_trip() {
    use crate::proto::v1::MarketDataEvent;

    let event = MarketDataEvent {
      symbol: "NDX".into(),
      bid: 15000.0,
      ask: 15001.0,
      last: 15000.5,
      volume: 500,
      timestamp: None,
    };

    let bytes = encode_envelope("backend", "MarketDataEvent", &event).unwrap();
    let (envelope, decoded): (NatsEnvelope, MarketDataEvent) =
      decode_envelope(&bytes).unwrap();

    assert_eq!(envelope.source, "backend");
    assert_eq!(envelope.message_type, "MarketDataEvent");
    assert_eq!(decoded.symbol, "NDX");
  }
}
