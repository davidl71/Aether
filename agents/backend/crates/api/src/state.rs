use chrono::Utc;
use serde::{Deserialize, Serialize};

use common::snapshot as cmn;

use crate::NatsTransportHealthState;

// ---------------------------------------------------------------------------
// Api-only types (not shared)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl Alert {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Info,
            message: message.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Warning,
            message: message.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Error,
            message: message.into(),
            timestamp: Utc::now(),
        }
    }
}

impl From<cmn::Alert> for Alert {
    fn from(a: cmn::Alert) -> Self {
        let level = match a.level.to_uppercase().as_str() {
            "WARNING" => AlertLevel::Warning,
            "ERROR" => AlertLevel::Error,
            _ => AlertLevel::Info,
        };
        Alert {
            level,
            message: a.message,
            timestamp: a.timestamp,
        }
    }
}

impl From<Alert> for cmn::Alert {
    fn from(a: Alert) -> Self {
        let level = match a.level {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARNING",
            AlertLevel::Error => "ERROR",
        };
        cmn::Alert {
            level: level.to_string(),
            message: a.message,
            timestamp: a.timestamp,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolSnapshot {
    pub symbol: String,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub spread: f64,
    pub roi: f64,
    pub maker_count: u32,
    pub taker_count: u32,
    pub volume: u64,
    pub candle: cmn::CandleSnapshot,
}

// ---------------------------------------------------------------------------
// System snapshot (api-only aggregate)
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub generated_at: chrono::DateTime<Utc>,
    pub started_at: chrono::DateTime<Utc>,
    pub mode: String,
    pub strategy: String,
    pub account_id: String,
    pub metrics: cmn::Metrics,
    pub symbols: Vec<SymbolSnapshot>,
    pub positions: Vec<cmn::PositionSnapshot>,
    pub historic: Vec<cmn::HistoricPosition>,
    pub orders: Vec<cmn::OrderSnapshot>,
    pub decisions: Vec<cmn::StrategyDecisionSnapshot>,
    pub alerts: Vec<Alert>,
    pub risk: cmn::RiskStatus,
    #[serde(default)]
    pub market_data_source: Option<String>,
    /// Backend NATS transport telemetry (snapshot publisher / primary client).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nats_transport: Option<NatsTransportHealthState>,
}

impl std::fmt::Debug for SystemSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemSnapshot")
            .field("generated_at", &self.generated_at)
            .field("started_at", &self.started_at)
            .field("mode", &self.mode)
            .field("strategy", &self.strategy)
            .field("account_id", &self.account_id)
            .field("metrics", &self.metrics)
            .field("symbols", &self.symbols)
            .field("positions", &self.positions)
            .field("historic", &self.historic)
            .field("orders", &self.orders)
            .field("decisions", &self.decisions)
            .field("alerts", &self.alerts)
            .field("risk", &self.risk)
            .field("nats_transport", &self.nats_transport)
            .finish()
    }
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            generated_at: Utc::now(),
            started_at: Utc::now(),
            mode: "DRY-RUN".into(),
            strategy: "IDLE".into(),
            account_id: "DU123456".into(),
            metrics: cmn::Metrics::default(),
            symbols: Vec::new(),
            positions: Vec::new(),
            historic: Vec::new(),
            orders: Vec::new(),
            decisions: Vec::new(),
            alerts: vec![Alert::info("Backend initialising")],
            risk: cmn::RiskStatus::default(),
            market_data_source: None,
            nats_transport: None,
        }
    }
}

impl SystemSnapshot {
    pub fn touch(&mut self) {
        self.generated_at = Utc::now();
    }

    pub fn set_strategy_status(&mut self, status: impl Into<String>) {
        self.strategy = status.into();
    }
}

// ---------------------------------------------------------------------------
// Re-exports from common
// ---------------------------------------------------------------------------

pub use common::snapshot::Alert as CommonAlert;
pub use common::snapshot::CandleSnapshot;
pub use common::snapshot::HistoricPosition;
pub use common::snapshot::Metrics;
pub use common::snapshot::OrderSnapshot;
pub use common::snapshot::PositionSnapshot;
pub use common::snapshot::RiskStatus;
pub use common::snapshot::StrategyDecisionSnapshot;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_from_common_round_trip() {
        let alert = Alert::info("test");
        let common: cmn::Alert = alert.clone().into();
        let back: Alert = common.into();
        assert!(matches!(back.level, AlertLevel::Info));
        assert_eq!(back.message, "test");
    }
}
