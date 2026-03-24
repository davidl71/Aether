//! Application state and event dispatch.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

use api::finance_rates::{BenchmarksResponse, CurveResponse};
use api::loans::LoanRecord;
use api::{
    Alert, BackendHealthState, CommandReply, CommandStatus, RuntimeOrderDto, RuntimePositionDto,
    ScenarioDto,
};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::{mpsc, watch};
use tui_logger::TuiWidgetState;

use crate::config::TuiConfig;
use crate::events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget};
use crate::models::TuiSnapshot;
use crate::ui::Candle;

const SPARKLINE_HISTORY_SIZE: usize = 20;
const CHART_HISTORY_SIZE: usize = 120;
const TOAST_TTL_SECS: u64 = 3;

/// Severity level for transient toast notifications.
#[derive(Debug, Clone, PartialEq)]
pub enum ToastLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Positions,
    Charts,
    Orders,
    Alerts,
    Yield,
    Loans,
    Scenarios,
    Logs,
    Settings,
}

/// Content for the row-detail overlay (Orders/Positions/Scenarios). Same overlay pattern as help (?); Esc to close.
#[derive(Debug, Clone)]
pub enum DetailPopupContent {
    Order(RuntimeOrderDto),
    Position(RuntimePositionDto),
    Scenario(ScenarioDto),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Help,
    DetailPopup,
    SettingsEditConfig,
    SettingsAddSymbol,
    LoanForm,
    ChartSearch,
    OrdersFilter,
    LogPanel,
}

/// Latest command state shown in the TUI status area.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandStatusView {
    pub command_id: Option<String>,
    pub issued_at: Option<String>,
    pub action: String,
    pub status: CommandStatus,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl CommandStatusView {
    pub fn from_reply(reply: &CommandReply) -> Self {
        Self {
            command_id: Some(reply.command_id.clone()),
            issued_at: Some(reply.issued_at.clone()),
            action: reply.action.clone(),
            status: reply.status.clone(),
            message: reply.message.clone(),
            error: reply.error.clone(),
        }
    }

    pub fn failure(action: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            command_id: None,
            issued_at: None,
            action: action.into(),
            status: CommandStatus::Failed,
            message: None,
            error: Some(error.into()),
        }
    }

    pub fn disabled(action: impl Into<String>) -> Self {
        Self::failure(
            action,
            "Execution is deprecated in exploration mode; data views remain available.",
        )
    }

    pub fn success(action: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            command_id: None,
            issued_at: None,
            action: action.into(),
            status: CommandStatus::Completed,
            message: Some(message.into()),
            error: None,
        }
    }
}

#[derive(Clone)]
pub struct MarketDataSourceMeta {
    pub source: String,
    pub priority: u32,
    last_tick_at: Instant,
}

impl MarketDataSourceMeta {
    pub fn new(source: impl Into<String>, priority: u32) -> Self {
        Self {
            source: source.into(),
            priority,
            last_tick_at: Instant::now(),
        }
    }

    pub fn age_secs(&self) -> u64 {
        self.last_tick_at.elapsed().as_secs()
    }
}

/// State for the loan entry form overlay in Loans tab.
#[derive(Debug, Clone)]
pub struct LoanEntryState {
    pub bank_name: String,
    pub account_number: String,
    pub loan_type: LoanType,
    pub principal: String,
    pub interest_rate: String,
    pub spread: String,
    pub origination_date: String,
    pub first_payment_date: String,
    pub num_payments: String,
    pub monthly_payment: String,
    pub maturity_date: String,
    pub current_field: usize,
    pub validation_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoanType {
    ShirBased,
    CpiLinked,
}

impl LoanEntryState {
    pub fn new() -> Self {
        Self {
            bank_name: String::new(),
            account_number: String::new(),
            loan_type: LoanType::ShirBased,
            principal: String::new(),
            interest_rate: String::new(),
            spread: String::new(),
            origination_date: String::new(),
            first_payment_date: String::new(),
            num_payments: String::new(),
            monthly_payment: String::new(),
            maturity_date: String::new(),
            current_field: 0,
            validation_error: None,
        }
    }

    pub fn calculate_maturity(&mut self) {
        if !self.first_payment_date.is_empty() && !self.num_payments.is_empty() {
            if let (Ok(payments), Ok(base)) = (
                self.num_payments.parse::<i32>(),
                parse_date(&self.first_payment_date),
            ) {
                let days_to_add = (payments as i64 * 30) - 1;
                let maturity = base + chrono::Duration::days(days_to_add);
                self.maturity_date = maturity.format("%Y-%m-%d").to_string();
            }
        }
    }

    pub fn calculate_monthly_payment(&mut self) {
        if !self.principal.is_empty()
            && !self.interest_rate.is_empty()
            && !self.num_payments.is_empty()
        {
            if let (Ok(principal), Ok(rate), Ok(payments)) = (
                self.principal.parse::<f64>(),
                self.interest_rate.parse::<f64>(),
                self.num_payments.parse::<i32>(),
            ) {
                if rate > 0.0 && payments > 0 {
                    let monthly_rate = rate / 100.0 / 12.0;
                    let n = payments as f64;
                    // Standard annuity formula: M = P * r * (1+r)^n / ((1+r)^n - 1)
                    let factor = (1.0 + monthly_rate).powf(n);
                    let payment = principal * monthly_rate * factor / (factor - 1.0);
                    self.monthly_payment = format!("{:.2}", payment);
                }
            }
        }
    }

    pub fn toggle_loan_type(&mut self) {
        self.loan_type = match self.loan_type {
            LoanType::ShirBased => LoanType::CpiLinked,
            LoanType::CpiLinked => LoanType::ShirBased,
        };
    }

    pub fn is_complete(&self) -> bool {
        !self.bank_name.is_empty()
            && !self.account_number.is_empty()
            && !self.principal.is_empty()
            && self.principal.parse::<f64>().is_ok()
            && !self.interest_rate.is_empty()
            && self.interest_rate.parse::<f64>().is_ok()
            && !self.origination_date.is_empty()
            && !self.first_payment_date.is_empty()
            && !self.num_payments.is_empty()
            && self.num_payments.parse::<i32>().is_ok()
    }

