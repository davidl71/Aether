use thiserror::Error;

/// Ledger operation result type
pub type Result<T> = std::result::Result<T, LedgerError>;

/// Ledger operation errors
#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Unbalanced transaction: debits {debits:?} != credits {credits:?}, difference: {difference:?}")]
    UnbalancedTransaction {
        debits: crate::Money,
        credits: crate::Money,
        difference: crate::Money,
    },

    #[error("Invalid account path: {0}")]
    InvalidAccountPath(String),

    #[error("Currency mismatch: expected {expected:?}, got {actual:?}")]
    CurrencyMismatch {
        expected: crate::Currency,
        actual: crate::Currency,
    },

    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),

    #[error("Invalid decimal conversion: {0}")]
    InvalidDecimal(String),

    #[error("Persistence error: {0}")]
    Persistence(#[from] anyhow::Error),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(crate::Uuid),

    #[error("Account not found: {0}")]
    AccountNotFound(crate::AccountPath),
}
