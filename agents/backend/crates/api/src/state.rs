use std::sync::Arc;

use chrono::{DateTime, Utc};
use market_data::MarketDataEvent;
use risk::RiskDecision;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub type SharedSnapshot = Arc<RwLock<SystemSnapshot>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSnapshot {
  pub generated_at: DateTime<Utc>,
  pub mode: String,
  pub strategy: String,
  pub account_id: String,
  pub metrics: Metrics,
  pub symbols: Vec<SymbolSnapshot>,
  pub positions: Vec<PositionSnapshot>,
  pub historic: Vec<HistoricPosition>,
  pub orders: Vec<OrderSnapshot>,
  pub decisions: Vec<StrategyDecisionSnapshot>,
  pub alerts: Vec<Alert>,
  pub risk: RiskStatus,
}

impl Default for SystemSnapshot {
  fn default() -> Self {
    Self {
      generated_at: Utc::now(),
      mode: "DRY-RUN".into(),
      strategy: "IDLE".into(),
      account_id: "DU123456".into(),
      metrics: Metrics::default(),
      symbols: Vec::new(),
      positions: Vec::new(),
      historic: Vec::new(),
      orders: Vec::new(),
      decisions: Vec::new(),
      alerts: vec![Alert::info("Backend initialising")],
      risk: RiskStatus::default(),
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

  pub fn apply_market_event(&mut self, event: &MarketDataEvent) {
    self.touch();

    if let Some(entry) = self.symbols.iter_mut().find(|sym| sym.symbol == event.symbol) {
      entry.update_from_event(event);
    } else {
      self.symbols.push(SymbolSnapshot::from_event(event));
    }

    let mark_sum = self.symbols.iter().map(|symbol| symbol.last).sum::<f64>();
    self.metrics.net_liq = 100_000.0 + mark_sum;
    self.metrics.buying_power = self.metrics.net_liq * 0.8;
    self.metrics.excess_liquidity = self.metrics.net_liq * 0.25;
    self.metrics.margin_requirement = self.metrics.net_liq * 0.15;
    self.metrics.commissions = (self.metrics.commissions * 0.98) + 0.02;

    self.metrics.portal_ok = true;
    self.metrics.tws_ok = true;
    self.metrics.orats_ok = true;
    self.metrics.questdb_ok = true;
  }

  pub fn apply_strategy_execution(&mut self, decision: StrategyDecisionSnapshot) {
    self.touch();
    self.strategy = "RUNNING".into();
    self.decisions.push(decision.clone());
    if self.decisions.len() > 50 {
      self.decisions.remove(0);
    }

    let order_id = format!("ORD-{}", Utc::now().timestamp_millis());
    self.orders.push(OrderSnapshot {
      id: order_id,
      symbol: decision.symbol.clone(),
      side: decision.side.clone(),
      quantity: decision.quantity,
      status: "FILLED".into(),
      submitted_at: decision.created_at,
    });
    if self.orders.len() > 32 {
      self.orders.remove(0);
    }

    if let Some(idx) = self.positions.iter().position(|p| p.symbol == decision.symbol) {
      let prev_qty = self.positions[idx].quantity;
      let cost_basis = self.positions[idx].cost_basis;
      let new_qty = prev_qty + decision.quantity;

      if new_qty == 0 {
        let position = self.positions.remove(idx);
        let realized = (decision.mark - cost_basis) * prev_qty as f64;
        self.historic.push(HistoricPosition {
          id: format!("HIST-{}", self.historic.len() + 1),
          symbol: position.symbol,
          quantity: prev_qty,
          realized_pnl: realized,
          closed_at: decision.created_at,
        });
      } else {
        let position = &mut self.positions[idx];
        position.quantity = new_qty;
        position.mark = decision.mark;
        position.unrealized_pnl = (position.mark - position.cost_basis) * position.quantity as f64;
      }
    } else {
      self.positions.push(PositionSnapshot {
        id: format!("POS-{}", self.positions.len() + self.historic.len() + 1),
        symbol: decision.symbol.clone(),
        quantity: decision.quantity,
        cost_basis: decision.mark,
        mark: decision.mark,
        unrealized_pnl: 0.0,
      });
    }
  }

  pub fn update_risk_status(&mut self, outcome: &RiskDecision) {
    self.touch();
    self.risk.allowed = outcome.allowed;
    self.risk.reason = outcome.reason.clone();
    self.risk.updated_at = Utc::now();
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metrics {
  pub net_liq: f64,
  pub buying_power: f64,
  pub excess_liquidity: f64,
  pub margin_requirement: f64,
  pub commissions: f64,
  pub portal_ok: bool,
  pub tws_ok: bool,
  pub orats_ok: bool,
  pub questdb_ok: bool,
}

impl Default for Metrics {
  fn default() -> Self {
    Self {
      net_liq: 100_000.0,
      buying_power: 80_000.0,
      excess_liquidity: 25_000.0,
      margin_requirement: 15_000.0,
      commissions: 0.0,
      portal_ok: false,
      tws_ok: false,
      orats_ok: false,
      questdb_ok: false,
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
  pub candle: CandleSnapshot,
}

impl SymbolSnapshot {
  fn from_event(event: &MarketDataEvent) -> Self {
    let mid = (event.bid + event.ask) * 0.5;
    Self {
      symbol: event.symbol.clone(),
      last: mid,
      bid: event.bid,
      ask: event.ask,
      spread: (event.ask - event.bid).max(0.0),
      roi: 0.0,
      maker_count: 1,
      taker_count: 0,
      volume: 1,
      candle: CandleSnapshot::new(mid, event.timestamp),
    }
  }

  fn update_from_event(&mut self, event: &MarketDataEvent) {
    let mid = (event.bid + event.ask) * 0.5;
    self.last = mid;
    self.bid = event.bid;
    self.ask = event.ask;
    self.spread = (event.ask - event.bid).max(0.0);
    self.volume = self.volume.saturating_add(1);
    self.roi = (self.roi * 0.9) + 0.1 * ((mid / self.candle.entry) - 1.0) * 100.0;
    self.candle.update(mid, event.timestamp);
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandleSnapshot {
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: u64,
  pub entry: f64,
  pub updated: DateTime<Utc>,
}

impl CandleSnapshot {
  fn new(price: f64, timestamp: DateTime<Utc>) -> Self {
    Self {
      open: price,
      high: price,
      low: price,
      close: price,
      volume: 1,
      entry: price,
      updated: timestamp,
    }
  }

  fn update(&mut self, price: f64, timestamp: DateTime<Utc>) {
    self.high = self.high.max(price);
    self.low = self.low.min(price);
    self.close = price;
    self.volume = self.volume.saturating_add(1);
    self.updated = timestamp;
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionSnapshot {
  pub id: String,
  pub symbol: String,
  pub quantity: i32,
  pub cost_basis: f64,
  pub mark: f64,
  pub unrealized_pnl: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoricPosition {
  pub id: String,
  pub symbol: String,
  pub quantity: i32,
  pub realized_pnl: f64,
  pub closed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderSnapshot {
  pub id: String,
  pub symbol: String,
  pub side: String,
  pub quantity: i32,
  pub status: String,
  pub submitted_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyDecisionSnapshot {
  pub symbol: String,
  pub quantity: i32,
  pub side: String,
  pub mark: f64,
  pub created_at: DateTime<Utc>,
}

impl StrategyDecisionSnapshot {
  pub fn new(symbol: String, quantity: i32, side: impl Into<String>, mark: f64, created_at: DateTime<Utc>) -> Self {
    Self {
      symbol,
      quantity,
      side: side.into(),
      mark,
      created_at,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskStatus {
  pub allowed: bool,
  pub reason: Option<String>,
  pub updated_at: DateTime<Utc>,
}

impl Default for RiskStatus {
  fn default() -> Self {
    Self {
      allowed: true,
      reason: None,
      updated_at: Utc::now(),
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
  pub level: AlertLevel,
  pub message: String,
  pub timestamp: DateTime<Utc>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
  Info,
  Warning,
  Error,
}
