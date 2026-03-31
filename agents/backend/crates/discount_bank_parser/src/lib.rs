//! Discount Bank Reconciliation File Parser
//!
//! Parses fixed-width text files from Discount Bank's Osh Matching service
//! and converts them to ledger transactions.

pub mod encoding;
pub mod errors;
pub mod csv_import;
pub mod ledger_integration;
pub mod parser;
pub mod records;

pub use errors::{ParseError, Result};
pub use csv_import::{convert_csv_to_transactions, DiscountBankCsvTransactionRow};
pub use ledger_integration::{convert_single_transaction, convert_to_transactions};
pub use parser::{DiscountBankParser, ParsedFile};
pub use records::{HeaderRecord, SummaryRecord, TransactionRecord};
