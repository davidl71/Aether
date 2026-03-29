//! Alpaca health monitoring for read-only exploration (quotes/account visibility).
//!
//! Connectivity checks and status for the system health stream — not order flow.

use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use tokio::time;
use tracing::{debug, info, warn};

use api::credentials::{credential_source, CredentialKey};

/// Alpaca health for paper vs live API credentials (which endpoint keys target; exploration mode stays read-only).
#[derive(Debug, Clone)]
pub struct AlpacaHealth {
    pub is_paper: bool,
    pub connected: bool,
    pub status: String,
    pub account_id: Option<String>,
    pub equity: Option<f64>,
    pub buying_power: Option<f64>,
    pub last_error: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

impl AlpacaHealth {
    pub fn new(is_paper: bool) -> Self {
        Self {
            is_paper,
            connected: false,
            status: "not_configured".to_string(),
            account_id: None,
            equity: None,
            buying_power: None,
            last_error: None,
            last_check: Utc::now(),
        }
    }

    pub fn source_name(&self) -> &'static str {
        if self.is_paper {
            "alpaca_paper"
        } else {
            "alpaca_live"
        }
    }

    pub fn with_connected(mut self, account_id: String, equity: f64, buying_power: f64) -> Self {
        self.connected = true;
        self.status = "ok".to_string();
        self.account_id = Some(account_id);
        self.equity = Some(equity);
        self.buying_power = Some(buying_power);
        self.last_error = None;
        self.last_check = Utc::now();
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.connected = false;
        self.status = "error".to_string();
        self.last_error = Some(error);
        self.last_check = Utc::now();
        self
    }

    pub fn with_not_configured(mut self) -> Self {
        self.connected = false;
        self.status = "not_configured".to_string();
        self.last_check = Utc::now();
        self
    }
}

/// Alpaca health monitor that periodically checks API connectivity.
pub struct AlpacaHealthMonitor {
    paper_health: AlpacaHealth,
    live_health: AlpacaHealth,
}

impl AlpacaHealthMonitor {
    pub fn new() -> Self {
        Self {
            paper_health: AlpacaHealth::new(true),
            live_health: AlpacaHealth::new(false),
        }
    }

    pub fn paper_health(&self) -> &AlpacaHealth {
        &self.paper_health
    }

    pub fn live_health(&self) -> &AlpacaHealth {
        &self.live_health
    }

    /// Check if credentials are configured for the given environment.
    fn has_credentials(is_paper: bool) -> bool {
        let key = if is_paper {
            CredentialKey::AlpacaPaperApiKeyId
        } else {
            CredentialKey::AlpacaLiveApiKeyId
        };

        match credential_source(key) {
            Some(source) => {
                debug!(?source, is_paper, "Found Alpaca credential");
                true
            }
            None => {
                debug!(is_paper, "No Alpaca credentials configured");
                false
            }
        }
    }

    /// Perform health check for a single environment.
    async fn check_health(is_paper: bool) -> AlpacaHealth {
        if !Self::has_credentials(is_paper) {
            return AlpacaHealth::new(is_paper).with_not_configured();
        }

        // Try to fetch account info
        match fetch_account_info(is_paper).await {
            Ok((account_id, equity, buying_power)) => {
                info!(
                    is_paper,
                    account_id = %account_id,
                    equity = %equity,
                    "Alpaca health check passed"
                );
                AlpacaHealth::new(is_paper).with_connected(account_id, equity, buying_power)
            }
            Err(e) => {
                warn!(
                    is_paper,
                    error = %e,
                    "Alpaca health check failed"
                );
                AlpacaHealth::new(is_paper).with_error(e.to_string())
            }
        }
    }

    /// Run health checks for both environments.
    pub async fn check_all(&mut self) {
        self.paper_health = Self::check_health(true).await;
        self.live_health = Self::check_health(false).await;
    }

    /// Spawn a background task that periodically checks health.
    pub fn spawn_health_checks(
        self,
        event_tx: tokio::sync::mpsc::UnboundedSender<crate::events::AppEvent>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(30));
            let mut monitor = self;

            loop {
                interval.tick().await;
                monitor.check_all().await;

                let _ = event_tx.send(crate::events::AppEvent::AlpacaHealthUpdate {
                    is_paper: monitor.paper_health.is_paper,
                    connected: monitor.paper_health.connected,
                    account_id: monitor.paper_health.account_id.clone(),
                    equity: monitor.paper_health.equity,
                    buying_power: monitor.paper_health.buying_power,
                    status: monitor.paper_health.status.clone(),
                    last_error: monitor.paper_health.last_error.clone(),
                });

                let _ = event_tx.send(crate::events::AppEvent::AlpacaHealthUpdate {
                    is_paper: monitor.live_health.is_paper,
                    connected: monitor.live_health.connected,
                    account_id: monitor.live_health.account_id.clone(),
                    equity: monitor.live_health.equity,
                    buying_power: monitor.live_health.buying_power,
                    status: monitor.live_health.status.clone(),
                    last_error: monitor.live_health.last_error.clone(),
                });
            }
        })
    }
}

impl Default for AlpacaHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Fetch account info from Alpaca API.
async fn fetch_account_info(is_paper: bool) -> Result<(String, f64, f64), anyhow::Error> {
    use api::AlpacaPositionSource;

    let source = AlpacaPositionSource::from_paper(is_paper)
        .ok_or_else(|| anyhow::anyhow!("Alpaca credentials not configured"))?;

    let account = source.fetch_account().await?;

    Ok((account.account_id, account.equity, account.buying_power))
}

/// Convert AlpacaHealth to NatsTransportHealthState for publishing.
pub fn to_transport_health(health: &AlpacaHealth) -> nats_adapter::NatsTransportHealthState {
    use nats_adapter::NatsTransportHealthState;

    let mut extra = HashMap::new();

    if let Some(ref account_id) = health.account_id {
        extra.insert("account_id".to_string(), account_id.clone());
    }

    if let Some(equity) = health.equity {
        extra.insert("equity".to_string(), format!("{:.2}", equity));
    }

    if let Some(buying_power) = health.buying_power {
        extra.insert("buying_power".to_string(), format!("{:.2}", buying_power));
    }

    let status = if health.connected {
        NatsTransportHealthState::connected(None, health.last_check)
            .with_extra("source", health.source_name())
    } else {
        NatsTransportHealthState::disconnected(
            None,
            health.last_check,
            health.last_error.clone(),
            Some(format!("Status: {}", health.status)),
        )
        .with_extra("source", health.source_name())
    };

    NatsTransportHealthState { extra, ..status }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpaca_health_new() {
        let health = AlpacaHealth::new(true);
        assert!(!health.connected);
        assert_eq!(health.status, "not_configured");
        assert!(health.is_paper);
    }

    #[test]
    fn test_alpaca_health_with_connected() {
        let health =
            AlpacaHealth::new(true).with_connected("PA-123".to_string(), 100000.0, 200000.0);

        assert!(health.connected);
        assert_eq!(health.status, "ok");
        assert_eq!(health.account_id, Some("PA-123".to_string()));
        assert_eq!(health.equity, Some(100000.0));
    }

    #[test]
    fn test_alpaca_health_with_error() {
        let health = AlpacaHealth::new(false).with_error("API timeout".to_string());

        assert!(!health.connected);
        assert_eq!(health.status, "error");
        assert_eq!(health.last_error, Some("API timeout".to_string()));
    }
}
