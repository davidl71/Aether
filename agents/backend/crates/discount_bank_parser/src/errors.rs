use thiserror::Error;

/// Parser result type
pub type Result<T> = std::result::Result<T, ParseError>;

/// Parser errors
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid record code: expected {expected:?}, got {actual:?} at line {line}")]
    InvalidRecordCode {
        expected: String,
        actual: String,
        line: usize,
    },

    #[error("Invalid record length: expected {expected}, got {actual} at line {line}")]
    InvalidRecordLength {
        expected: usize,
        actual: usize,
        line: usize,
    },

    #[error("Failed to parse date: {0} at line {1}")]
    InvalidDate(String, usize),

    #[error("Failed to parse amount: {0} at line {1}")]
    InvalidAmount(String, usize),

    #[error("Failed to parse currency code: {0} at line {1}")]
    InvalidCurrencyCode(String, usize),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
}
