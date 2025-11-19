//! Ledger Core Library
//!
//! A lightweight double-entry accounting system optimized for trading operations.
//! Provides transaction recording, balance calculation, and Ledger CLI-compatible export.

pub mod account;
pub mod currency;
pub mod engine;
pub mod error;
pub mod export;
pub mod import;
pub mod integration;
pub mod money;
pub mod persistence;
pub mod posting;
pub mod transaction;

pub use account::AccountPath;
pub use currency::Currency;
pub use engine::{LedgerEngine, PersistenceLayer, TransactionFilter};
pub use error::{LedgerError, Result};
pub use export::LedgerExporter;
pub use import::LedgerImporter;
pub use persistence::SqlitePersistence;
pub use integration::{
    record_box_spread, record_box_spread_expiration, record_box_spread_safe,
    record_cash_flow, record_position_change, record_position_change_safe, record_position_close,
    record_transaction_safe,
};
pub use money::Money;
pub use posting::{Cost, Posting};
pub use transaction::{Transaction, TransactionBuilder};

// Re-export for convenience
pub use uuid::Uuid;