    pub fn to_loan_record(&self) -> Option<LoanRecord> {
        if !self.is_complete() {
            return None;
        }

        let principal = self.principal.parse::<f64>().ok()?;
        let interest_rate = self.interest_rate.parse::<f64>().ok()?;
        let spread = if self.spread.is_empty() {
            0.0
        } else {
            self.spread.parse::<f64>().unwrap_or(0.0)
        };
        self.num_payments.parse::<i32>().ok()?;
        let monthly_payment = if self.monthly_payment.is_empty() {
            0.0
        } else {
            self.monthly_payment.parse::<f64>().unwrap_or(0.0)
        };

        let loan_type = match self.loan_type {
            LoanType::ShirBased => api::loans::LoanType::ShirBased,
            LoanType::CpiLinked => api::loans::LoanType::CpiLinked,
        };

        let now = chrono::Utc::now().to_rfc3339();

        Some(LoanRecord {
            loan_id: format!("loan-{}-{}", self.bank_name, chrono::Utc::now().timestamp()),
            bank_name: self.bank_name.clone(),
            account_number: self.account_number.clone(),
            loan_type,
            principal,
            original_principal: principal,
            interest_rate,
            spread,
            base_cpi: 0.0,
            current_cpi: 0.0,
            origination_date: self.origination_date.clone(),
            maturity_date: self.maturity_date.clone(),
            next_payment_date: self.first_payment_date.clone(),
            monthly_payment,
            payment_frequency_months: 1,
            status: api::loans::LoanStatus::Active,
            last_update: now,
        })
    }
}

fn parse_date(s: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Dashboard,
        Tab::Positions,
        Tab::Charts,
        Tab::Orders,
        Tab::Alerts,
        Tab::Yield,
        Tab::Loans,
        Tab::Scenarios,
        Tab::Logs,
        Tab::Settings,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dash",
            Tab::Positions => "Pos",
            Tab::Charts => "Charts",
            Tab::Orders => "Orders",
            Tab::Alerts => "Alerts",
            Tab::Yield => "Yield",
            Tab::Loans => "Loans",
            Tab::Scenarios => "Scen",
            Tab::Logs => "Logs",
            Tab::Settings => "Set",
        }
    }

    fn index(&self) -> usize {
        Tab::ALL.iter().position(|t| t == self).unwrap_or(0)
    }

    pub fn next(&self) -> Tab {
        let i = (self.index() + 1) % Tab::ALL.len();
        Tab::ALL[i].clone()
    }

    pub fn prev(&self) -> Tab {
        let i = (self.index() + Tab::ALL.len() - 1) % Tab::ALL.len();
        Tab::ALL[i].clone()
    }
}

