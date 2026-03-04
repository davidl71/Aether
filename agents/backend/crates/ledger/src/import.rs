//! Ledger CLI import functionality
//!
//! Parses existing `.ledger` files in Ledger CLI format and converts
//! them to Transaction format for integration with the ledger system.

use crate::account::AccountPath;
use crate::currency::Currency;
use crate::error::{LedgerError, Result};
use crate::money::Money;
use crate::posting::{Cost, Posting};
use crate::transaction::Transaction;
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, warn};

/// Ledger CLI file parser
pub struct LedgerImporter;

impl LedgerImporter {
    /// Import transactions from Ledger CLI format string
    pub fn import_from_string(content: &str) -> Result<Vec<Transaction>> {
        let mut transactions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut line_index = 0;

        while line_index < lines.len() {
            match Self::parse_transaction(&lines, &mut line_index) {
                Ok(Some(transaction)) => {
                    transactions.push(transaction);
                }
                Ok(None) => {
                    // Empty line or comment, skip
                    line_index += 1;
                }
                Err(e) => {
                    warn!(line = line_index + 1, error = %e, "Failed to parse transaction, skipping");
                    line_index += 1;
                }
            }
        }

        debug!(
            count = transactions.len(),
            "Imported transactions from Ledger CLI format"
        );
        Ok(transactions)
    }

    /// Import transactions from file
    pub async fn import_from_file(file_path: &std::path::Path) -> Result<Vec<Transaction>> {
        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| LedgerError::Persistence(anyhow::anyhow!("Failed to read file: {}", e)))?;

