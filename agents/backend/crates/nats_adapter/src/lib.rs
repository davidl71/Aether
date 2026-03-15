//! NATS Adapter for IBKR Box Spread Trading System
//!
//! This crate provides a bridge between Tokio channels and NATS message queue,
//! enabling event-driven communication across multiple language components.

pub mod bridge;
pub mod client;
pub mod conversions;
pub mod dlq;
pub mod error;
pub mod proto;
pub mod rpc;
pub mod serde;
pub mod topics;

pub use bridge::{ChannelBridge, Publisher, Subscriber};
pub use client::NatsClient;
pub use dlq::{error_type_from_error, DeadLetterMessage, DlqConfig, DlqService};
pub use error::{NatsAdapterError, Result};
pub use rpc::{
    request_json, request_json_with_timeout, request_proto, request_proto_with_timeout, serve_json,
    serve_proto,
};
pub use serde::{
    decode_envelope, decode_proto, encode_envelope, encode_proto, extract_proto_payload,
};
pub use topics::{topic_matches, validate_topic};

/// Re-export commonly used types
pub use async_nats;
