//! Alpaca account/position read API (paper or live endpoints) for exploration snapshots — not order flow.

use std::sync::Mutex;

/// Serialize Alpaca HTTP calls: `alpaca_api_client` reads `APCA_API_KEY_*` from the environment
/// (`expect`), so we temporarily inject credential-store values and restore afterward.
static ALPACA_API_ENV_LOCK: Mutex<()> = Mutex::new(());

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
    /// When true, inject keyring/file credentials into `APCA_*` env for each sync call (see module static lock).
    use_credential_store: bool,
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

        Some(Self {
            is_paper,
            use_credential_store: false,
        })
    }

    /// Create position source for Alpaca **paper vs live REST** credentials (read-only account data).
    /// Returns None if credentials are not configured for the requested environment.
    pub fn from_paper(is_paper: bool) -> Option<Self> {
        use crate::credentials::{alpaca_credentials, AlpacaEnvironment};

        let env = if is_paper {
            AlpacaEnvironment::Paper
        } else {
            AlpacaEnvironment::Live
        };

        alpaca_credentials(env).map(|_| Self {
            is_paper,
            use_credential_store: true,
        })
    }

    /// True when using Alpaca paper API credentials (vs live account API).
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

        let account_type = self.account_type();

        let run = || {
            let query = PositionsQuery::new(account_type);
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
        };

        if self.use_credential_store {
            with_alpaca_env_for_credential_store(self.is_paper, run)
        } else {
            run()
        }
    }

    /// Fetch account information.
    pub fn fetch_account_sync(&self) -> Result<AlpacaAccountInfo, anyhow::Error> {
        use alpaca_api_client::trading::account::get_account;

        let is_paper = self.is_paper;
        let account_type = self.account_type();

        let run = || {
            let account = get_account(account_type)
                .map_err(|e| anyhow::anyhow!("Failed to fetch account: {}", e))?;

            Ok(AlpacaAccountInfo {
                account_id: account.id,
                cash: account.cash.parse().unwrap_or(0.0),
                buying_power: account.buying_power.parse().unwrap_or(0.0),
                equity: account.equity.parse().unwrap_or(0.0),
                portfolio_value: account.portfolio_value.parse().unwrap_or(0.0),
                is_paper,
            })
        };

        if self.use_credential_store {
            with_alpaca_env_for_credential_store(self.is_paper, run)
        } else {
            run()
        }
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

fn with_alpaca_env_for_credential_store<R>(
    is_paper: bool,
    f: impl FnOnce() -> Result<R, anyhow::Error>,
) -> Result<R, anyhow::Error> {
    use crate::credentials::{alpaca_credentials, AlpacaEnvironment};

    let env = if is_paper {
        AlpacaEnvironment::Paper
    } else {
        AlpacaEnvironment::Live
    };
    let creds = alpaca_credentials(env)
        .ok_or_else(|| anyhow::anyhow!("Alpaca credentials not configured"))?;

    let _lock = ALPACA_API_ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("Alpaca API env lock poisoned"))?;

    struct AlpacaEnvRestore {
        prev_id: Option<String>,
        prev_secret: Option<String>,
    }

    impl Drop for AlpacaEnvRestore {
        fn drop(&mut self) {
            match self.prev_id.as_deref() {
                Some(v) => std::env::set_var("APCA_API_KEY_ID", v),
                None => {
                    let _ = std::env::remove_var("APCA_API_KEY_ID");
                }
            }
            match self.prev_secret.as_deref() {
                Some(v) => std::env::set_var("APCA_API_SECRET_KEY", v),
                None => {
                    let _ = std::env::remove_var("APCA_API_SECRET_KEY");
                }
            }
        }
    }

    let prev_id = std::env::var("APCA_API_KEY_ID").ok();
    let prev_secret = std::env::var("APCA_API_SECRET_KEY").ok();
    let _restore = AlpacaEnvRestore {
        prev_id,
        prev_secret,
    };

    std::env::set_var("APCA_API_KEY_ID", creds.api_key_id.trim());
    std::env::set_var("APCA_API_SECRET_KEY", creds.api_secret_key.trim());

    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpaca_position_source() {
        let _ = AlpacaPositionSource {
            is_paper: true,
            use_credential_store: false,
        };
    }
}
