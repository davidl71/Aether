use crate::error::{LedgerError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Account path representing hierarchical account structure
/// Follows Ledger CLI format: "Assets:IBKR:SPY"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountPath {
    segments: Vec<String>,
}

impl AccountPath {
    /// Create new account path from segments
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Parse account path from string (colon-separated)
    pub fn from_string(path: &str) -> Result<Self> {
        if path.is_empty() {
            return Err(LedgerError::InvalidAccountPath(path.to_string()));
        }

        let segments: Vec<String> = path
            .split(':')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if segments.is_empty() {
            return Err(LedgerError::InvalidAccountPath(path.to_string()));
        }

        Ok(Self { segments })
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.segments.join(":")
    }

    /// Get parent account path (if exists)
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() > 1 {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        } else {
            None
        }
    }

    /// Get account segments
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Get account name (last segment)
    pub fn name(&self) -> &str {
        self.segments
            .last()
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}

impl FromStr for AccountPath {
    type Err = crate::error::LedgerError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        AccountPath::from_string(s)
    }
}

impl fmt::Display for AccountPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&str> for AccountPath {
    fn from(s: &str) -> Self {
        AccountPath::from_string(s).unwrap()
    }
}

impl From<String> for AccountPath {
    fn from(s: String) -> Self {
        AccountPath::from_string(&s).unwrap()
    }
}

/// Common account path helpers
pub mod accounts {
    use super::AccountPath;

    pub fn ibkr_cash() -> AccountPath {
        AccountPath::from_string("Assets:IBKR:Cash").unwrap()
    }

    pub fn ibkr_cash_ils() -> AccountPath {
        AccountPath::from_string("Assets:IBKR:Cash:ILS").unwrap()
    }

    pub fn ibkr_position(symbol: &str) -> AccountPath {
        AccountPath::from_string(&format!("Assets:IBKR:{}", symbol)).unwrap()
    }

    pub fn ibkr_box_spread(symbol: &str, strike1: i32, strike2: i32, expiry: &str) -> AccountPath {
        AccountPath::from_string(&format!(
            "Assets:IBKR:BoxSpread:{}:{}:{}:{}",
            symbol, strike1, strike2, expiry
        ))
        .unwrap()
    }

    pub fn equity_capital() -> AccountPath {
        AccountPath::from_string("Equity:Capital").unwrap()
    }

    pub fn equity_realized_pnl() -> AccountPath {
        AccountPath::from_string("Equity:RealizedPnL").unwrap()
    }

    pub fn equity_unrealized_pnl() -> AccountPath {
        AccountPath::from_string("Equity:UnrealizedPnL").unwrap()
    }

    pub fn expenses_commissions() -> AccountPath {
        AccountPath::from_string("Expenses:Commissions").unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_path_from_string() {
        let path = AccountPath::from_string("Assets:IBKR:SPY").unwrap();
        assert_eq!(path.segments(), &["Assets", "IBKR", "SPY"]);
    }

    #[test]
    fn test_account_path_to_string() {
        let path = AccountPath::from_string("Assets:IBKR:SPY").unwrap();
        assert_eq!(path.to_string(), "Assets:IBKR:SPY");
    }

    #[test]
    fn test_account_path_parent() {
        let path = AccountPath::from_string("Assets:IBKR:SPY").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "Assets:IBKR");
    }

    #[test]
    fn test_account_path_name() {
        let path = AccountPath::from_string("Assets:IBKR:SPY").unwrap();
        assert_eq!(path.name(), "SPY");
    }

    #[test]
    fn test_account_path_invalid() {
        assert!(AccountPath::from_string("").is_err());
        assert!(AccountPath::from_string(":").is_err());
    }

    #[test]
    fn test_account_helpers() {
        assert_eq!(accounts::ibkr_cash().to_string(), "Assets:IBKR:Cash");
        assert_eq!(
            accounts::ibkr_box_spread("SPY", 450, 460, "20251219").to_string(),
            "Assets:IBKR:BoxSpread:SPY:450:460:20251219"
        );
    }
}
