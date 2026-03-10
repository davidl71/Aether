//! Main parser for Discount Bank reconciliation files

use crate::encoding::decode_to_utf8;
use crate::errors::{ParseError, Result};
use crate::records::{HeaderRecord, SummaryRecord, TransactionRecord};
use std::path::Path;
use tracing::{debug, warn};

/// Discount Bank file parser
pub struct DiscountBankParser;

impl DiscountBankParser {
    /// Parse file from path
    pub async fn parse_file(file_path: &Path) -> Result<ParsedFile> {
        let bytes = tokio::fs::read(file_path).await?;
        Self::parse_bytes(&bytes).await
    }

    /// Parse file from bytes
    pub async fn parse_bytes(bytes: &[u8]) -> Result<ParsedFile> {
        // Decode to UTF-8 (handles Windows-1255 Hebrew encoding)
        let content = decode_to_utf8(bytes)?;

        // Parse line by line
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim_end_matches("\r\n").trim_end_matches('\n'))
            .collect();

        let mut headers = Vec::new();
        let mut transactions = Vec::new();
        let mut summary: Option<SummaryRecord> = None;

        for (line_num, line) in lines.iter().enumerate() {
            let line_number = line_num + 1;
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Determine record type by first 2 characters
            if trimmed.len() < 2 {
                warn!(line = line_number, "Line too short, skipping");
                continue;
            }

            let record_code = &trimmed[0..2];

            match record_code {
                "00" => match HeaderRecord::from_line(trimmed, line_number) {
                    Ok(header) => {
                        debug!(line = line_number, "Parsed header record");
                        headers.push(header);
                    }
                    Err(e) => {
                        warn!(line = line_number, error = %e, "Failed to parse header record");
                    }
                },
                "01" => match TransactionRecord::from_line(trimmed, line_number) {
                    Ok(txn) => {
                        debug!(
                            line = line_number,
                            amount_raw = %txn.amount,
                            "Parsed transaction record"
                        );
                        transactions.push(txn);
                    }
                    Err(e) => {
                        warn!(line = line_number, error = %e, "Failed to parse transaction record");
                    }
                },
                "04" => match SummaryRecord::from_line(trimmed, line_number) {
                    Ok(summ) => {
                        debug!(line = line_number, "Parsed summary record");
                        summary = Some(summ);
                    }
                    Err(e) => {
                        warn!(line = line_number, error = %e, "Failed to parse summary record");
                    }
                },
                _ => {
                    warn!(
                        line = line_number,
                        record_code = record_code,
                        "Unknown record code, skipping"
                    );
                }
            }
        }

        debug!(
            headers = headers.len(),
            transactions = transactions.len(),
            has_summary = summary.is_some(),
            "Parsed Discount Bank file"
        );

        Ok(ParsedFile {
            headers,
            transactions,
            summary,
        })
    }
}

/// Parsed file structure
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub headers: Vec<HeaderRecord>,
    pub transactions: Vec<TransactionRecord>,
    pub summary: Option<SummaryRecord>,
}

impl ParsedFile {
    /// Get account number from first header (if available)
    pub fn account_number(&self) -> Option<&str> {
        self.headers.first().map(|h| h.account_number.as_str())
    }

    /// Get currency code from first header (if available)
    pub fn currency_code(&self) -> Option<&str> {
        self.headers.first().map(|h| h.currency_code.as_str())
    }

    /// Get transaction count from summary (if available)
    pub fn transaction_count(&self) -> Option<u64> {
        self.summary.as_ref().map(|s| s.transaction_count)
    }

    /// Validate parsed file (check summary matches transaction count)
    pub fn validate(&self) -> Result<()> {
        if let Some(ref summary) = self.summary {
            let actual_count = self.transactions.len() as u64;
            if summary.transaction_count != actual_count {
                return Err(ParseError::ParseError {
                    line: 0,
                    message: format!(
                        "Transaction count mismatch: expected {}, got {}",
                        summary.transaction_count, actual_count
                    ),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_file() {
        let content = concat!(
            "00123456701234567000000000123456 00000000789012 251118\r\n",
            "01251118000000123456 1234567\r\n",
            "0412345678901234567800000000001",
            "0000000000000000000000000000000000000000000000000000000000000000000000000000\r\n"
        )
        .as_bytes();

        let parsed = DiscountBankParser::parse_bytes(content).await.unwrap();
        assert_eq!(parsed.headers.len(), 1);
        assert_eq!(parsed.transactions.len(), 1);
        assert!(parsed.summary.is_some());
    }
}
