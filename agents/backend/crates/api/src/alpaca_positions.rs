//! Alpaca position source for fetching account positions and balances.

/// Position information from Alpaca.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlpacaPosition {
    pub symbol: String,
    pub quantity: i32,
    pub cost_basis: f64,
    pub market_value: f64,
    pub unrealized_pl: f64,
}

/// Account information from Alpaca.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlpacaAccountInfo {
    pub account_id: String,
    pub cash: f64,
    pub buying_power: f64,
    pub equity: f64,
    pub portfolio_value: f64,
    pub is_paper: bool,
}

/// Alpaca position source.
#[derive(Clone)]
pub struct AlpacaPositionSource {
    is_paper: bool,
}

impl AlpacaPositionSource {
    /// Create new position source from environment.
    pub fn from_env() -> Option<Self> {
        let has_key = std::env::var("APCA_API_KEY_ID").is_ok();
        let has_secret = std::env::var("APCA_API_SECRET_KEY").is_ok();

        if !has_key || !has_secret {
            return None;
        }

        let is_paper = std::env::var("APCA_API_BASE_URL")
            .map(|url| url.contains("paper"))
            .unwrap_or(true);

        Some(Self { is_paper })
    }

    /// Create new position source for paper or live trading.
    /// Returns None if credentials are not configured for the requested environment.
    pub fn from_paper(is_paper: bool) -> Option<Self> {
        use crate::credentials::{credential_source, CredentialKey};

        let key = if is_paper {
            CredentialKey::AlpacaPaperApiKeyId
        } else {
            CredentialKey::AlpacaLiveApiKeyId
        };

        if credential_source(key).is_some() {
            Some(Self { is_paper })
        } else {
            None
        }
    }

    /// Check if this is paper trading environment.
    pub fn is_paper(&self) -> bool {
        self.is_paper
    }

    /// Get the account type for API calls.
    fn account_type(&self) -> alpaca_api_client::trading::AccountType {
        if self.is_paper {
            alpaca_api_client::trading::AccountType::Paper
        } else {
            alpaca_api_client::trading::AccountType::Live
        }
    }

    /// Fetch all open positions.
    pub fn fetch_positions_sync(&self) -> Result<Vec<AlpacaPosition>, anyhow::Error> {
        use alpaca_api_client::trading::positions::PositionsQuery;

        let query = PositionsQuery::new(self.account_type());
        let positions = query
            .get_all_open_positions()
            .map_err(|e| anyhow::anyhow!("Failed to fetch positions: {}", e))?;

        Ok(positions
            .into_iter()
            .map(|p| AlpacaPosition {
                symbol: p.symbol,
                quantity: p.qty.parse().unwrap_or(0.0) as i32,
                cost_basis: p.cost_basis.parse().unwrap_or(0.0),
                market_value: p.market_value.parse().unwrap_or(0.0),
                unrealized_pl: p.unrealized_pl.parse().unwrap_or(0.0),
            })
            .collect())
    }

    /// Fetch account information.
    pub fn fetch_account_sync(&self) -> Result<AlpacaAccountInfo, anyhow::Error> {
        use alpaca_api_client::trading::account::get_account;

        let account = get_account(self.account_type())
            .map_err(|e| anyhow::anyhow!("Failed to fetch account: {}", e))?;

        Ok(AlpacaAccountInfo {
            account_id: account.id,
            cash: account.cash.parse().unwrap_or(0.0),
            buying_power: account.buying_power.parse().unwrap_or(0.0),
            equity: account.equity.parse().unwrap_or(0.0),
            portfolio_value: account.portfolio_value.parse().unwrap_or(0.0),
            is_paper: self.is_paper,
        })
    }

    /// Fetch all open positions (async wrapper).
    pub async fn fetch_positions(&self) -> Result<Vec<AlpacaPosition>, anyhow::Error> {
        let source = self.clone();
        tokio::task::spawn_blocking(move || source.fetch_positions_sync()).await?
    }

    /// Fetch account information (async wrapper).
    pub async fn fetch_account(&self) -> Result<AlpacaAccountInfo, anyhow::Error> {
        let source = self.clone();
        tokio::task::spawn_blocking(move || source.fetch_account_sync()).await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpaca_position_source() {
        let _ = AlpacaPositionSource { is_paper: true };
    }
}
