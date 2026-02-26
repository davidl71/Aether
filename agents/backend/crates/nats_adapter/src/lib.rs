//! NATS Adapter for IBKR Box Spread Trading System
//!
//! This crate provides a bridge between Tokio channels and NATS message queue,
//! enabling event-driven communication across multiple language components.

pub mod bridge;
pub mod client;
pub mod dlq;
pub mod error;
pub mod proto;
pub mod rpc;
pub mod serde;
pub mod topics;

pub use bridge::{ChannelBridge, Publisher, Subscriber};
pub use client::NatsClient;
pub use dlq::{DlqConfig, DlqService, DeadLetterMessage, error_type_from_error};
pub use error::{NatsAdapterError, Result};
pub use topics::{validate_topic, topic_matches};

/// Re-export commonly used types
pub use async_nats;
