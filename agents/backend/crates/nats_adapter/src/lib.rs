//! NATS Adapter for IBKR Box Spread Trading System
//!
//! This crate provides a bridge between Tokio channels and NATS message queue,
//! enabling event-driven communication across multiple language components.

pub mod bridge;
pub mod client;
pub mod error;
pub mod serde;
pub mod topics;

pub use bridge::{ChannelBridge, Publisher, Subscriber};
pub use client::NatsClient;
pub use error::{NatsAdapterError, Result};
pub use topics::{validate_topic, topic_matches};

/// Re-export commonly used types
pub use async_nats;
