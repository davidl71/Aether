//! Broker error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("not connected")]
    NotConnected,
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("order failed: {0}")]
    OrderFailed(String),
    #[error("contract error: {0}")]
    ContractError(String),
    #[error("timeout")]
    Timeout,
    #[error("other: {0}")]
    Other(String),
}