pub struct App {
    pub config: TuiConfig,
    /// Set true when state changes require redraw. Reset after each render.
    pub needs_redraw: bool,
    pub active_tab: Tab,
    /// Latest snapshot from periodic NATS publication. Updated via `snapshot_rx`.
    /// Access via `get_snapshot()` / `set_snapshot()` using UnsafeCell interior mutability.
    snapshot: std::cell::UnsafeCell<Option<TuiSnapshot>>,
    /// Per-symbol ROI history for sparkline visualization (symbol -> deque of ROI values)
    pub roi_history: HashMap<String, VecDeque<f64>>,
    /// Order filter text (filters orders by symbol or status)
    pub order_filter: String,
    /// True while the Orders filter input mode is active.
    pub order_filter_active: bool,
    /// State for the tui-logger widget (scroll position, level filter).
    pub log_state: TuiWidgetState,
    /// Current display level shown in the Logs tab title.
    pub log_display_level: log::LevelFilter,
    pub nats_status: ConnectionStatus,
    pub should_quit: bool,
    /// Last command result from strategy/control actions (shown in status/hint bar).
    pub last_command_status: Option<CommandStatusView>,
    /// Transient toast notifications (auto-expire after TOAST_TTL_SECS).
    pub toast_queue: VecDeque<(String, ToastLevel, Instant)>,
    /// When true, show the help overlay (key bindings).
    pub show_help: bool,
    /// When true, show the debug log panel overlay (toggled with backtick).
    pub show_log_panel: bool,
    /// When Some, show detail overlay for selected Order or Position (Enter to open, Esc to close).
    pub detail_popup: Option<DetailPopupContent>,
    /// Config validation warning (e.g. missing NATS_URL); shown in status bar when set.
    pub config_warning: Option<String>,
    /// Backend health from system.health (backend id → state). Updated by NATS health subscriber.
    pub backend_health: HashMap<String, BackendHealthState>,
    /// When true, main area shows Dashboard (left) and Positions (right) side-by-side; toggled with [p] or from config.
    pub split_pane: bool,
    /// Scroll/selection index for Positions tab (arrow-key scroll).
    pub positions_scroll: usize,
    /// When true, Positions tab groups by combo (account + strategy + symbol stem) and shows header + legs.
    pub positions_combo_view: bool,
    /// Combo keys that are expanded (show legs). Empty = all combos collapsed by default. Key = (account_id, strategy, symbol_stem).
    pub positions_expanded_combos: HashSet<(String, String, String)>,
    /// Scroll/selection index for Orders tab (arrow-key scroll; index into filtered list).
    pub orders_scroll: usize,
    /// Symbol currently selected for charting in the Charts tab.
    pub symbol_for_chart: String,
    /// Rolling live candle history keyed by symbol, fed by NATS candle updates.
    pub chart_history: HashMap<String, VecDeque<Candle>>,
    /// Charts search: input buffer (type to search).
    pub chart_search_input: String,
    /// Charts search: history of searched symbols.
    pub chart_search_history: VecDeque<String>,
    /// Charts search: visible dropdown (true while searching).
    pub chart_search_visible: bool,
    /// Charts search: selected result index.
    pub chart_search_selected: usize,
    /// Charts search: results from API.
    pub chart_search_results: Vec<String>,
    /// Charts search: last search timestamp for debounce (ms since Unix epoch).
    pub chart_search_last_search_ms: u64,
    /// Charts search: debounce interval (ms).
    pub chart_search_debounce_ms: u64,
    /// Charts pill navigation: which row is active (0 = expiry, 1 = strike width).
    pub chart_pill_row: usize,
    /// Charts pill navigation: selected expiry index.
    pub chart_expiry_index: usize,
    /// Charts pill navigation: selected strike width index.
    pub chart_strike_index: usize,
    /// Scroll/selection index for Alerts tab (arrow-key scroll).
    pub alerts_scroll: usize,
    /// Scroll/selection index for Dashboard symbols (arrow-key scroll).
    pub dashboard_scroll: usize,
    /// Scroll/selection index for Scenarios tab (arrow-key scroll).
    pub scenarios_scroll: usize,
    /// DTE window center for Scenarios (default 4). Range = center ± scenarios_dte_half_width.
    pub scenarios_dte_center: i32,
    /// Half-width of DTE window (default 2 → range 2–6). [ ] to contract/expand.
    pub scenarios_dte_half_width: i32,
    /// Strike width filter: None = all, Some(w) = only that width. 'w' to cycle 25/50/100/all.
    pub scenarios_strike_width_filter: Option<u32>,
    /// Selected symbol index for Yield tab (into effective watchlist).
    pub yield_symbol_index: usize,
    /// In-Settings watchlist override (add/remove symbols in memory). None = use config.watchlist.
    pub watchlist_override: Option<Vec<String>>,
    /// Selected row in Settings tab: 0 = backends, 1 = config, 2 = symbols. For symbol list, use settings_symbol_index.
    pub settings_section_index: usize,
    /// Selected symbol index in Settings watchlist (for remove / highlight).
    pub settings_symbol_index: usize,
    /// When Some, Settings is in "add symbol" mode; buffer for the new symbol (Enter confirm, Esc cancel).
    pub settings_add_symbol_input: Option<String>,
    /// When Some, Settings is in "edit config" mode for this key; buffer in settings_add_symbol_input.
    pub settings_edit_config_key: Option<String>,
    /// In-memory config overrides (key = NATS_URL, BACKEND_ID, etc.). Applied on top of file/env config.
    pub config_overrides: HashMap<String, String>,
    /// Selected config row in Settings (0..=4): NATS_URL, BACKEND_ID, TICK_MS, SNAPSHOT_TTL_SECS, SPLIT_PANE.
    pub settings_config_key_index: usize,
    /// Last fetched box spread curve (NATS api.finance_rates.build_curve).
    pub yield_curve: Option<CurveResponse>,
    /// Last fetched benchmark rates (NATS api.finance_rates.benchmarks).
    pub yield_benchmarks: Option<BenchmarksResponse>,
    /// Last yield fetch error message.
    pub yield_error: Option<String>,
    /// Cached NYSE market-open flag (None = not yet checked). Updated every ~60 ticks (~15s).
    pub market_open: Option<bool>,
    /// Tick counter for periodic market-hours check (resets at MARKET_CHECK_INTERVAL_TICKS).
    market_open_tick: u32,
    /// Tick counter for periodic yield fetch when on Yield tab.
    pub yield_fetch_tick: u32,
    /// True while a yield fetch is in flight; prevents overlapping requests and mock/real cycling.
    pub yield_fetch_pending: bool,
    /// Sender to trigger yield fetch (symbol); None when not wired.
    pub yield_fetch_tx: Option<mpsc::UnboundedSender<String>>,
    /// Last fetched loans list (NATS api.loans.list).
    pub loans_list: Option<Result<Vec<LoanRecord>, String>>,
    /// True while a loans fetch is in flight.
    pub loans_fetch_pending: bool,
    /// Scroll index for Loans tab.
    pub loans_scroll: usize,
    /// Sender to trigger loans fetch; None when not wired.
    pub loans_fetch_tx: Option<mpsc::UnboundedSender<()>>,
    /// Loan entry form state (when Some, show form overlay).
    pub loan_entry: Option<LoanEntryState>,
    /// Sender to create a new loan via NATS api.loans.create; None when not wired.
    pub loan_create_tx: Option<mpsc::UnboundedSender<LoanRecord>>,
    /// Latest metadata from the live market-data tick stream (source, priority, age).
    pub live_market_data_source: Option<MarketDataSourceMeta>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
    config_rx: watch::Receiver<TuiConfig>,
    health_rx: watch::Receiver<HashMap<String, BackendHealthState>>,
}

impl App {
    pub fn new(
        config: TuiConfig,
        snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
        event_rx: mpsc::UnboundedReceiver<AppEvent>,
        config_rx: watch::Receiver<TuiConfig>,
        health_rx: watch::Receiver<HashMap<String, BackendHealthState>>,
        yield_fetch_tx: Option<mpsc::UnboundedSender<String>>,
        loans_fetch_tx: Option<mpsc::UnboundedSender<()>>,
        loan_create_tx: Option<mpsc::UnboundedSender<LoanRecord>>,
    ) -> Self {
        let config_warning = validate_config_hint(&config);
        let split_pane = config.split_pane;
        Self {
            config,
            needs_redraw: true,
            active_tab: Tab::Dashboard,
            snapshot: std::cell::UnsafeCell::new(None),
            roi_history: HashMap::new(),
            order_filter: String::new(),
            order_filter_active: false,
            log_state: TuiWidgetState::default(),
            log_display_level: log::LevelFilter::Debug,
            nats_status: ConnectionStatus::new(ConnectionState::Starting, "Connecting to NATS"),
            should_quit: false,
            last_command_status: None,
            toast_queue: VecDeque::new(),
            show_help: false,
            show_log_panel: false,
            detail_popup: None,
            config_warning,
            backend_health: HashMap::new(),
            split_pane,
            positions_scroll: 0,
            positions_combo_view: false,
            positions_expanded_combos: HashSet::new(),
            orders_scroll: 0,
            symbol_for_chart: String::new(),
            chart_history: HashMap::new(),
            chart_search_input: String::new(),
            chart_search_history: VecDeque::with_capacity(10),
            chart_search_visible: false,
            chart_search_selected: 0,
            chart_search_results: Vec::new(),
            chart_search_last_search_ms: 0,
            chart_search_debounce_ms: 300,
            chart_pill_row: 0,
            chart_expiry_index: 0,
            chart_strike_index: 2,
            alerts_scroll: 0,
            dashboard_scroll: 0,
            scenarios_scroll: 0,
            scenarios_dte_center: 4,
            scenarios_dte_half_width: 2,
            scenarios_strike_width_filter: None,
            yield_symbol_index: 0,
            watchlist_override: None,
            settings_section_index: 0,
            settings_symbol_index: 0,
            settings_add_symbol_input: None,
            settings_edit_config_key: None,
            config_overrides: HashMap::new(),
            settings_config_key_index: 0,
            yield_curve: None,
            yield_benchmarks: None,
            yield_error: None,
            market_open: None,
            market_open_tick: 0,
            yield_fetch_tick: 0,
            yield_fetch_pending: false,
            yield_fetch_tx,
            loans_list: None,
            loans_fetch_pending: false,
            loans_scroll: 0,
            loans_fetch_tx,
            loan_entry: None,
            loan_create_tx,
            live_market_data_source: None,
            event_rx,
            snapshot_rx,
            config_rx,
            health_rx,
        }
    }

