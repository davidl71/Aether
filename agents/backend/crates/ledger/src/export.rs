//! Ledger CLI export functionality
//!
//! Exports transactions to Ledger CLI-compatible `.ledger` file format
//! for reconciliation and audit trail.

use crate::transaction::Transaction;
use chrono::{DateTime, Utc};
use std::fmt::Write;

/// Export transactions to Ledger CLI format
pub struct LedgerExporter;

impl LedgerExporter {
    /// Export transactions to Ledger CLI format string
    pub fn export_transactions(transactions: &[Transaction]) -> String {
        let mut output = String::new();

        // Sort transactions by date
        let mut sorted = transactions.to_vec();
        sorted.sort_by(|a, b| a.date.cmp(&b.date));

        for transaction in sorted {
            Self::format_transaction(&mut output, &transaction);
            output.push('\n');
        }

        output
    }

    /// Format a single transaction in Ledger CLI format
    fn format_transaction(output: &mut String, transaction: &Transaction) {
        // Date and cleared status
        let date_str = Self::format_date(&transaction.date);
        let cleared = if transaction.cleared { "*" } else { "!" };
        writeln!(output, "{} {} {}", date_str, cleared, transaction.description).unwrap();

        // Postings
        for posting in &transaction.postings {
            let amount_str = Self::format_posting_amount(posting, &transaction.postings);
            writeln!(output, "    {:40} {}", posting.account.to_string(), amount_str).unwrap();
        }

        // Metadata as comments
        for (key, value) in &transaction.metadata {
            writeln!(output, "    ; {}: {}", key, value).unwrap();
        }
    }

    /// Format date in Ledger CLI format (YYYY/MM/DD)
    fn format_date(date: &DateTime<Utc>) -> String {
        date.format("%Y/%m/%d").to_string()
    }

    /// Format posting amount with cost basis if present
    fn format_posting_amount(posting: &crate::posting::Posting, _all_postings: &[crate::posting::Posting]) -> String {
        let mut result = String::new();

        // Add cost basis if present (e.g., "100 SPY @ $450.00")
        if let Some(ref cost) = posting.cost {
            write!(result, "{} ", cost.quantity).unwrap();
            // Get symbol from account path (last segment)
            let account_name = posting.account.name();
            write!(result, "{} @ ", account_name).unwrap();
        }

        // Format amount with currency sign
        let amount = posting.amount.abs();
        let sign = if posting.amount.is_positive() { "" } else { "-" };

        // Use currency code or $ for USD
        let currency_symbol = match amount.currency {
            crate::Currency::USD => "$",
            crate::Currency::ILS => "ILS ",
            crate::Currency::EUR => "EUR ",
            crate::Currency::GBP => "GBP ",
        };

        write!(result, "{}{}{:.2}", sign, currency_symbol, amount.amount).unwrap();

        result
    }

    /// Export transactions to file
    pub async fn export_to_file(
        transactions: &[Transaction],
        file_path: &std::path::Path,
    ) -> Result<(), crate::error::LedgerError> {
        let content = Self::export_transactions(transactions);
        tokio::fs::write(file_path, content)
            .await
            .map_err(|e| crate::error::LedgerError::Persistence(anyhow::anyhow!("Failed to write file: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::accounts;
    use crate::currency::Currency;
    use crate::money::Money;
    use crate::posting::{Cost, Posting};
    use crate::transaction::TransactionBuilder;
    use rust_decimal::Decimal;

    #[test]
    fn test_export_simple_transaction() {
        let transaction = TransactionBuilder::new("Buy SPY")
            .debit(
                accounts::ibkr_position("SPY"),
                Money::new(Decimal::from(45000), Currency::USD),
            )
            .credit(accounts::ibkr_cash(), Money::new(Decimal::from(45000), Currency::USD))
            .with_metadata("trade_id", "ORD-12345")
            .build()
            .unwrap();

        let exported = LedgerExporter::export_transactions(&[transaction]);
        assert!(exported.contains("Buy SPY"));
        assert!(exported.contains("Assets:IBKR:SPY"));
        assert!(exported.contains("Assets:IBKR:Cash"));
        assert!(exported.contains("trade_id: ORD-12345"));
    }

    #[test]
    fn test_export_with_cost_basis() {
        let mut posting = Posting::new(
            accounts::ibkr_position("SPY"),
            Money::new(Decimal::from(45000), Currency::USD),
        );
        posting.cost = Some(Cost::new(
            Decimal::from(100),
            Money::new(Decimal::from(450), Currency::USD),
        ));

        let transaction = TransactionBuilder::new("Buy SPY")
            .add_posting(posting)
            .credit(accounts::ibkr_cash(), Money::new(Decimal::from(45000), Currency::USD))
            .build()
            .unwrap();

        let exported = LedgerExporter::export_transactions(&[transaction]);
        assert!(exported.contains("100"));
        assert!(exported.contains("@"));
        assert!(exported.contains("SPY"));
    }

    #[test]
    fn test_export_multiple_transactions() {
        let tx1 = TransactionBuilder::new("Transaction 1")
            .debit(accounts::ibkr_cash(), Money::new(Decimal::from(100), Currency::USD))
            .credit(accounts::equity_capital(), Money::new(Decimal::from(100), Currency::USD))
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Transaction 2")
            .debit(accounts::ibkr_cash(), Money::new(Decimal::from(200), Currency::USD))
            .credit(accounts::equity_capital(), Money::new(Decimal::from(200), Currency::USD))
            .build()
            .unwrap();

        let exported = LedgerExporter::export_transactions(&[tx1, tx2]);
        assert!(exported.contains("Transaction 1"));
        assert!(exported.contains("Transaction 2"));
    }
}
