//! Broker abstraction — trait + domain types for engine switching.
//!
//! The [`crate::traits::BrokerEngine`] trait abstracts active read-only broker operations,
//! enabling the backend to switch between implementations (IBKR, mock) without
//! code changes. Domain types are broker-agnostic and live in [`crate::domain`].
//!
//! # Crate structure
//!
//! - [`crate::traits`] — `BrokerEngine` async trait definition
//! - [`crate::domain`] — domain types, events, config, helpers
//! - [`crate::error`] — `BrokerError` enum

pub mod domain;
pub mod error;
pub mod traits;

pub use domain::*;
pub use error::BrokerError;
pub use traits::{BrokerEngine, MarketDataSubscription, MarketDataSubscriptionError};