    /// Mark that the UI needs to be redrawn on the next render cycle.
    pub fn mark_dirty(&mut self) {
        self.needs_redraw = true;
    }

    /// Returns a reference to the current snapshot (for rendering).
    #[inline]
    pub fn snapshot(&self) -> &Option<TuiSnapshot> {
        unsafe { &*self.snapshot.get() }
    }

    /// Replaces the current snapshot (used by tick processing).
    #[inline]
    pub fn set_snapshot(&mut self, snap: Option<TuiSnapshot>) {
        unsafe {
            *self.snapshot.get() = snap;
        }
        self.hydrate_chart_history_from_snapshot();
    }

    /// Applies a tick market data event to the current snapshot.
    /// Creates a new symbol entry if this is the first tick for the symbol.
    /// Does nothing if no snapshot is loaded yet.
    #[inline]
    pub fn apply_tick(&mut self, symbol: &str, bid: f64, ask: f64, last: f64) {
        let snap_ptr = self.snapshot.get();
        if snap_ptr.is_null() {
            return;
        }
        let snap = unsafe { &mut *snap_ptr };
        if let Some(ref mut s) = snap {
            let mid = if last != 0.0 { last } else { (bid + ask) * 0.5 };
            if let Some(entry) = s.inner.symbols.iter_mut().find(|e| e.symbol == symbol) {
                entry.last = mid;
                entry.bid = bid;
                entry.ask = ask;
                entry.spread = (ask - bid).max(0.0);
            } else if bid != 0.0 || ask != 0.0 {
                s.inner.symbols.push(api::SymbolSnapshot {
                    symbol: symbol.to_string(),
                    last: mid,
                    bid,
                    ask,
                    spread: (ask - bid).max(0.0),
                    roi: 0.0,
                    maker_count: 1,
                    taker_count: 0,
                    volume: 0,
                    candle: api::CandleSnapshot {
                        open: mid,
                        high: mid,
                        low: mid,
                        close: mid,
                        volume: 0,
                        entry: mid,
                        updated: chrono::Utc::now(),
                    },
                });
            }

            for position in &mut s.inner.positions {
                if position.symbol == symbol {
                    position.mark = mid;
                    position.unrealized_pnl =
                        (mid - position.cost_basis) * position.quantity as f64;
                }
            }

            s.refresh_display_dto();
            self.mark_dirty();
        }
    }

    pub fn apply_candle(
        &mut self,
        symbol: &str,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
    ) {
        self.push_chart_candle(
            symbol,
            Candle {
                open,
                high,
                low,
                close,
                volume: Some(volume as f64),
            },
        );
        self.mark_dirty();
    }

    pub fn apply_alert(
        &mut self,
        level: api::AlertLevel,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) {
        let snap_ptr = self.snapshot.get();
        if snap_ptr.is_null() {
            return;
        }
        let snap = unsafe { &mut *snap_ptr };
        if let Some(ref mut s) = snap {
            s.inner.alerts.push(Alert {
                level,
                message,
                timestamp,
            });
            while s.inner.alerts.len() > 32 {
                s.inner.alerts.remove(0);
            }
            s.refresh_display_dto();
            self.mark_dirty();
        }
    }

    fn hydrate_chart_history_from_snapshot(&mut self) {
        let symbols = self
            .snapshot()
            .as_ref()
            .map(|snap| snap.inner.symbols.clone())
            .unwrap_or_default();

        for symbol in symbols {
            self.push_chart_candle(
                &symbol.symbol,
                Candle {
                    open: symbol.candle.open,
                    high: symbol.candle.high,
                    low: symbol.candle.low,
                    close: symbol.candle.close,
                    volume: Some(symbol.candle.volume as f64),
                },
            );
        }
    }

    fn push_chart_candle(&mut self, symbol: &str, candle: Candle) {
        let history = self.chart_history.entry(symbol.to_string()).or_default();

        if let Some(last) = history.back_mut() {
            if last.open == candle.open
                && last.high == candle.high
                && last.low == candle.low
                && last.close == candle.close
                && last.volume == candle.volume
            {
                return;
            }
        }

        history.push_back(candle);
        while history.len() > CHART_HISTORY_SIZE {
            history.pop_front();
        }
    }

    /// Effective watchlist: override if set, else config.
    pub fn watchlist(&self) -> &[String] {
        self.watchlist_override
            .as_deref()
            .unwrap_or(&self.config.watchlist)
    }