        Self::import_from_string(&content)
    }

    /// Parse a single transaction from lines vector
    fn parse_transaction(lines: &[&str], line_index: &mut usize) -> Result<Option<Transaction>> {
        // Skip empty lines and comments
        while *line_index < lines.len() {
            let trimmed = lines[*line_index].trim();
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                *line_index += 1;
            } else {
                break;
            }
        }

        if *line_index >= lines.len() {
            return Ok(None);
        }

        let header_line = lines[*line_index].trim();
        *line_index += 1;

        // Parse transaction header: YYYY/MM/DD [*!] Description
        let (date, cleared, description) = Self::parse_header(header_line)?;

        let mut postings = Vec::new();
        let mut metadata = HashMap::new();

        // Parse postings and metadata
        while *line_index < lines.len() {
            let trimmed = lines[*line_index].trim();

            if trimmed.is_empty() {
                *line_index += 1;
                break; // End of transaction
            }

            if trimmed.starts_with(';') {
                // Metadata comment: ; key: value
                let (key, value) = Self::parse_metadata(trimmed)?;
                metadata.insert(key, value);
                *line_index += 1;
            } else if trimmed
                .chars()
                .next()
                .map(|c| c.is_alphanumeric() || c == ':')
                .unwrap_or(false)
            {
                // Posting line: Account    Amount
                let posting = Self::parse_posting(trimmed)?;
                postings.push(posting);
                *line_index += 1;
            } else {
                // Unknown line, skip
                warn!(line = *line_index + 1, content = %trimmed, "Unknown line format, skipping");
                *line_index += 1;
            }
        }

        // Build transaction
        let transaction = Transaction {
            id: uuid::Uuid::new_v4(), // Generate new ID since not in Ledger CLI format
            date,
            description: description.to_string(),
            cleared,
            postings,
            metadata,
        };

        // Validate transaction balances
        transaction.validate_balance()?;

        Ok(Some(transaction))
    }

    /// Parse transaction header: YYYY/MM/DD [*!] Description
    fn parse_header(line: &str) -> Result<(DateTime<Utc>, bool, String)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err(LedgerError::InvalidDecimal(
                "Empty transaction header".to_string(),
            ));
        }

        // Parse date: YYYY/MM/DD
        let date_str = parts[0];
        let date = Self::parse_date(date_str)?;

        // Parse cleared status (optional): * or !
        let (cleared, desc_start) = if parts.len() > 1 {
            match parts[1] {
                "*" => (true, 2),
                "!" => (false, 2),
                _ => (true, 1), // Default to cleared if no status marker
            }
        } else {
            (true, 1)
        };

        // Parse description (rest of the line)
        let description = if desc_start < parts.len() {
            parts[desc_start..].join(" ")
        } else {
            "".to_string()
        };

        Ok((date, cleared, description))
    }

    /// Parse date: YYYY/MM/DD
    fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() != 3 {
            return Err(LedgerError::InvalidDecimal(format!(
                "Invalid date format: {}",
                date_str
            )));
        }

        let year: i32 = parts[0]
            .parse()
            .map_err(|_| LedgerError::InvalidDecimal(format!("Invalid year: {}", parts[0])))?;
        let month: u32 = parts[1]
            .parse()
            .map_err(|_| LedgerError::InvalidDecimal(format!("Invalid month: {}", parts[1])))?;
        let day: u32 = parts[2]
            .parse()
            .map_err(|_| LedgerError::InvalidDecimal(format!("Invalid day: {}", parts[2])))?;

        Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
            .single()
            .ok_or_else(|| LedgerError::InvalidDecimal(format!("Invalid date: {}", date_str)))
    }

    /// Parse posting: Account    Amount [or Quantity Symbol @ Price]
    fn parse_posting(line: &str) -> Result<Posting> {
        // Split on whitespace (multiple spaces)
        // Ledger format uses multiple spaces as separator between account and amount
        // So we split on 2+ spaces or use the last few fields as amount
        let trimmed = line.trim();

        // Find the boundary between account and amount (2+ spaces)
        let account_end = trimmed.find("  ").ok_or_else(|| {
            LedgerError::InvalidDecimal("No separator found in posting line".to_string())
        })?;

        let account_str = trimmed[..account_end].trim();
        let amount_str = trimmed[account_end..].trim_start();

        let account = AccountPath::from_string(account_str)
            .map_err(|e| LedgerError::InvalidAccountPath(format!("{}: {:?}", account_str, e)))?;

        // Parse amount
        let (amount, cost) = Self::parse_amount(amount_str)?;

        Ok(Posting {
            account,
            amount,
            cost,
            metadata: HashMap::new(),
        })
    }

    /// Parse amount string: $123.45 or -$123.45 or 100 SPY @ $450.00
    fn parse_amount(amount_str: &str) -> Result<(Money, Option<Cost>)> {
        let trimmed = amount_str.trim();

        // Check if it's a cost basis format: Quantity Symbol @ Price
        if trimmed.contains('@') {
            return Self::parse_cost_basis(trimmed);
        }

        // Parse simple amount: $123.45 or -$123.45 or USD 123.45
        let is_negative = trimmed.starts_with('-');
        let positive_str = if is_negative {
            trimmed.strip_prefix('-').unwrap_or(trimmed)
        } else {
            trimmed
        }
        .trim();

        // Parse currency and amount
        let (currency, amount_str) = if positive_str.starts_with('$') {
            (
                Currency::USD,
                positive_str.strip_prefix('$').unwrap_or(positive_str),
            )
        } else if positive_str.starts_with("USD ") {
            (
                Currency::USD,
                positive_str.strip_prefix("USD ").unwrap_or(positive_str),
            )
        } else if positive_str.starts_with("ILS ") {
            (
                Currency::ILS,
                positive_str.strip_prefix("ILS ").unwrap_or(positive_str),
            )
        } else if positive_str.starts_with("EUR ") {
            (
                Currency::EUR,
                positive_str.strip_prefix("EUR ").unwrap_or(positive_str),
            )
        } else if positive_str.starts_with("GBP ") {
            (
                Currency::GBP,
                positive_str.strip_prefix("GBP ").unwrap_or(positive_str),
            )
        } else {
            // Default to USD if no currency specified
            (Currency::USD, positive_str)
        };

        let amount = Decimal::from_str(amount_str)
            .map_err(|_| LedgerError::InvalidDecimal(format!("Invalid amount: {}", amount_str)))?;

        let mut money = Money::new(amount, currency);
        if is_negative {
            money.amount = -money.amount;
        }

        Ok((money, None))
    }

    /// Parse cost basis format: Quantity Symbol @ Price
    fn parse_cost_basis(amount_str: &str) -> Result<(Money, Option<Cost>)> {
        let parts: Vec<&str> = amount_str.split('@').collect();
        if parts.len() != 2 {
            return Err(LedgerError::InvalidDecimal(format!(
                "Invalid cost basis format: {}",
                amount_str
            )));
        }

        let quantity_and_symbol = parts[0].trim();
        let price_str = parts[1].trim();

        // Parse quantity (first word before symbol)
        // Format: "100 SPY" -> quantity = 100
        let quantity_parts: Vec<&str> = quantity_and_symbol.split_whitespace().collect();
        if quantity_parts.is_empty() {
            return Err(LedgerError::InvalidDecimal(format!(
                "Invalid quantity format: {}",
                quantity_and_symbol
            )));
        }

        let quantity_str = quantity_parts[0];
        let quantity = Decimal::from_str(quantity_str).map_err(|_| {
            LedgerError::InvalidDecimal(format!("Invalid quantity: {}", quantity_str))
        })?;

        // Parse price
        let (price_currency, price_amount_str) = if price_str.starts_with('$') {
            (
                Currency::USD,
                price_str.strip_prefix('$').unwrap_or(price_str),
            )
        } else if price_str.starts_with("USD ") {
            (
                Currency::USD,
                price_str.strip_prefix("USD ").unwrap_or(price_str),
            )
        } else {
            (Currency::USD, price_str)
        };

        let price_amount = Decimal::from_str(price_amount_str).map_err(|_| {
            LedgerError::InvalidDecimal(format!("Invalid price: {}", price_amount_str))
        })?;

        let price = Money::new(price_amount, price_currency);
        let cost = Cost::new(quantity, price.clone());

        // Calculate total amount
        let total_amount = quantity * price_amount;
        let total_money = Money::new(total_amount, price_currency);

        Ok((total_money, Some(cost)))
    }

    /// Parse metadata comment: ; key: value
    fn parse_metadata(line: &str) -> Result<(String, String)> {
        let line = line.strip_prefix(';').unwrap_or(line).trim();
        let parts: Vec<&str> = line.splitn(2, ':').collect();

        if parts.len() != 2 {
            return Err(LedgerError::InvalidDecimal(format!(
                "Invalid metadata format: {}",
                line
            )));
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        Ok((key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;

    #[test]
    fn test_import_simple_transaction() {
        let content = r#"
2025/11/18 * Buy SPY
    Assets:IBKR:SPY            $45000.00
    Assets:IBKR:Cash           -$45000.00
    ; trade_id: ORD-12345
"#;

        let transactions = LedgerImporter::import_from_string(content).unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].description, "Buy SPY");
        assert_eq!(transactions[0].postings.len(), 2);
        assert_eq!(
            transactions[0].metadata.get("trade_id"),
            Some(&"ORD-12345".to_string())
        );
    }

    #[test]
    fn test_import_with_cost_basis() {
        let content = r#"
2025/11/18 * Buy SPY
    Assets:IBKR:SPY            100 SPY @ $450.00
    Assets:IBKR:Cash           -$45000.00
"#;

        let transactions = LedgerImporter::import_from_string(content).unwrap();
        assert_eq!(transactions.len(), 1);
        let posting = &transactions[0].postings[0];
        assert!(posting.cost.is_some());
        if let Some(ref cost) = posting.cost {
            assert_eq!(cost.quantity, Decimal::from(100));
            assert_eq!(cost.price.amount, Decimal::from(450));
        }
    }

    #[test]
    fn test_import_box_spread() {
        let content = r#"
2025/11/18 * Box Spread: SPY 450/460 20251219
    Assets:IBKR:BoxSpread:SPY:450:460:20251219    $1000.00
    Assets:IBKR:Cash                              -$1000.00
    ; trade_id: BOX-12345
    ; strategy: box_spread
    ; net_debit: 1000.0
"#;

        let transactions = LedgerImporter::import_from_string(content).unwrap();
        assert_eq!(transactions.len(), 1);
        assert!(transactions[0].description.contains("Box Spread"));
        assert_eq!(
            transactions[0].metadata.get("strategy"),
            Some(&"box_spread".to_string())
        );
    }

    #[test]
    fn test_import_multiple_transactions() {
        let content = r#"
2025/11/18 * Transaction 1
    Assets:IBKR:Cash            $100.00
    Equity:Capital              -$100.00

2025/11/19 * Transaction 2
    Assets:IBKR:Cash            $200.00
    Equity:Capital              -$200.00
"#;

        let transactions = LedgerImporter::import_from_string(content).unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_import_pending_transaction() {
        let content = r#"
2025/11/18 ! Pending Transaction
    Assets:IBKR:Cash            $100.00
    Equity:Capital              -$100.00
"#;

        let transactions = LedgerImporter::import_from_string(content).unwrap();
        assert_eq!(transactions.len(), 1);
        assert!(!transactions[0].cleared);
    }

    #[test]
    fn test_parse_date() {
        use chrono::Datelike;
        let date = LedgerImporter::parse_date("2025/11/18").unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 18);
    }

    #[test]
    fn test_parse_amount_negative() {
        let (money, cost) = LedgerImporter::parse_amount("-$123.45").unwrap();
        assert!(money.is_negative());
        assert_eq!(money.amount.abs(), Decimal::from_str("123.45").unwrap());
        assert!(cost.is_none());
    }

    #[test]
    fn test_parse_amount_currency() {
        let (money, _) = LedgerImporter::parse_amount("ILS 1000.00").unwrap();
        assert_eq!(money.currency, Currency::ILS);
        assert_eq!(money.amount, Decimal::from(1000));
    }

    #[test]
    fn test_parse_cost_basis() {
        let (money, cost) = LedgerImporter::parse_amount("100 SPY @ $450.00").unwrap();
        assert_eq!(money.amount, Decimal::from(45000));
        assert!(cost.is_some());
        if let Some(ref cost) = cost {
            assert_eq!(cost.quantity, Decimal::from(100));
            assert_eq!(cost.price.amount, Decimal::from(450));
        }
    }

    #[tokio::test]
    async fn test_import_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"
2025/11/18 * Test Transaction
    Assets:IBKR:Cash            $100.00
    Equity:Capital              -$100.00
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        let path = file.path();

        let transactions = LedgerImporter::import_from_file(path).await.unwrap();
        assert_eq!(transactions.len(), 1);
    }
}
