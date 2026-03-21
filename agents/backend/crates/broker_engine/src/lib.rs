//! Broker abstraction — trait + domain types for engine switching.
//!
//! The [`BrokerEngine`](traits::BrokerEngine) trait abstracts all broker operations,
//! enabling the backend to switch between implementations (IBKR, yatws, mock) without
//! code changes. Domain types are broker-agnostic and live in [`domain`](domain).
//!
//! # Crate structure
//!
//! - [`traits`](traits) — [`BrokerEngine`] async trait definition
//! - [`domain`](domain) — domain types, events, config, helpers
//! - [`error`](error) — [`BrokerError`] enum

pub mod domain;
pub mod error;
pub mod traits;

pub use domain::*;
pub use error::BrokerError;
pub use traits::{BrokerEngine, OptionChainProvider};
