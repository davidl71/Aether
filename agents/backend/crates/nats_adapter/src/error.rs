use thiserror::Error;

/// Result type for NATS adapter operations
pub type Result<T> = std::result::Result<T, NatsAdapterError>;

/// Errors that can occur in the NATS adapter
#[derive(Error, Debug)]
pub enum NatsAdapterError {
  #[error("NATS connection error: {0}")]
  Connection(String),

  #[error("Serialization error: {0}")]
  Serialization(#[from] serde_json::Error),

  #[error("Channel error: {0}")]
  Channel(String),

  #[error("Publish error: {0}")]
  Publish(String),

  #[error("Subscribe error: {0}")]
  Subscribe(String),

  #[error("Invalid subject: {0}")]
  InvalidSubject(String),

  #[error("Message encoding error: {0}")]
  Encoding(String),

  #[error("Protobuf encode error: {0}")]
  ProtoEncode(#[from] prost::EncodeError),

  #[error("Protobuf decode error: {0}")]
  ProtoDecode(#[from] prost::DecodeError),

  #[error("Bridge error: {0}")]
  Bridge(String),
}

impl From<async_nats::ConnectError> for NatsAdapterError {
  fn from(err: async_nats::ConnectError) -> Self {
    NatsAdapterError::Connection(format!("Connection error: {}", err))
  }
}

impl From<async_nats::PublishError> for NatsAdapterError {
  fn from(err: async_nats::PublishError) -> Self {
    NatsAdapterError::Publish(format!("Publish error: {}", err))
  }
}

impl From<async_nats::SubscribeError> for NatsAdapterError {
  fn from(err: async_nats::SubscribeError) -> Self {
    NatsAdapterError::Subscribe(format!("Subscribe error: {}", err))
  }
}

// FlushError may not exist in this version of async-nats
// Handle flush errors as Connection errors

impl From<tokio::sync::mpsc::error::SendError<bytes::Bytes>> for NatsAdapterError {
  fn from(err: tokio::sync::mpsc::error::SendError<bytes::Bytes>) -> Self {
    NatsAdapterError::Channel(format!("Failed to send message: {}", err))
  }
}
