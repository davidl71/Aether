//! Application state and event dispatch.

use std::collections::{HashMap, HashSet, VecDeque};

use api::finance_rates::{BenchmarksResponse, CurveResponse};
use api::loans::LoanRecord;
use api::{BackendHealthState, RuntimeOrderDto, RuntimePositionDto, ScenarioDto};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::{mpsc, watch};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::config::TuiConfig;
use crate::events::{
    AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget, StrategyCommand,
};
use crate::models::TuiSnapshot;

const SPARKLINE_HISTORY_SIZE: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Positions,
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

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Dashboard,
        Tab::Positions,
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

    fn next(&self) -> Tab {
        let i = (self.index() + 1) % Tab::ALL.len();
        Tab::ALL[i].clone()
    }

    fn prev(&self) -> Tab {
        let i = (self.index() + Tab::ALL.len() - 1) % Tab::ALL.len();
        Tab::ALL[i].clone()
    }
}

pub struct App {
    pub config: TuiConfig,
    pub active_tab: Tab,
    pub snapshot: Option<TuiSnapshot>,
    /// Per-symbol ROI history for sparkline visualization (symbol -> deque of ROI values)
    pub roi_history: HashMap<String, VecDeque<f64>>,
    /// Order filter text (filters orders by symbol or status)
    pub order_filter: String,
    /// State for the tui-logger widget (scroll position, level filter).
    pub log_state: TuiWidgetState,
    pub nats_status: ConnectionStatus,
    pub should_quit: bool,
    /// Last result from strategy start/stop/cancel-all (shown in hint bar until cleared). Ok = message, Err = error.
    pub last_strategy_result: Option<Result<String, String>>,
    /// When true, show the help overlay (key bindings).
    pub show_help: bool,
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
    /// Scroll/selection index for Alerts tab (arrow-key scroll).
    pub alerts_scroll: usize,
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
    /// Tick counter for periodic yield fetch when on Yield tab.
    pub yield_fetch_tick: u32,
    /// True while a yield fetch is in flight; prevents overlapping requests and mock/real cycling.
    pub yield_fetch_pending: bool,
    /// Sender to trigger yield fetch (symbol); None when not wired.
    yield_fetch_tx: Option<mpsc::UnboundedSender<String>>,
    /// Last fetched loans list (NATS api.loans.list).
    pub loans_list: Option<Result<Vec<LoanRecord>, String>>,
    /// True while a loans fetch is in flight.
    pub loans_fetch_pending: bool,
    /// Scroll index for Loans tab.
    pub loans_scroll: usize,
    /// Sender to trigger loans fetch; None when not wired.
    loans_fetch_tx: Option<mpsc::UnboundedSender<()>>,
    /// Sender for strategy commands (S=start, T=stop); None when not wired.
    strategy_cmd_tx: Option<mpsc::UnboundedSender<StrategyCommand>>,
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
        strategy_cmd_tx: Option<mpsc::UnboundedSender<StrategyCommand>>,
        yield_fetch_tx: Option<mpsc::UnboundedSender<String>>,
        loans_fetch_tx: Option<mpsc::UnboundedSender<()>>,
    ) -> Self {
        let config_warning = validate_config_hint(&config);
        let split_pane = config.split_pane;
        Self {
            config,
            active_tab: Tab::Dashboard,
            snapshot: None,
            roi_history: HashMap::new(),
            order_filter: String::new(),
            log_state: TuiWidgetState::default(),
            nats_status: ConnectionStatus::new(ConnectionState::Starting, "Connecting to NATS"),
            should_quit: false,
            last_strategy_result: None,
            show_help: false,
            detail_popup: None,
            config_warning,
            backend_health: HashMap::new(),
            split_pane,
            positions_scroll: 0,
            positions_combo_view: false,
            positions_expanded_combos: HashSet::new(),
            orders_scroll: 0,
            alerts_scroll: 0,
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
            yield_fetch_tick: 0,
            yield_fetch_pending: false,
            yield_fetch_tx,
            loans_list: None,
            loans_fetch_pending: false,
            loans_scroll: 0,
            loans_fetch_tx,
            strategy_cmd_tx,
            event_rx,
            snapshot_rx,
            config_rx,
            health_rx,
        }
    }

    /// Effective watchlist: override if set, else config.
    pub fn watchlist(&self) -> &[String] {
        self.watchlist_override
            .as_deref()
            .unwrap_or(&self.config.watchlist)
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

    /// Set the last strategy command result (shown in hint bar). Ok(msg) = success message, Err(e) = error.
    pub fn set_strategy_result(&mut self, r: Result<String, String>) {
        self.last_strategy_result = Some(r);
    }

    /// Pull latest snapshot and config updates, process queued events.
    pub fn tick(&mut self) {
        // Apply hot-reloaded config if it changed (file/env); then apply in-TUI overrides
        if self.config_rx.has_changed().unwrap_or(false) {
            let base = self.config_rx.borrow_and_update().clone();
            self.config = merge_config_overrides(base, &self.config_overrides);
            self.config_warning = validate_config_hint(&self.config);
            self.split_pane = self.config.split_pane;
            tracing::info!("Config reloaded from disk");
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
                    self.snapshot = Some(snap.clone());
                    self.update_roi_history(&snap);
                }
            }
        }

        // Clamp positions scroll to current display row count (flat or combo)
        if let Some(ref s) = self.snapshot {
            let (display_len, _, _) = crate::ui::positions_display_info(
                &s.dto().positions,
                self.positions_combo_view,
                &self.positions_expanded_combos,
            );
            if display_len > 0 {
                self.positions_scroll = self.positions_scroll.min(display_len - 1);
            }
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
    }

    fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Connection { target, status } => match target {
                ConnectionTarget::Nats => self.nats_status = status,
            },
        }
    }

    fn should_accept_snapshot(&self, _incoming: &TuiSnapshot) -> bool {
        // NATS-only: always accept the latest snapshot.
        true
    }

    /// Length of orders list after applying order_filter (for Orders tab selection clamp).
    fn filtered_orders_len(&self) -> usize {
        self.snapshot
            .as_ref()
            .map(|s| self.filtered_orders(s).len())
            .unwrap_or(0)
    }

    /// Filter snapshot orders by order_filter (symbol, status, or side).
    /// Result is sorted by submitted_at descending (newest first).
    fn filtered_orders(&self, snap: &TuiSnapshot) -> Vec<RuntimeOrderDto> {
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
        if self.show_help {
            self.show_help = false;
            return;
        }
        if self.detail_popup.is_some() {
            self.detail_popup = None;
            return;
        }
        // Settings input mode: add symbol or edit config
        if let Some(ref mut buf) = self.settings_add_symbol_input {
            match key.code {
                KeyCode::Enter => {
                    if let Some(ref key_name) = self.settings_edit_config_key {
                        // Apply config override
                        let val = buf.trim().to_string();
                        if !val.is_empty() {
                            self.config_overrides.insert(key_name.clone(), val);
                            let base = self.config_rx.borrow().clone();
                            self.config = merge_config_overrides(base, &self.config_overrides);
                            self.config_warning = validate_config_hint(&self.config);
                            self.split_pane = self.config.split_pane;
                        }
                        self.settings_edit_config_key = None;
                    } else {
                        // Add symbol
                        let s = buf.trim().to_uppercase();
                        if !s.is_empty() {
                            let mut list = self
                                .watchlist_override
                                .clone()
                                .unwrap_or_else(|| self.config.watchlist.clone());
                            if !list.contains(&s) {
                                list.push(s);
                                list.sort();
                                self.watchlist_override = Some(list);
                            }
                        }
                    }
                    self.settings_add_symbol_input = None;
                }
                KeyCode::Esc => {
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
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            // Yield tab: ←/→ change symbol (before generic tab switch)
            KeyCode::Left if self.active_tab == Tab::Yield => {
                let len = self.watchlist().len();
                if len > 0 {
                    self.yield_symbol_index = (self.yield_symbol_index + len - 1) % len;
                    let symbol = self.watchlist()[self.yield_symbol_index].clone();
                    self.request_yield_fetch(&symbol);
                }
            }
            KeyCode::Right if self.active_tab == Tab::Yield => {
                let len = self.watchlist().len();
                if len > 0 {
                    self.yield_symbol_index = (self.yield_symbol_index + 1) % len;
                    let symbol = self.watchlist()[self.yield_symbol_index].clone();
                    self.request_yield_fetch(&symbol);
                }
            }
            KeyCode::Tab | KeyCode::Right => self.active_tab = self.active_tab.next(),
            KeyCode::BackTab | KeyCode::Left => self.active_tab = self.active_tab.prev(),
            KeyCode::Char('1') => self.active_tab = Tab::Dashboard,
            KeyCode::Char('2') => self.active_tab = Tab::Positions,
            KeyCode::Char('3') => self.active_tab = Tab::Orders,
            KeyCode::Char('4') => self.active_tab = Tab::Alerts,
            KeyCode::Char('5') => {
                self.active_tab = Tab::Yield;
                let wl = self.watchlist();
                if !wl.is_empty() {
                    let idx = self.yield_symbol_index.min(wl.len().saturating_sub(1));
                    let symbol = wl[idx].clone();
                    self.request_yield_fetch(&symbol);
                }
            }
            KeyCode::Char('6') => {
                self.active_tab = Tab::Loans;
                self.request_loans_fetch();
            }
            KeyCode::Char('7') => self.active_tab = Tab::Scenarios,
            KeyCode::Char('8') => self.active_tab = Tab::Logs,
            KeyCode::Char('9') => self.active_tab = Tab::Settings,
            // Log tab navigation — forwarded to TuiWidgetState
            KeyCode::Up if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::UpKey);
            }
            KeyCode::Down if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::DownKey);
            }
            KeyCode::PageUp if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::PrevPageKey);
            }
            KeyCode::PageDown if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::NextPageKey);
            }
            // Log level filter (canonical tui-logger keys)
            KeyCode::Char('+') if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::PlusKey);
            }
            KeyCode::Char('-') if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::MinusKey);
            }
            KeyCode::Char('h') if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::HideKey);
            }
            KeyCode::Esc if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::EscapeKey);
            }
            // Positions tab (or right pane when split): [c] combo view, arrow-key scroll
            KeyCode::Char('c') | KeyCode::Char('C')
                if self.active_tab == Tab::Positions || self.split_pane =>
            {
                self.positions_combo_view = !self.positions_combo_view;
                self.positions_scroll = 0;
            }
            KeyCode::Up if self.active_tab == Tab::Positions || self.split_pane => {
                self.positions_scroll = self.positions_scroll.saturating_sub(1);
            }
            KeyCode::Down if self.active_tab == Tab::Positions || self.split_pane => {
                let len = self
                    .snapshot
                    .as_ref()
                    .map(|s| {
                        crate::ui::positions_display_info(
                            &s.dto().positions,
                            self.positions_combo_view,
                            &self.positions_expanded_combos,
                        )
                        .0
                    })
                    .unwrap_or(0);
                if len > 0 {
                    self.positions_scroll = (self.positions_scroll + 1).min(len - 1);
                }
            }
            KeyCode::PageUp if self.active_tab == Tab::Positions || self.split_pane => {
                self.positions_scroll = self.positions_scroll.saturating_sub(10);
            }
            KeyCode::PageDown if self.active_tab == Tab::Positions || self.split_pane => {
                let len = self
                    .snapshot
                    .as_ref()
                    .map(|s| {
                        crate::ui::positions_display_info(
                            &s.dto().positions,
                            self.positions_combo_view,
                            &self.positions_expanded_combos,
                        )
                        .0
                    })
                    .unwrap_or(0);
                if len > 0 {
                    self.positions_scroll = (self.positions_scroll + 10).min(len - 1);
                }
            }
            // Orders tab: arrow-key scroll (selection index into filtered list)
            KeyCode::Up if self.active_tab == Tab::Orders => {
                self.orders_scroll = self.orders_scroll.saturating_sub(1);
            }
            KeyCode::Down if self.active_tab == Tab::Orders => {
                let len = self.filtered_orders_len();
                if len > 0 {
                    self.orders_scroll = (self.orders_scroll + 1).min(len - 1);
                }
            }
            KeyCode::PageUp if self.active_tab == Tab::Orders => {
                self.orders_scroll = self.orders_scroll.saturating_sub(10);
            }
            KeyCode::PageDown if self.active_tab == Tab::Orders => {
                let len = self.filtered_orders_len();
                if len > 0 {
                    self.orders_scroll = (self.orders_scroll + 10).min(len - 1);
                }
            }
            KeyCode::Enter if self.active_tab == Tab::Orders => {
                if let Some(ref snap) = self.snapshot {
                    let filtered = self.filtered_orders(snap);
                    let idx = self.orders_scroll.min(filtered.len().saturating_sub(1));
                    if let Some(order) = filtered.get(idx) {
                        self.detail_popup = Some(DetailPopupContent::Order(order.clone()));
                    }
                }
            }
            KeyCode::Up if self.active_tab == Tab::Loans => {
                self.loans_scroll = self.loans_scroll.saturating_sub(1);
            }
            KeyCode::Down if self.active_tab == Tab::Loans => {
                let len = self
                    .loans_list
                    .as_ref()
                    .and_then(|r| r.as_ref().ok())
                    .map(|l| l.len())
                    .unwrap_or(0);
                if len > 0 {
                    self.loans_scroll = (self.loans_scroll + 1).min(len - 1);
                }
            }
            KeyCode::PageUp if self.active_tab == Tab::Loans => {
                self.loans_scroll = self.loans_scroll.saturating_sub(10);
            }
            KeyCode::PageDown if self.active_tab == Tab::Loans => {
                let len = self
                    .loans_list
                    .as_ref()
                    .and_then(|r| r.as_ref().ok())
                    .map(|l| l.len())
                    .unwrap_or(0);
                if len > 0 {
                    self.loans_scroll = (self.loans_scroll + 10).min(len - 1);
                }
            }
            KeyCode::Enter if self.active_tab == Tab::Positions || self.split_pane => {
                if let Some(ref snap) = self.snapshot {
                    let (_display_len, index_map, combo_key_per_row) =
                        crate::ui::positions_display_info(
                            &snap.dto().positions,
                            self.positions_combo_view,
                            &self.positions_expanded_combos,
                        );
                    if let Some(Some(combo_key)) = combo_key_per_row.get(self.positions_scroll) {
                        if self.positions_expanded_combos.contains(combo_key) {
                            self.positions_expanded_combos.remove(combo_key);
                        } else {
                            self.positions_expanded_combos.insert(combo_key.clone());
                        }
                    } else if let Some(Some(pos_idx)) = index_map.get(self.positions_scroll) {
                        if let Some(pos) = snap.dto().positions.get(*pos_idx) {
                            self.detail_popup = Some(DetailPopupContent::Position(pos.clone()));
                        }
                    }
                }
            }
            // Alerts tab: arrow-key scroll
            KeyCode::Up if self.active_tab == Tab::Alerts => {
                self.alerts_scroll = self.alerts_scroll.saturating_sub(1);
            }
            KeyCode::Down if self.active_tab == Tab::Alerts => {
                let len = self
                    .snapshot
                    .as_ref()
                    .map(|s| s.dto().alerts.len())
                    .unwrap_or(0);
                if len > 0 {
                    self.alerts_scroll = (self.alerts_scroll + 1).min(len - 1);
                }
            }
            KeyCode::PageUp if self.active_tab == Tab::Alerts => {
                self.alerts_scroll = self.alerts_scroll.saturating_sub(10);
            }
            KeyCode::PageDown if self.active_tab == Tab::Alerts => {
                let len = self
                    .snapshot
                    .as_ref()
                    .map(|s| s.dto().alerts.len())
                    .unwrap_or(0);
                if len > 0 {
                    self.alerts_scroll = (self.alerts_scroll + 10).min(len - 1);
                }
            }
            // Scenarios tab: arrow-key scroll
            KeyCode::Up if self.active_tab == Tab::Scenarios => {
                self.scenarios_scroll = self.scenarios_scroll.saturating_sub(1);
            }
            KeyCode::Down if self.active_tab == Tab::Scenarios => {
                let filtered = crate::ui::filtered_scenarios(self);
                if !filtered.is_empty() {
                    self.scenarios_scroll =
                        (self.scenarios_scroll + 1).min(filtered.len().saturating_sub(1));
                }
            }
            KeyCode::PageUp if self.active_tab == Tab::Scenarios => {
                self.scenarios_scroll = self.scenarios_scroll.saturating_sub(10);
            }
            KeyCode::PageDown if self.active_tab == Tab::Scenarios => {
                let filtered = crate::ui::filtered_scenarios(self);
                if !filtered.is_empty() {
                    self.scenarios_scroll =
                        (self.scenarios_scroll + 10).min(filtered.len().saturating_sub(1));
                }
            }
            KeyCode::Enter if self.active_tab == Tab::Scenarios => {
                let filtered = crate::ui::filtered_scenarios(self);
                let idx = self.scenarios_scroll.min(filtered.len().saturating_sub(1));
                if let Some(scenario) = filtered.get(idx) {
                    self.detail_popup = Some(DetailPopupContent::Scenario(scenario.clone()));
                }
            }
            KeyCode::Char('[') if self.active_tab == Tab::Scenarios => {
                self.scenarios_dte_half_width = (self.scenarios_dte_half_width - 1).max(0);
            }
            KeyCode::Char(']') if self.active_tab == Tab::Scenarios => {
                self.scenarios_dte_half_width = (self.scenarios_dte_half_width + 1).min(60);
            }
            KeyCode::Char('w') | KeyCode::Char('W') if self.active_tab == Tab::Scenarios => {
                self.scenarios_strike_width_filter = match self.scenarios_strike_width_filter {
                    None => Some(25),
                    Some(25) => Some(50),
                    Some(50) => Some(100),
                    Some(_) => None,
                };
            }
            // Settings tab: section/symbol scroll, config key scroll (section 1), add symbol (a), edit config (e), remove (Del), reset override (r)
            KeyCode::Up if self.active_tab == Tab::Settings => {
                if self.settings_section_index == 2 {
                    self.settings_symbol_index = self.settings_symbol_index.saturating_sub(1);
                } else if self.settings_section_index == 1 {
                    self.settings_config_key_index =
                        self.settings_config_key_index.saturating_sub(1);
                } else {
                    self.settings_section_index = self.settings_section_index.saturating_sub(1);
                }
            }
            KeyCode::Down if self.active_tab == Tab::Settings => {
                if self.settings_section_index == 2 {
                    let len = self.watchlist().len();
                    if len > 0 {
                        self.settings_symbol_index =
                            (self.settings_symbol_index + 1).min(len.saturating_sub(1));
                    }
                } else if self.settings_section_index == 1 {
                    self.settings_config_key_index = (self.settings_config_key_index + 1).min(4);
                } else {
                    self.settings_section_index = (self.settings_section_index + 1).min(2);
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') if self.active_tab == Tab::Settings => {
                if self.settings_section_index != 2 {
                    return;
                }
                self.settings_add_symbol_input = Some(String::new());
            }
            KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter
                if self.active_tab == Tab::Settings && self.settings_section_index == 1 =>
            {
                if let Some((key, value)) = self.config_key_value_at(self.settings_config_key_index)
                {
                    self.settings_edit_config_key = Some(key);
                    self.settings_add_symbol_input = Some(value);
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') if self.active_tab == Tab::Settings => {
                self.watchlist_override = None;
            }
            KeyCode::Delete if self.active_tab == Tab::Settings => {
                let wl = self.watchlist();
                if !wl.is_empty() && self.settings_symbol_index < wl.len() {
                    let mut list = self
                        .watchlist_override
                        .clone()
                        .unwrap_or_else(|| self.config.watchlist.clone());
                    list.remove(self.settings_symbol_index);
                    let new_len = list.len();
                    self.watchlist_override = Some(list);
                    self.settings_symbol_index =
                        self.settings_symbol_index.min(new_len.saturating_sub(1));
                }
            }
            // Order filter: '/' to activate, chars to add, Backspace to delete, Esc to clear
            // Mode cycle: M → Live / Mock / DRY-RUN (api.admin.set_mode)
            KeyCode::Char('m') | KeyCode::Char('M') => {
                if let Some(ref tx) = self.strategy_cmd_tx {
                    let current = self
                        .snapshot
                        .as_ref()
                        .map(|s| s.dto().mode.to_uppercase())
                        .unwrap_or_else(|| "DRY-RUN".into());
                    let current = if current == "TUI" {
                        "DRY-RUN"
                    } else {
                        current.as_str()
                    };
                    let next = match current {
                        "LIVE" => "MOCK",
                        "MOCK" => "DRY-RUN",
                        _ => "LIVE",
                    };
                    let _ = tx.send(StrategyCommand::SetMode(next.to_string()));
                }
            }
            // Strategy control via NATS (S=start, T=stop); skip when on Orders tab so s/t can filter
            KeyCode::Char('s') | KeyCode::Char('S') if self.active_tab != Tab::Orders => {
                if let Some(ref tx) = self.strategy_cmd_tx {
                    let _ = tx.send(StrategyCommand::Start);
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') if self.active_tab != Tab::Orders => {
                if let Some(ref tx) = self.strategy_cmd_tx {
                    let _ = tx.send(StrategyCommand::Stop);
                }
            }
            KeyCode::Char('k') | KeyCode::Char('K') if self.active_tab != Tab::Orders => {
                if let Some(ref tx) = self.strategy_cmd_tx {
                    let _ = tx.send(StrategyCommand::CancelAll);
                }
            }
            // Force snapshot: backend publishes current snapshot once to NATS (api.snapshot.publish_now)
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if let Some(ref tx) = self.strategy_cmd_tx {
                    let _ = tx.send(StrategyCommand::PublishSnapshot);
                }
            }
            KeyCode::Char('/') if self.active_tab == Tab::Orders => {
                self.order_filter.clear();
            }
            KeyCode::Char(c) => {
                if self.active_tab == Tab::Orders {
                    self.order_filter.push(c);
                }
            }
            KeyCode::Backspace if self.active_tab == Tab::Orders => {
                self.order_filter.pop();
            }
            KeyCode::Esc if self.active_tab == Tab::Orders => {
                self.order_filter.clear();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use api::finance_rates::{CurveResponse, RatePointResponse};
    use crossterm::event::{KeyCode, KeyEvent};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tokio::sync::{mpsc, watch};

    use std::collections::HashMap;

    use super::{App, Tab};
    use crate::{
        config::TuiConfig,
        events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget},
        models::TuiSnapshot,
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
        );
        (app, snap_tx, event_tx)
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
            content.contains("APR %"),
            "Yield tab should show 'APR %' column header; got:\n{}",
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
}
