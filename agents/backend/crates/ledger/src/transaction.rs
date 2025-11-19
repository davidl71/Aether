use crate::account::AccountPath;
use crate::error::{LedgerError, Result};
use crate::posting::Posting;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Transaction represents a complete financial operation
/// with multiple postings (debits and credits) that must balance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: Uuid,
    /// Transaction date and time
    pub date: DateTime<Utc>,
    /// Transaction description (e.g., "Buy SPY", "Box Spread: SPY 450/460")
    pub description: String,
    /// Transaction cleared status (true = cleared, false = pending)
    pub cleared: bool,
    /// Multiple postings (debits and credits) - must balance
    pub postings: Vec<Posting>,
    /// Transaction metadata (trade_id, strategy, etc.)
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Transaction {
    /// Create new transaction
    pub fn new(
        description: impl Into<String>,
        postings: Vec<Posting>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            date: Utc::now(),
            description: description.into(),
            cleared: true,
            postings,
            metadata,
        }
    }

    /// Validate that transaction balances (sum of debits = sum of credits)
    pub fn validate_balance(&self) -> Result<()> {
        if self.postings.is_empty() {
            return Err(LedgerError::UnbalancedTransaction {
                debits: crate::Money::zero(),
                credits: crate::Money::zero(),
                difference: crate::Money::zero(),
            });
        }

        // Group postings by currency
        let mut totals_by_currency: HashMap<crate::Currency, (crate::Money, crate::Money)> =
            HashMap::new();

        for posting in &self.postings {
            let currency = posting.amount.currency;
            let entry = totals_by_currency.entry(currency).or_insert_with(|| {
                (
                    crate::Money::zero_with_currency(currency),
                    crate::Money::zero_with_currency(currency),
                )
            });

            if posting.amount.is_positive() {
                entry.0 = (entry.0.clone() + posting.amount.clone())?;
            } else {
                entry.1 = (entry.1.clone() + posting.amount.clone().abs())?;
            }
        }

        // Check each currency balances
        for (_currency, (debits, credits)) in totals_by_currency {
            let difference = (debits.clone() - credits.clone())?.abs();
            // Allow small floating-point differences (0.01)
            if difference.amount > rust_decimal::Decimal::new(1, 2) {
                return Err(LedgerError::UnbalancedTransaction {
                    debits,
                    credits,
                    difference,
                });
            }
        }

        Ok(())
    }

    /// Get transaction total (for single-currency transactions)
    pub fn total(&self) -> Result<Option<crate::Money>> {
        if self.postings.is_empty() {
            return Ok(None);
        }

        let mut total = crate::Money::zero_with_currency(self.postings[0].amount.currency);
        for posting in &self.postings {
            total = (total + posting.amount.clone())?;
        }

        Ok(Some(total.abs()))
    }
}

/// Transaction builder for constructing transactions with validation
pub struct TransactionBuilder {
    date: DateTime<Utc>,
    description: String,
    cleared: bool,
    postings: Vec<Posting>,
    metadata: HashMap<String, String>,
}

impl TransactionBuilder {
    /// Create new transaction builder
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            date: Utc::now(),
            description: description.into(),
            cleared: true,
            postings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set transaction date
    pub fn with_date(mut self, date: DateTime<Utc>) -> Self {
        self.date = date;
        self
    }

    /// Set cleared status
    pub fn cleared(mut self, cleared: bool) -> Self {
        self.cleared = cleared;
        self
    }

    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Add posting
    pub fn add_posting(mut self, posting: Posting) -> Self {
        self.postings.push(posting);
        self
    }

    /// Add debit posting (positive amount)
    pub fn debit(mut self, account: impl Into<AccountPath>, amount: crate::Money) -> Self {
        self.postings.push(Posting::new(account.into(), amount));
        self
    }

    /// Add credit posting (negative amount)
    pub fn credit(mut self, account: impl Into<AccountPath>, amount: crate::Money) -> Self {
        let mut credit_amount = amount;
        credit_amount.amount = -credit_amount.amount;
        self.postings.push(Posting::new(account.into(), credit_amount));
        self
    }

    /// Build transaction with validation
    pub fn build(self) -> Result<Transaction> {
        let transaction = Transaction {
            id: Uuid::new_v4(),
            date: self.date,
            description: self.description,
            cleared: self.cleared,
            postings: self.postings,
            metadata: self.metadata,
        };

        transaction.validate_balance()?;
        Ok(transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;
    use crate::money::Money;
    use rust_decimal::Decimal;

    #[test]
    fn test_transaction_creation() {
        let account1 = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let account2 = AccountPath::from_string("Equity:Capital").unwrap();
        let amount = Money::new(Decimal::from(100), Currency::USD);

        let transaction = TransactionBuilder::new("Initial deposit")
            .debit(account1, amount.clone())
            .credit(account2, amount)
            .build()
            .unwrap();

        assert_eq!(transaction.postings.len(), 2);
        assert_eq!(transaction.description, "Initial deposit");
    }

    #[test]
    fn test_transaction_validation_balanced() {
        let account1 = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let account2 = AccountPath::from_string("Equity:Capital").unwrap();
        let amount = Money::new(Decimal::from(100), Currency::USD);

        let transaction = TransactionBuilder::new("Balanced transaction")
            .debit(account1, amount.clone())
            .credit(account2, amount)
            .build();

        assert!(transaction.is_ok());
        assert!(transaction.unwrap().validate_balance().is_ok());
    }

    #[test]
    fn test_transaction_validation_unbalanced() {
        let account1 = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let account2 = AccountPath::from_string("Equity:Capital").unwrap();
        let amount1 = Money::new(Decimal::from(100), Currency::USD);
        let amount2 = Money::new(Decimal::from(50), Currency::USD);

        let transaction = TransactionBuilder::new("Unbalanced transaction")
            .debit(account1, amount1)
            .credit(account2, amount2)
            .build();

        assert!(transaction.is_err());
    }

    #[test]
    fn test_transaction_with_metadata() {
        let account1 = AccountPath::from_string("Assets:IBKR:SPY").unwrap();
        let account2 = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let amount = Money::new(Decimal::from(45000), Currency::USD);

        let transaction = TransactionBuilder::new("Buy SPY")
            .with_metadata("trade_id", "ORD-12345")
            .with_metadata("strategy", "box_spread")
            .debit(account1, amount.clone())
            .credit(account2, amount)
            .build()
            .unwrap();

        assert_eq!(transaction.metadata.get("trade_id"), Some(&"ORD-12345".to_string()));
        assert_eq!(transaction.metadata.get("strategy"), Some(&"box_spread".to_string()));
    }
}
