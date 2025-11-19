use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Currency enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    /// US Dollar
    USD,
    /// Israeli Shekel
    ILS,
    /// Euro
    EUR,
    /// British Pound
    GBP,
}

impl Currency {
    /// Get currency code string
    pub fn code(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::ILS => "ILS",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
        }
    }

    /// Parse currency from code string
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "USD" => Some(Currency::USD),
            "ILS" => Some(Currency::ILS),
            "EUR" => Some(Currency::EUR),
            "GBP" => Some(Currency::GBP),
            _ => None,
        }
    }
}

impl FromStr for Currency {
    type Err = crate::error::LedgerError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Currency::from_code(s)
            .ok_or_else(|| crate::error::LedgerError::InvalidCurrency(s.to_string()))
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_code() {
        assert_eq!(Currency::USD.code(), "USD");
        assert_eq!(Currency::ILS.code(), "ILS");
    }

    #[test]
    fn test_currency_from_code() {
        assert_eq!(Currency::from_code("USD"), Some(Currency::USD));
        assert_eq!(Currency::from_code("usd"), Some(Currency::USD));
        assert_eq!(Currency::from_code("ILS"), Some(Currency::ILS));
        assert_eq!(Currency::from_code("INVALID"), None);
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!("USD".parse::<Currency>().unwrap(), Currency::USD);
        assert!("INVALID".parse::<Currency>().is_err());
    }
}