    pub fn input_mode(&self) -> InputMode {
        if self.show_help {
            InputMode::Help
        } else if self.detail_popup.is_some() {
            InputMode::DetailPopup
        } else if self.settings_add_symbol_input.is_some() {
            if self.settings_edit_config_key.is_some() {
                InputMode::SettingsEditConfig
            } else {
                InputMode::SettingsAddSymbol
            }
        } else if self.loan_entry.is_some() {
            InputMode::LoanForm
        } else if self.chart_search_visible {
            InputMode::ChartSearch
        } else if self.order_filter_active {
            InputMode::OrdersFilter
        } else if self.show_log_panel {
            InputMode::LogPanel
        } else {
            InputMode::Normal
        }
    }

    /// Set yield data from NATS fetch (curve + benchmarks).
    pub fn set_yield_data(&mut self, res: Result<(CurveResponse, BenchmarksResponse), String>) {
        self.yield_fetch_pending = false;
        self.yield_error = None;
        match res {
            Ok((curve, benchmarks)) => {
                self.yield_curve = Some(curve);
                self.yield_benchmarks = Some(benchmarks);
            }
            Err(e) => {
                self.yield_error = Some(e);
            }
        }
        self.mark_dirty();
    }

    /// Request a yield fetch for the given symbol (no-op if yield_fetch_tx is None or a fetch is already in flight).
    pub fn request_yield_fetch(&mut self, symbol: &str) {
        if self.yield_fetch_pending {
            return;
        }
        if let Some(ref tx) = self.yield_fetch_tx {
            if tx.send(symbol.to_string()).is_ok() {
                self.yield_fetch_pending = true;
            }
        }
    }

    /// Set loans list from NATS fetch.
    pub fn set_loans_data(&mut self, res: Result<Vec<LoanRecord>, String>) {
        self.loans_fetch_pending = false;
        self.loans_list = Some(res);
        self.mark_dirty();
    }

    /// Request a loans list fetch (no-op if already in flight or tx not wired).
    pub fn request_loans_fetch(&mut self) {
        if self.loans_fetch_pending {
            return;
        }
        if let Some(ref tx) = self.loans_fetch_tx {
            if tx.send(()).is_ok() {
                self.loans_fetch_pending = true;
            }
        }
    }

    /// Set the last strategy/control command result (shown in the status bar).
    pub fn set_command_status(&mut self, status: CommandStatusView) {
        self.last_command_status = Some(status);
        self.mark_dirty();
    }

    /// Push a transient toast notification. Toasts expire after TOAST_TTL_SECS seconds.
    pub fn push_toast(&mut self, msg: impl Into<String>, level: ToastLevel) {
        self.toast_queue
            .push_back((msg.into(), level, Instant::now()));
        self.mark_dirty();
    }

    /// Pull latest snapshot and config updates, process queued events.
    /// Returns true if the UI state changed and needs redraw.
    pub fn tick(&mut self) {
        let mut changed = false;

        // Drain expired toasts
        let ttl = std::time::Duration::from_secs(TOAST_TTL_SECS);
        let before = self.toast_queue.len();
        self.toast_queue.retain(|(_, _, ts)| ts.elapsed() < ttl);
        if self.toast_queue.len() != before {
            changed = true;
        }

        // Apply hot-reloaded config if it changed (file/env); then apply in-TUI overrides
        if self.config_rx.has_changed().unwrap_or(false) {
            let base = self.config_rx.borrow_and_update().clone();
            self.config = merge_config_overrides(base, &self.config_overrides);
            self.config_warning = validate_config_hint(&self.config);
            self.split_pane = self.config.split_pane;
            tracing::info!("Config reloaded from disk");
            changed = true;
        }

        // Apply backend health updates from system.health subscriber
        if self.health_rx.has_changed().unwrap_or(false) {
            self.backend_health = self.health_rx.borrow_and_update().clone();
        }

        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(event);
        }

        if self.snapshot_rx.has_changed().unwrap_or(false) {
            let next_snapshot = {
                let borrowed = self.snapshot_rx.borrow_and_update();
                borrowed.clone()
            };

            if let Some(snap) = next_snapshot {
                if self.should_accept_snapshot(&snap) {
                    self.set_snapshot(Some(snap.clone()));
                    self.update_roi_history(&snap);
                    changed = true;
                }
            }
        }

        // Clamp positions scroll to current display row count (flat or combo)
        if let Some(ref s) = self.snapshot() {
            let (display_len, _, _) = crate::ui::positions_display_info(
                &s.dto().positions,
                self.positions_combo_view,
                &self.positions_expanded_combos,
            );
            if display_len > 0 {
                self.positions_scroll = self.positions_scroll.min(display_len - 1);
            }
        }

        // Periodic NYSE market-hours check (~every 60s at 250ms tick).
        const MARKET_CHECK_INTERVAL_TICKS: u32 = 240;
        self.market_open_tick = self.market_open_tick.saturating_add(1);
        if self.market_open_tick == 1 || self.market_open_tick >= MARKET_CHECK_INTERVAL_TICKS {
            self.market_open_tick = 0;
            self.market_open = nyse_is_open();
        }

        // Periodic yield fetch when on Yield tab (~every 10s at 250ms tick). Skip if a fetch is already in flight to avoid mock/real cycling from overlapping responses.
        const YIELD_FETCH_INTERVAL_TICKS: u32 = 40;
        if self.active_tab == Tab::Yield
            && !self.yield_fetch_pending
            && !self.config.watchlist.is_empty()
        {
            self.yield_fetch_tick = self.yield_fetch_tick.saturating_add(1);
            if self.yield_fetch_tick >= YIELD_FETCH_INTERVAL_TICKS {
                self.yield_fetch_tick = 0;
                let idx = self.yield_symbol_index.min(self.config.watchlist.len() - 1);
                let symbol = self.config.watchlist[idx].clone();
                self.request_yield_fetch(&symbol);
            }
        }

        // When on Loans tab and no data yet, trigger a fetch once.
        if self.active_tab == Tab::Loans && self.loans_list.is_none() && !self.loans_fetch_pending {
            self.request_loans_fetch();
        }

