use bytes::Bytes;
use prost::Message as ProstMessage;
use uuid::Uuid;

use crate::error::{NatsAdapterError, Result};
use crate::proto::v1::NatsEnvelope;

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

    #[derive(Clone, PartialEq, prost::Message)]
    struct TestPayload {
        #[prost(int32, tag = "1")]
        value: i32,
        #[prost(string, tag = "2")]
        text: String,
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
        let (envelope, decoded): (NatsEnvelope, MarketDataEvent) = decode_envelope(&bytes).unwrap();

        assert_eq!(envelope.source, "backend");
        assert_eq!(envelope.message_type, "MarketDataEvent");
        assert_eq!(decoded.symbol, "NDX");
    }

    #[test]
    fn test_extract_proto_payload() {
        let payload = TestPayload {
            value: 100,
            text: "extract".to_string(),
        };

        let bytes = encode_envelope("source", "TestPayload", &payload).unwrap();
        let extracted: TestPayload = extract_proto_payload(&bytes).unwrap();

        assert_eq!(extracted.value, payload.value);
        assert_eq!(extracted.text, payload.text);
    }

    #[test]
    fn test_decode_envelope_rejects_json_payload() {
        let bytes = br#"{"source":"legacy","type":"TestMessage","payload":{"value":1}}"#;
        let result = decode_envelope::<TestPayload>(bytes);
        assert!(matches!(result, Err(NatsAdapterError::ProtoDecode(_))));
    }
}
