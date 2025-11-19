use crate::account::AccountPath;
use crate::error::Result;
use crate::money::Money;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost basis for investment tracking (e.g., "100 SPY @ $450.00")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cost {
    /// Quantity (e.g., 100 shares)
    pub quantity: Decimal,
    /// Unit price (e.g., $450.00 per share)
    pub price: Money,
}

impl Cost {
    /// Create new cost basis
    pub fn new(quantity: Decimal, price: Money) -> Self {
        Self { quantity, price }
    }

    /// Calculate total cost
    pub fn total(&self) -> Result<Money> {
        let total_amount = self.quantity * self.price.amount;
        Ok(Money::new(total_amount, self.price.currency))
    }
}

/// Posting represents one side (debit or credit) of a transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Posting {
    /// Account path (e.g., "Assets:IBKR:SPY", "Assets:IBKR:Cash")
    pub account: AccountPath,
    /// Amount and currency (positive = debit, negative = credit)
    pub amount: Money,
    /// Cost basis for investment tracking (e.g., "100 SPY @ $450.00")
    pub cost: Option<Cost>,
    /// Posting metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Posting {
    /// Create new posting
    pub fn new(account: AccountPath, amount: Money) -> Self {
        Self {
            account,
            amount,
            cost: None,
            metadata: HashMap::new(),
        }
    }

    /// Create posting with cost basis
    pub fn with_cost(account: AccountPath, amount: Money, cost: Cost) -> Self {
        Self {
            account,
            amount,
            cost: Some(cost),
            metadata: HashMap::new(),
        }
    }

    /// Create posting with metadata
    pub fn with_metadata(
        account: AccountPath,
        amount: Money,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            account,
            amount,
            cost: None,
            metadata,
        }
    }

    /// Check if posting is a debit (positive amount)
    pub fn is_debit(&self) -> bool {
        self.amount.is_positive()
    }

    /// Check if posting is a credit (negative amount)
    pub fn is_credit(&self) -> bool {
        self.amount.is_negative()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;

    #[test]
    fn test_posting_creation() {
        let account = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let money = Money::new(Decimal::from(100), Currency::USD);
        let posting = Posting::new(account, money);
        assert!(posting.is_debit());
        assert_eq!(posting.amount.amount, Decimal::from(100));
    }

    #[test]
    fn test_posting_credit() {
        let account = AccountPath::from_string("Assets:IBKR:Cash").unwrap();
        let money = Money::new(Decimal::from(-100), Currency::USD);
        let posting = Posting::new(account, money);
        assert!(posting.is_credit());
        assert_eq!(posting.amount.amount, Decimal::from(-100));
    }

    #[test]
    fn test_cost_total() {
        let price = Money::new(Decimal::from(450), Currency::USD);
        let cost = Cost::new(Decimal::from(100), price);
        let total = cost.total().unwrap();
        assert_eq!(total.amount, Decimal::from(45000));
    }
}