        if changed {
            self.needs_redraw = true;
        }
    }

    fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Connection { target, status } => match target {
                ConnectionTarget::Nats => self.nats_status = status,
            },
            AppEvent::CommandStatus(reply) => {
                self.set_command_status(CommandStatusView::from_reply(&reply));
            }
            AppEvent::MarketTick {
                symbol,
                bid,
                ask,
                last,
                source,
                source_priority,
            } => {
                self.apply_tick(&symbol, bid, ask, last);
                self.live_market_data_source =
                    Some(MarketDataSourceMeta::new(source, source_priority));
            }
            AppEvent::MarketCandle {
                symbol,
                open,
                high,
                low,
                close,
                volume,
            } => {
                self.apply_candle(&symbol, open, high, low, close, volume);
            }
            AppEvent::AlertReceived {
                level,
                message,
                timestamp,
            } => {
                self.apply_alert(level, message, timestamp);
            }
        }
    }

    fn should_accept_snapshot(&self, _incoming: &TuiSnapshot) -> bool {
        // NATS-only: always accept the latest snapshot.
        true
    }

    /// Length of orders list after applying order_filter (for Orders tab selection clamp).
    pub fn filtered_orders_len(&self) -> usize {
        self.snapshot()
            .as_ref()
            .map(|s| self.filtered_orders(s).len())
            .unwrap_or(0)
    }

    /// Filter snapshot orders by order_filter (symbol, status, or side).
    /// Result is sorted by submitted_at descending (newest first).
    pub fn filtered_orders(&self, snap: &TuiSnapshot) -> Vec<RuntimeOrderDto> {
        let filter_lower = self.order_filter.to_lowercase();
        let mut orders = if filter_lower.is_empty() {
            snap.dto().orders.clone()
        } else {
            snap.dto()
                .orders
                .iter()
                .filter(|o| {
                    o.symbol.to_lowercase().contains(&filter_lower)
                        || o.status.to_lowercase().contains(&filter_lower)
                        || o.side.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect()
        };
        orders.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
        orders
    }
}

/// Returns `Some(true)` when NYSE is currently open, `Some(false)` when closed, `None` on error.
/// Uses NYSE as proxy for options market hours (CBOE follows NYSE schedule).
fn nyse_is_open() -> Option<bool> {
    use trading_calendar::{Market, TradingCalendar};
    let cal = TradingCalendar::new(Market::NYSE).ok()?;
    cal.is_open_now().ok()
}

/// Merge in-TUI config overrides on top of base (file/env) config.
fn merge_config_overrides(base: TuiConfig, overrides: &HashMap<String, String>) -> TuiConfig {
    let mut c = base;
    if let Some(v) = overrides.get("NATS_URL") {
        c.nats_url = v.trim().to_string();
    }
    if let Some(v) = overrides.get("BACKEND_ID") {
        let v = v.trim().to_lowercase();
        if !v.is_empty() {
            c.backend_id = v;
        }
    }
    if let Some(v) = overrides.get("TICK_MS") {
        if let Ok(n) = v.trim().parse::<u64>() {
            c.tick_ms = n.max(1);
        }
    }
    if let Some(v) = overrides.get("SNAPSHOT_TTL_SECS") {
        if let Ok(n) = v.trim().parse::<u64>() {
            c.snapshot_ttl_secs = n.max(1);
        }
    }
    if let Some(v) = overrides.get("SPLIT_PANE") {
        let v = v.trim().to_lowercase();
        c.split_pane = v == "1" || v == "true" || v == "yes";
    }
    c
}

/// Config keys editable from Settings (index 0..=4).
fn config_key_value_at(config: &TuiConfig, index: usize) -> Option<(String, String)> {
    let (key, value) = match index {
        0 => ("NATS_URL", config.nats_url.clone()),
        1 => ("BACKEND_ID", config.backend_id.clone()),
        2 => ("TICK_MS", config.tick_ms.to_string()),
        3 => ("SNAPSHOT_TTL_SECS", config.snapshot_ttl_secs.to_string()),
        4 => ("SPLIT_PANE", config.split_pane.to_string()),
        _ => return None,
    };
    Some((key.to_string(), value))
}

/// Returns a short validation hint if config is missing required fields.
fn validate_config_hint(config: &TuiConfig) -> Option<String> {
    let mut issues = Vec::new();
    if config.nats_url.trim().is_empty() {
        issues.push("NATS_URL empty");
    }
    if config.backend_id.trim().is_empty() {
        issues.push("BACKEND_ID empty");
    }
    if issues.is_empty() {
        None
    } else {
        Some(issues.join("; "))
    }
}

impl App {
    /// Config key/value for Settings config list (index 0..=4).
    pub fn config_key_value_at(&self, index: usize) -> Option<(String, String)> {
        config_key_value_at(&self.config, index)
    }

    fn update_roi_history(&mut self, snap: &TuiSnapshot) {
        for symbol_data in &snap.dto().symbols {
            let roi = symbol_data.roi;
            let entry = self
                .roi_history
                .entry(symbol_data.symbol.clone())
                .or_default();
            entry.push_back(roi);
            while entry.len() > SPARKLINE_HISTORY_SIZE {
                entry.pop_front();
            }
        }
        let current_symbols: std::collections::HashSet<_> = snap
            .inner
            .symbols
            .iter()
            .map(|s| s.symbol.clone())
            .collect();
        self.roi_history.retain(|k, _| current_symbols.contains(k));
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.needs_redraw = true;

        match self.input_mode() {
            InputMode::Help => {
                self.show_help = false;
                return;
            }
            InputMode::DetailPopup => {
                self.detail_popup = None;
                return;
            }
            InputMode::SettingsEditConfig | InputMode::SettingsAddSymbol => {
                if let Some(ref mut buf) = self.settings_add_symbol_input {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(ref key_name) = self.settings_edit_config_key {
                                let val = buf.trim().to_string();
                                if !val.is_empty() {
                                    self.config_overrides.insert(key_name.clone(), val);
                                    let base = self.config_rx.borrow().clone();
                                    self.config =
                                        merge_config_overrides(base, &self.config_overrides);
                                    self.config_warning = validate_config_hint(&self.config);
                                    self.split_pane = self.config.split_pane;
                                    self.set_command_status(CommandStatusView::success(
                                        "settings",
                                        format!("Saved override for {}.", key_name),
                                    ));
                                }
                                self.settings_edit_config_key = None;
                            } else {
                                let s = buf.trim().to_uppercase();
                                if !s.is_empty() {
                                    let mut list = self
                                        .watchlist_override
                                        .clone()
                                        .unwrap_or_else(|| self.config.watchlist.clone());
                                    if !list.contains(&s) {
                                        list.push(s.clone());
                                        list.sort();
                                        self.watchlist_override = Some(list);
                                        self.push_toast(
                                            format!("Symbol {} added to watchlist.", s),
                                            ToastLevel::Info,
                                        );
                                    }
                                }
                            }
                            self.settings_add_symbol_input = None;
                        }
                        KeyCode::Esc => {
                            self.set_command_status(CommandStatusView::success(
                                "settings",
                                "Edit cancelled.",
                            ));
                            self.settings_edit_config_key = None;
                            self.settings_add_symbol_input = None;
                        }
                        KeyCode::Backspace => {
                            buf.pop();
                        }
                        KeyCode::Char(c) if !c.is_control() => {
                            buf.push(c);
                        }
                        _ => {}
                    }
                }
                return;
            }
            _ => {}
        }

        if let Some(action) = crate::input::key_to_action(self, key) {
            crate::input::apply_action(self, action);
        }
    }
}

