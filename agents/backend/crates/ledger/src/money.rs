use crate::currency::Currency;
use crate::error::{LedgerError, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// Money represents an amount in a specific currency
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    /// Amount (always positive, sign handled by posting direction)
    pub amount: Decimal,
    /// Currency code
    pub currency: Currency,
}

impl Money {
    /// Create zero money in USD
    pub fn zero() -> Self {
        Self {
            amount: Decimal::ZERO,
            currency: Currency::USD,
        }
    }

    /// Create zero money in specified currency
    pub fn zero_with_currency(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// Create new money amount
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Create money from f64 (for convenience, converts to Decimal)
    pub fn from_f64(amount: f64, currency: Currency) -> Result<Self> {
        use rust_decimal::prelude::*;
        Decimal::try_from(amount)
            .map(|d| Self::new(d, currency))
            .map_err(|_| LedgerError::InvalidDecimal(format!("Invalid f64: {}", amount)))
    }

    /// Get absolute value
    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency,
        }
    }

    /// Check if amount is zero
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// Check if amount is positive
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive()
    }

    /// Check if amount is negative
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative()
    }
}

impl Add for Money {
    type Output = Result<Money>;

    fn add(self, other: Money) -> Result<Money> {
        if self.currency != other.currency {
            return Err(LedgerError::CurrencyMismatch {
                expected: self.currency,
                actual: other.currency,
            });
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }
}

impl Sub for Money {
    type Output = Result<Money>;

    fn sub(self, other: Money) -> Result<Money> {
        if self.currency != other.currency {
            return Err(LedgerError::CurrencyMismatch {
                expected: self.currency,
                actual: other.currency,
            });
        }
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.currency.code(), self.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_creation() {
        let money = Money::new(Decimal::from(100), Currency::USD);
        assert_eq!(money.amount, Decimal::from(100));
        assert_eq!(money.currency, Currency::USD);
    }

    #[test]
    fn test_money_from_f64() {
        let money = Money::from_f64(100.50, Currency::USD).unwrap();
        assert_eq!(money.currency, Currency::USD);
        // Decimal precision means we check the amount is close
        use rust_decimal::prelude::*;
        let expected = Decimal::try_from(100.50).unwrap();
        assert!((money.amount - expected).abs() < Decimal::new(1, 2)); // 0.01 tolerance
    }

    #[test]
    fn test_money_add() {
        let m1 = Money::new(Decimal::from(100), Currency::USD);
        let m2 = Money::new(Decimal::from(50), Currency::USD);
        let sum = (m1 + m2).unwrap();
        assert_eq!(sum.amount, Decimal::from(150));
    }

    #[test]
    fn test_money_add_currency_mismatch() {
        let m1 = Money::new(Decimal::from(100), Currency::USD);
        let m2 = Money::new(Decimal::from(50), Currency::ILS);
        assert!(m1.add(m2).is_err());
    }

    #[test]
    fn test_money_sub() {
        let m1 = Money::new(Decimal::from(100), Currency::USD);
        let m2 = Money::new(Decimal::from(30), Currency::USD);
        let diff = (m1 - m2).unwrap();
        assert_eq!(diff.amount, Decimal::from(70));
    }

    #[test]
    fn test_money_abs() {
        let money = Money::new(Decimal::from(-100), Currency::USD);
        let abs = money.abs();
        assert_eq!(abs.amount, Decimal::from(100));
    }
}
