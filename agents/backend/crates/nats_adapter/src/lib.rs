//! NATS messaging: client wrapper, protobuf envelopes, subjects, RPC helpers, and health hooks.
//!
//! Use [`topics`] for **canonical subject strings** (avoid scattering literals). Use [`NatsClient`]
//! and [`ChannelBridge`] to integrate with async services. See repository `docs/NATS_TOPICS_REGISTRY.md`
//! for an operator-facing subject listing.
//!
//! # Example — topic wildcard check
//!
//! ```
//! use nats_adapter::topic_matches;
//!
//! assert!(topic_matches("market-data.*.SPY", "market-data.tick.SPY"));
//! assert!(!topic_matches("api.cmd", "system.heartbeat"));
//! ```

pub mod bridge;
pub mod client;
pub mod conversions;
pub mod dlq;
pub mod error;
pub mod health;
pub mod proto;
pub mod rpc;
pub mod serde;
pub mod topics;

pub use bridge::{ChannelBridge, Publisher, Subscriber};
pub use client::NatsClient;
pub use dlq::{error_type_from_error, DeadLetterMessage, DlqConfig, DlqService};
pub use error::{NatsAdapterError, Result};
pub use health::{spawn_health_publisher, NatsTransportHealthState};
pub use rpc::{
    request_json, request_json_with_retry, request_json_with_retry_timeout,
    request_json_with_timeout, request_proto, request_proto_with_timeout, serve_json, serve_proto,
    RetryConfig,
};
pub use serde::{
    decode_envelope, decode_proto, encode_envelope, encode_proto, extract_proto_payload,
};
pub use topics::{topic_matches, validate_topic};

/// Re-export commonly used types
pub use async_nats;