#[cfg(test)]
mod tests {
    use api::finance_rates::{CurveResponse, RatePointResponse};
    use api::{Alert, AlertLevel, OrderSnapshot, SystemSnapshot};
    use chrono::{Duration, Utc};
    use crossterm::event::{KeyCode, KeyEvent};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tokio::sync::{mpsc, watch};

    use std::collections::HashMap;

    use super::{App, InputMode, Tab};
    use crate::{
        config::TuiConfig,
        events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget},
        models::SnapshotSource,
        models::TuiSnapshot,
        ui::{charts::render_charts, render},
    };

    fn make_app() -> (
        App,
        watch::Sender<Option<TuiSnapshot>>,
        mpsc::UnboundedSender<AppEvent>,
    ) {
        let (snap_tx, snap_rx) = watch::channel(None);
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (_config_tx, config_rx) = watch::channel(TuiConfig::default());
        let (_health_tx, health_rx) = watch::channel(HashMap::new());
        let app = App::new(
            TuiConfig::default(),
            snap_rx,
            event_rx,
            config_rx,
            health_rx,
            None,
            None,
            None,
        );
        (app, snap_tx, event_tx)
    }

    fn make_snapshot() -> TuiSnapshot {
        let mut snap = TuiSnapshot::new(SystemSnapshot::default(), SnapshotSource::Nats);
        snap.inner.alerts.clear();
        snap.refresh_display_dto();
        snap
    }

    #[test]
    fn input_mode_prefers_settings_edit_over_base_flags() {
        let (mut app, _, _) = make_app();
        app.settings_edit_config_key = Some("NATS_URL".into());
        app.settings_add_symbol_input = Some("nats://demo".into());

        assert_eq!(app.input_mode(), InputMode::SettingsEditConfig);
    }

    #[test]
    fn input_mode_reports_chart_search_when_active() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Charts;
        app.chart_search_visible = true;

        assert_eq!(app.input_mode(), InputMode::ChartSearch);
    }

    #[test]
    fn app_updates_connection_status() {
        let (mut app, _, event_tx) = make_app();

        event_tx
            .send(AppEvent::Connection {
                target: ConnectionTarget::Nats,
                status: ConnectionStatus::new(ConnectionState::Retrying, "Connection refused"),
            })
            .expect("send connection status");

        app.tick();

        assert_eq!(app.nats_status.state, ConnectionState::Retrying);
        assert_eq!(app.nats_status.detail, "Connection refused");
    }

    #[test]
    fn config_hot_reload_updates_app_config() {
        let (snap_tx, snap_rx) = watch::channel(None);
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let (config_tx, config_rx) = watch::channel(TuiConfig::default());
        let (_health_tx, health_rx) = watch::channel(HashMap::new());
        let mut app = App::new(
            TuiConfig::default(),
            snap_rx,
            event_rx,
            config_rx,
            health_rx,
            None,
            None,
            None,
        );
        drop(snap_tx);

        let new_config = TuiConfig {
            watchlist: vec!["TSLA".into()],
            ..Default::default()
        };
        config_tx.send(new_config).expect("send new config");

        app.tick();

        assert_eq!(app.config.watchlist, vec!["TSLA"]);
    }

    #[test]
    fn log_tab_keys_do_not_panic() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Logs;

        // Verify scroll/filter keys are handled without panicking
        for key in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Char('+'),
            KeyCode::Char('-'),
            KeyCode::Esc,
        ] {
            app.handle_key(KeyEvent::from(key));
        }
    }

    #[test]
    fn positions_and_alerts_scroll_keys_do_not_panic() {
        let (mut app, _, _) = make_app();

        app.active_tab = Tab::Positions;
        for key in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::PageUp,
            KeyCode::PageDown,
        ] {
            app.handle_key(KeyEvent::from(key));
        }

        app.active_tab = Tab::Alerts;
        for key in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::PageUp,
            KeyCode::PageDown,
        ] {
            app.handle_key(KeyEvent::from(key));
        }
    }

    /// Flatten the drawn buffer to a single string (one line per row) for assertion.
    fn buffer_to_string(area: &ratatui::layout::Rect, buffer: &ratatui::buffer::Buffer) -> String {
        let mut s = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                s.push_str(buffer[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }

    #[test]
    fn yield_curve_tab_renders_with_data() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Yield;
        app.yield_curve = Some(CurveResponse {
            symbol: "SPX".to_string(),
            points: vec![RatePointResponse {
                symbol: "SPX".to_string(),
                expiry: "2026-03-20".to_string(),
                days_to_expiry: 30,
                strike_width: 5.0,
                strike_low: None,
                strike_high: None,
                buy_implied_rate: 0.04,
                sell_implied_rate: 0.05,
                mid_rate: 0.045,
                net_debit: 4.5,
                net_credit: 4.4,
                liquidity_score: 70.0,
                timestamp: String::new(),
                spread_id: None,
                convenience_yield: None,
                data_source: None,
            }],
            timestamp: String::new(),
            strike_width: None,
            point_count: 1,
            underlying_price: None,
        });
        app.yield_benchmarks = None;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal
            .draw(|f| crate::ui::render_yield_curve_tab(f, &app, f.area()))
            .unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(
            content.contains("Yield"),
            "Yield tab should show 'Yield' title; got:\n{}",
            content
        );
        assert!(
            content.contains("Rate %"),
            "Yield tab should show 'Rate %' column header; got:\n{}",
            content
        );
        assert!(
            content.contains("SPX"),
            "Yield tab should show symbol SPX; got:\n{}",
            content
        );
        assert!(
            content.contains("4.50") || content.contains("4.5"),
            "Yield tab should show mid rate 4.50% (or truncated 4.5) for one point; got:\n{}",
            content
        );
    }

    #[test]
    fn yield_curve_tab_renders_empty_state() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Yield;
        app.yield_curve = Some(CurveResponse {
            symbol: "SPX".to_string(),
            points: vec![],
            timestamp: String::new(),
            strike_width: None,
            point_count: 0,
            underlying_price: None,
        });
        app.yield_benchmarks = None;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal
            .draw(|f| crate::ui::render_yield_curve_tab(f, &app, f.area()))
            .unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Yield"), "Yield title; got:\n{}", content);
        assert!(
            content.contains("No data") || content.contains("No "),
            "Empty curve should show 'No data' (or truncated 'No'); got:\n{}",
            content
        );
        assert!(
            content.contains("Box spread curve (empty)"),
            "Empty curve should show empty title; got:\n{}",
            content
        );
    }

    #[test]
    fn charts_tab_shows_waiting_state_without_history() {
        let (mut app, _, _) = make_app();
        app.symbol_for_chart = "SPX".to_string();
        app.active_tab = Tab::Charts;

        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render_charts(f, &app, f.area())).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Waiting for live candle data for SPX."));
        assert!(content.contains("Synthetic candles are disabled."));
        assert!(content.contains("Waiting for the first backend snapshot"));
    }

    #[test]
    fn charts_tab_shows_stale_snapshot_warning_without_history() {
        let (mut app, _, _) = make_app();
        let mut snap = make_snapshot();
        snap.received_at = Utc::now() - Duration::seconds(45);
        app.set_snapshot(Some(snap));
        app.symbol_for_chart = "SPX".to_string();
        app.active_tab = Tab::Charts;

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render_charts(f, &app, f.area())).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Waiting for live candle data for SPX."));
        assert!(content.contains("Synthetic candles are disabled."));
        assert!(content.contains("Latest snapshot is stale"));
    }

    #[test]
    fn alerts_tab_displays_placeholder_when_no_snapshot() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Alerts;

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render(f, &app)).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("No alerts"));
    }

    #[test]
    fn alerts_tab_renders_live_alert_messages() {
        let (mut app, _, _) = make_app();
        let mut snap = make_snapshot();
        snap.inner.alerts = vec![
            Alert {
                level: AlertLevel::Info,
                message: "provider switched to polygon".into(),
                timestamp: Utc::now() - Duration::seconds(5),
            },
            Alert {
                level: AlertLevel::Warning,
                message: "SPX quote is stale".into(),
                timestamp: Utc::now(),
            },
        ];
        snap.refresh_display_dto();
        app.set_snapshot(Some(snap));
        app.active_tab = Tab::Alerts;

        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render(f, &app)).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("provider switched to polygon"));
        assert!(content.contains("SPX quote is stale"));
    }

    #[test]
    fn help_overlay_documents_mode_aware_bindings() {
        let (mut app, _, _) = make_app();
        app.show_help = true;

        let backend = TestBackend::new(100, 32);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render(f, &app)).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Key bindings"));
    }

    #[test]
    fn split_pane_renders_visible_mode_label() {
        let (mut app, _, _) = make_app();
        app.split_pane = true;

        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render(f, &app)).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Split pane"));
        assert!(content.contains("Dashboard + Positions"));
        assert!(content.contains("PANE:DASH+POS"));
    }

    #[test]
    fn orders_tab_renders_filter_mode_cues() {
        let (mut app, _, _) = make_app();
        let mut snap = make_snapshot();
        snap.inner.orders = vec![OrderSnapshot {
            id: "ord-1".into(),
            symbol: "SPY".into(),
            side: "BUY".into(),
            quantity: 3,
            status: "Submitted".into(),
            submitted_at: Utc::now(),
        }];
        snap.refresh_display_dto();
        app.set_snapshot(Some(snap));
        app.active_tab = Tab::Orders;
        app.order_filter_active = true;
        app.order_filter = "SPY".into();

        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render(f, &app)).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Orders [FILTER]"));
        assert!(content.contains("symbol/status/side"));
        assert!(content.contains("SPY"));
    }

    #[test]
    fn settings_tab_renders_config_edit_label_and_prompt() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Settings;
        app.settings_section_index = 1;
        app.settings_edit_config_key = Some("NATS_URL".into());
        app.settings_add_symbol_input = Some("nats://demo".into());

        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render(f, &app)).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Config overrides (editing NATS_URL)"));
        assert!(content.contains("Edit NATS_URL:"));
        assert!(content.contains("Active section: Config"));
    }

    #[test]
    fn hint_bar_renders_async_status_cues() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Yield;
        app.yield_fetch_pending = true;
        app.loans_fetch_pending = true;

        let backend = TestBackend::new(240, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let frame = terminal.draw(|f| render(f, &app)).unwrap();

        let content = buffer_to_string(&frame.area, &frame.buffer);
        assert!(content.contains("Yield:loading"));
        assert!(content.contains("Loans:loading"));
    }
}
