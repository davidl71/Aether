//! Application state and event dispatch.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

use api::discount_bank::{DiscountBankBalanceDto, DiscountBankTransactionsListDto};
use api::finance_rates::{BenchmarksResponse, CurveResponse, RatePointResponse};
use api::loans::{LoanRecord, LoansBulkImportResponse};
use api::{
    Alert, BackendHealthState, CommandReply, CommandStatus, NatsTransportHealthState,
    RuntimeOrderDto, RuntimePositionDto, ScenarioDto,
};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::{mpsc, watch};
use tui_logger::TuiWidgetState;

use crate::app_config::{
    config_key_value_at, merge_config_overrides, nyse_is_open, validate_config_hint,
};
use crate::config::TuiConfig;
use crate::events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget};
use crate::mode::AppMode;
use crate::models::TuiSnapshot;
use crate::pane::pane_spec;
use crate::scrollable_table::ScrollableTableState;
use crate::ui::{Candle, ToastLevel, ToastManager};
use crate::workspace::{
    SecondaryFocus, SettingsHealthFocus, SettingsSection, VisibleWorkspace, WorkspaceSpec,
    SPLIT_PANE_TABS,
};

const SPARKLINE_HISTORY_SIZE: usize = 20;
const CHART_HISTORY_SIZE: usize = 120;

/// Loans worker → TUI: plain list refresh or bulk import with server summary.
#[derive(Debug)]
pub enum LoansUiOutcome {
    Plain(Result<Vec<LoanRecord>, String>),
    BulkSuccess {
        summary: LoansBulkImportResponse,
        list: Result<Vec<LoanRecord>, String>,
    },
}
// ── Yield-refresh task tracking ─────────────────────────────────────────────

/// Status of a manual yield-refresh request.
#[derive(Debug, Clone, PartialEq)]
pub enum RefreshTaskStatus {
    /// Request sent to backend; waiting for KV update.
    Pending,
    /// KV update received; data is fresh.
    Done,
    /// Request failed or backend returned an error.
    Error(String),
}

/// One manual or periodic refresh event tracked in the TUI task queue.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RefreshTask {
    /// Monotone id (incrementing u64).
    pub id: u64,
    /// Human-readable label, e.g. "Yield refresh SPX,XSP".
    pub label: String,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: RefreshTaskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Positions,
    Charts,
    Orders,
    Alerts,
    Yield,
    Loans,
    DiscountBank,
    Scenarios,
    Logs,
    Settings,
}

/// Display-ready greeks result for an option position.
#[derive(Debug, Clone)]
pub struct GreeksDisplay {
    pub iv: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

/// Combined FMP quote + most recent income statement for a symbol.
#[derive(Debug, Clone)]
pub struct FmpDetail {
    pub symbol: String,
    pub price: Option<f64>,
    pub day_high: Option<f64>,
    pub day_low: Option<f64>,
    pub prev_close: Option<f64>,
    pub eps: Option<f64>,
    pub revenue: Option<f64>,
    pub net_income: Option<f64>,
}

/// Request sent to the background greeks fetcher task.
#[derive(Debug, Clone)]
pub struct GreeksFetchRequest {
    pub underlying: f64,
    pub strike: f64,
    pub tte_years: f64,
    pub rate: f64,
    pub market_price: f64,
    pub option_type: String,
}

/// Content for the row-detail overlay (Orders/Positions/Scenarios). Same overlay pattern as help (?); Esc to close.
#[derive(Debug, Clone)]
pub enum DetailPopupContent {
    Order(RuntimeOrderDto),
    Position(RuntimePositionDto, Option<GreeksDisplay>),
    Scenario(ScenarioDto),
    YieldPoint(RatePointResponse),
    FmpSymbol(FmpDetail),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Help,
    DetailPopup,
    SettingsEditConfig,
    SettingsAddSymbol,
    SettingsCredentialEntry,
    LoanForm,
    LoanImportPath,
    ChartSearch,
    OrdersFilter,
    LogPanel,
    TreePanel,
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
    /// When false, failed status is shown in the status bar only (no error toast).
    pub toast_on_failure: bool,
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
            toast_on_failure: true,
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
            toast_on_failure: true,
        }
    }

    pub fn disabled(action: impl Into<String>) -> Self {
        Self {
            command_id: None,
            issued_at: None,
            action: action.into(),
            status: CommandStatus::Failed,
            message: None,
            error: Some(
                "Order and strategy-run controls are off in exploration mode; inspect data and settings as usual."
                    .into(),
            ),
            toast_on_failure: false,
        }
    }

    pub fn success(action: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            command_id: None,
            issued_at: None,
            action: action.into(),
            status: CommandStatus::Completed,
            message: Some(message.into()),
            error: None,
            toast_on_failure: true,
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

    fn is_fresh(&self, ttl_secs: u64) -> bool {
        self.age_secs() <= ttl_secs
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
    pub currency: String,
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
            currency: "ILS".to_string(),
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
            currency: if self.currency.is_empty() {
                "ILS".to_string()
            } else {
                self.currency.clone()
            },
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
        Tab::DiscountBank,
        Tab::Scenarios,
        Tab::Settings,
    ];

    pub fn label(&self) -> &'static str {
        pane_spec(*self).label
    }

    pub fn title(&self) -> &'static str {
        pane_spec(*self).title
    }

    fn index(&self) -> usize {
        let primary = match self {
            Tab::Logs => Tab::Alerts,
            other => *other,
        };
        Tab::ALL.iter().position(|t| t == &primary).unwrap_or(0)
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
    pub needs_redraw: bool,
    pub dirty_flags: crate::dirty_flags::DirtyFlags,
    pub active_tab: Tab,
    /// Last rendered main-content area size; input uses this to decide whether wide workspaces
    /// are actually visible before hijacking focus navigation.
    last_main_area_size: Cell<(u16, u16)>,
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
    /// Current application mode (Navigation/Edit/View).
    pub app_mode: AppMode,
    /// Last operator-facing action outcome for the status/hint bar (includes disabled no-ops for legacy strategy keys).
    pub last_command_status: Option<CommandStatusView>,
    /// Toast notification manager for user feedback.
    pub toast_manager: ToastManager,
    /// Command palette for discoverability.
    pub command_palette: crate::discoverability::CommandPalette,
    /// When true, show the help overlay (key bindings).
    pub show_help: bool,
    /// When true, show the debug log panel overlay (toggled with backtick).
    pub show_log_panel: bool,

    /// When true, show the tree panel overlay (tui-tree-widget spike).
    pub show_tree_panel: bool,
    /// Stateful selection/open state for the tree panel.
    pub tree_state: RefCell<tui_tree_widget::TreeState<&'static str>>,
    /// Spike tree items; next step is mapping domain objects.
    pub tree_items: Vec<tui_tree_widget::TreeItem<'static, &'static str>>,
    /// When Some, show detail overlay for selected Order or Position (Enter to open, Esc to close).
    pub detail_popup: Option<DetailPopupContent>,
    /// Config validation warning (e.g. missing NATS_URL); shown in status bar when set.
    pub config_warning: Option<String>,
    /// Backend health from system.health (backend id → state). Updated by NATS health subscriber.
    pub backend_health: HashMap<String, BackendHealthState>,
    /// First-class NATS transport health for the subscriber path.
    pub nats_transport: NatsTransportHealthState,
    /// Alpaca connectivity for paper API credentials (read-only exploration; not order flow).
    pub alpaca_paper_status: ConnectionStatus,
    /// Alpaca connectivity for live API credentials (read-only exploration; not order flow).
    pub alpaca_live_status: ConnectionStatus,
    /// Alpaca health monitor for tracking API connectivity.
    pub alpaca_health: crate::alpaca_health::AlpacaHealthMonitor,
    /// When true, main area shows Dashboard (left) and Positions (right) side-by-side; toggled with [p] or from config.
    pub split_pane: bool,
    /// Positions tab: selected display row (combo or flat).
    pub positions_table: ScrollableTableState,
    /// When true, Positions tab groups by combo (account + strategy + symbol stem) and shows header + legs.
    pub positions_combo_view: bool,
    /// Combo keys that are expanded (show legs). Empty = all combos collapsed by default. Key = (account_id, strategy, symbol_stem).
    pub positions_expanded_combos: HashSet<(String, String, String)>,
    /// Orders tab: selected row index into filtered list.
    pub orders_table: ScrollableTableState,
    /// Symbol currently selected for charting in the Charts tab.
    pub symbol_for_chart: String,
    /// Rolling live candle history keyed by symbol, fed by NATS candle updates.
    pub chart_history: HashMap<String, VecDeque<Candle>>,
    /// Charts search: input buffer (type to search).
    pub chart_search_input: String,
    /// Charts search: history of searched symbols (loaded from / saved to config dir; see `chart_search_history`).
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
    /// Alerts tab: paragraph scroll offset (only [`ScrollableTableState::scroll`] is used).
    pub alerts_scroll: ScrollableTableState,
    /// Dashboard: selected symbol row.
    pub dashboard_table: ScrollableTableState,
    /// Scenarios tab: selected row in filtered list.
    pub scenarios_table: ScrollableTableState,
    /// DTE window center for Scenarios (default 4). Range = center ± scenarios_dte_half_width.
    pub scenarios_dte_center: i32,
    /// Half-width of DTE window (default 2 → range 2–6). [ ] to contract/expand.
    pub scenarios_dte_half_width: i32,
    /// Strike width filter: None = all, Some(w) = only that width. 'w' to cycle 25/50/100/all.
    pub scenarios_strike_width_filter: Option<u32>,
    /// Selected symbol index for Yield tab (into effective watchlist).
    pub yield_symbol_index: usize,
    /// Yield curve points table: selected row (↑↓, Enter for detail).
    pub yield_curve_table: ScrollableTableState,
    /// In-Settings watchlist override (add/remove symbols in memory). None = use config.watchlist.
    pub watchlist_override: Option<Vec<String>>,
    /// Active secondary focus target inside the Settings pane.
    pub settings_section: SettingsSection,
    /// Nested focus target within Settings -> Health.
    pub settings_health_focus: SettingsHealthFocus,
    /// Selected symbol index in Settings watchlist (for remove / highlight).
    pub settings_symbol_index: usize,
    /// When Some, Settings is in "add symbol" mode; buffer for the new symbol (Enter confirm, Esc cancel).
    pub settings_add_symbol_input: Option<String>,
    /// When Some, Settings is in "edit config" mode for this key; buffer in settings_add_symbol_input.
    pub settings_edit_config_key: Option<String>,
    /// In-memory config overrides (key = NATS_URL, BACKEND_ID, etc.). Applied on top of file/env config.
    pub config_overrides: HashMap<String, String>,
    /// Selected config row in Settings (visible runtime config entries list).
    pub settings_config_key_index: usize,
    /// Selected row in Settings → Alpaca credential table.
    pub settings_alpaca_row: usize,
    /// Selected row in Settings → Data sources table (FMP, Polygon, …).
    pub settings_sources_row: usize,
    /// When set with `settings_credential_buffer`, user is editing this API key (Alpaca or Sources row).
    pub settings_credential_edit_key: Option<api::credentials::CredentialKey>,
    pub settings_credential_buffer: Option<String>,
    /// Last fetched box spread curve for the selected symbol.
    pub yield_curve: Option<CurveResponse>,
    /// Box spread curves for all watchlist symbols (keyed by symbol), populated by batch fetch.
    pub yield_curves_all: HashMap<String, CurveResponse>,
    /// Last fetched benchmark rates (NATS api.finance_rates.benchmarks).
    pub yield_benchmarks: Option<BenchmarksResponse>,
    /// Last yield fetch error message.
    pub yield_error: Option<String>,
    /// Per-symbol: timestamp of the last KV update received (from curve.timestamp).
    pub yield_last_refreshed: HashMap<String, chrono::DateTime<chrono::Utc>>,
    /// Recent refresh tasks (max 8, newest first) for tracking manual requests.
    pub yield_tasks: VecDeque<RefreshTask>,
    /// Monotone counter for task ids.
    yield_task_id_counter: u64,
    /// True while a manual refresh request is in-flight (sent but KV update not yet received).
    pub yield_refresh_pending: bool,
    /// Sender to trigger a refresh request publish to `api.yield_curve.refresh`; None if not wired.
    pub yield_refresh_tx: Option<mpsc::UnboundedSender<()>>,
    /// Cached NYSE market-open flag (None = not yet checked). Updated every ~60 ticks (~15s).
    pub market_open: Option<bool>,
    /// Tick counter for periodic market-hours check (resets at MARKET_CHECK_INTERVAL_TICKS).
    market_open_tick: u32,
    /// Cached loans list (`api.loans.list`). Refreshed after NATS outcomes (fetch/create/import);
    /// tab navigation uses [`Self::request_loans_fetch_if_uncached`] to avoid redundant fetches.
    pub loans_list: Option<Result<Vec<LoanRecord>, String>>,
    /// True while a loans fetch is in flight.
    pub loans_fetch_pending: bool,
    /// True after user submits bulk import path until the worker reports an outcome.
    pub loans_bulk_import_inflight: bool,
    /// Loans tab: selected row.
    pub loans_table: ScrollableTableState,
    /// Sender to trigger loans fetch; None when not wired.
    pub loans_fetch_tx: Option<mpsc::UnboundedSender<()>>,
    /// Loan entry form state (when Some, show form overlay).
    pub loan_entry: Option<LoanEntryState>,
    /// Sender to create a new loan via NATS api.loans.create; None when not wired.
    pub loan_create_tx: Option<mpsc::UnboundedSender<LoanRecord>>,
    /// Buffer for bulk-import file path on Loans tab (`b` / `i`); None when not editing.
    pub loan_import_path: Option<String>,
    /// Sender to bulk-import loans from a JSON file path (reads file, NATS `api.loans.import_bulk`).
    pub loan_bulk_import_tx: Option<mpsc::UnboundedSender<std::path::PathBuf>>,
    /// Last fetched Discount Bank balance (NATS api.discount_bank.balance).
    pub discount_bank_balance: Option<Result<DiscountBankBalanceDto, String>>,
    /// Last fetched Discount Bank transactions (NATS api.discount_bank.transactions).
    pub discount_bank_transactions: Option<Result<DiscountBankTransactionsListDto, String>>,
    /// True while a Discount Bank fetch is in flight.
    pub discount_bank_fetch_pending: bool,
    /// Discount Bank tab: selected row.
    pub discount_bank_table: ScrollableTableState,
    /// Sender to trigger Discount Bank fetch; None when not wired.
    pub discount_bank_fetch_tx: Option<mpsc::UnboundedSender<()>>,
    /// Sender to trigger FMP quote+fundamentals fetch (symbol string); None when not wired.
    pub fmp_fetch_tx: Option<mpsc::UnboundedSender<String>>,
    /// Sender to trigger greeks fetch for an option position; None when not wired.
    pub greeks_fetch_tx: Option<mpsc::UnboundedSender<GreeksFetchRequest>>,
    /// Latest metadata from the live market-data tick stream (source, priority, age).
    pub live_market_data_source: Option<MarketDataSourceMeta>,
    /// Per-provider activity cache used to choose the freshest/highest-priority live source.
    market_data_sources: HashMap<String, MarketDataSourceMeta>,
    /// Cached credential presence per provider (refreshed every ~30s). True = key found.
    pub credential_status: HashMap<&'static str, bool>,
    /// Cached credential source per provider (`env`, `keyring`, `file`) when present.
    pub credential_source: HashMap<&'static str, &'static str>,
    /// Instant when credential_status was last refreshed.
    credential_status_refreshed_at: Option<Instant>,
    /// Messages received on `strategy.signal.>` (diagnostic NATS subscription).
    pub strategy_nats_signal_count: u64,
    /// Messages received on `strategy.decision.>` (diagnostic NATS subscription).
    pub strategy_nats_decision_count: u64,
    /// Truncated summary of the last strategy signal or decision (Settings → health).
    pub strategy_nats_last: String,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
    config_rx: watch::Receiver<TuiConfig>,
    health_rx: watch::Receiver<HashMap<String, BackendHealthState>>,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: TuiConfig,
        snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
        event_rx: mpsc::UnboundedReceiver<AppEvent>,
        config_rx: watch::Receiver<TuiConfig>,
        health_rx: watch::Receiver<HashMap<String, BackendHealthState>>,
        yield_refresh_tx: Option<mpsc::UnboundedSender<()>>,
        loans_fetch_tx: Option<mpsc::UnboundedSender<()>>,
        loan_create_tx: Option<mpsc::UnboundedSender<LoanRecord>>,
        loan_bulk_import_tx: Option<mpsc::UnboundedSender<std::path::PathBuf>>,
        fmp_fetch_tx: Option<mpsc::UnboundedSender<String>>,
        greeks_fetch_tx: Option<mpsc::UnboundedSender<GreeksFetchRequest>>,
        discount_bank_fetch_tx: Option<mpsc::UnboundedSender<()>>,
    ) -> Self {
        let config_warning = validate_config_hint(&config);
        let split_pane = config.split_pane;
        Self {
            config,
            needs_redraw: true,
            dirty_flags: crate::dirty_flags::DirtyFlags::new(),
            active_tab: Tab::Dashboard,
            last_main_area_size: Cell::new((0, 0)),
            snapshot: std::cell::UnsafeCell::new(None),
            roi_history: HashMap::new(),
            order_filter: String::new(),
            order_filter_active: false,
            log_state: TuiWidgetState::default(),
            log_display_level: log::LevelFilter::Debug,
            nats_status: ConnectionStatus::new(ConnectionState::Starting, "Connecting to NATS"),
            should_quit: false,
            app_mode: AppMode::default(),
            last_command_status: None,
            toast_manager: ToastManager::new(),
            command_palette: crate::discoverability::CommandPalette::new(),
            show_help: false,
            show_log_panel: false,
            show_tree_panel: false,
            tree_state: RefCell::new(tui_tree_widget::TreeState::default()),
            tree_items: Vec::new(),
            detail_popup: None,
            config_warning,
            backend_health: HashMap::new(),
            nats_transport: NatsTransportHealthState::default(),
            alpaca_paper_status: ConnectionStatus::new(ConnectionState::Retrying, "Not configured"),
            alpaca_live_status: ConnectionStatus::new(ConnectionState::Retrying, "Not configured"),
            alpaca_health: crate::alpaca_health::AlpacaHealthMonitor::new(),
            split_pane,
            positions_table: ScrollableTableState::default(),
            positions_combo_view: false,
            positions_expanded_combos: HashSet::new(),
            orders_table: ScrollableTableState::default(),
            symbol_for_chart: String::new(),
            chart_history: HashMap::new(),
            chart_search_input: String::new(),
            chart_search_history: crate::chart_search_history::load_chart_search_history(),
            chart_search_visible: false,
            chart_search_selected: 0,
            chart_search_results: Vec::new(),
            chart_search_last_search_ms: 0,
            chart_search_debounce_ms: 300,
            chart_pill_row: 0,
            chart_expiry_index: 0,
            chart_strike_index: 2,
            alerts_scroll: ScrollableTableState::default(),
            dashboard_table: ScrollableTableState::default(),
            scenarios_table: ScrollableTableState::default(),
            scenarios_dte_center: 4,
            scenarios_dte_half_width: 2,
            scenarios_strike_width_filter: None,
            yield_symbol_index: 0,
            yield_curve_table: ScrollableTableState::default(),
            watchlist_override: None,
            settings_section: SettingsSection::Health,
            settings_health_focus: SettingsHealthFocus::Transport,
            settings_symbol_index: 0,
            settings_add_symbol_input: None,
            settings_edit_config_key: None,
            config_overrides: HashMap::new(),
            settings_config_key_index: 0,
            settings_alpaca_row: 0,
            settings_sources_row: 0,
            settings_credential_edit_key: None,
            settings_credential_buffer: None,
            yield_curve: None,
            yield_curves_all: HashMap::new(),
            yield_benchmarks: None,
            yield_error: None,
            yield_last_refreshed: HashMap::new(),
            yield_tasks: VecDeque::new(),
            yield_task_id_counter: 0,
            yield_refresh_pending: false,
            yield_refresh_tx,
            market_open: None,
            market_open_tick: 0,
            loans_list: None,
            loans_fetch_pending: false,
            loans_bulk_import_inflight: false,
            loans_table: ScrollableTableState::default(),
            loans_fetch_tx,
            loan_entry: None,
            loan_create_tx,
            loan_import_path: None,
            loan_bulk_import_tx,
            discount_bank_balance: None,
            discount_bank_transactions: None,
            discount_bank_fetch_pending: false,
            discount_bank_table: ScrollableTableState::default(),
            discount_bank_fetch_tx,
            fmp_fetch_tx,
            greeks_fetch_tx,
            live_market_data_source: None,
            market_data_sources: HashMap::new(),
            credential_status: HashMap::new(),
            credential_source: HashMap::new(),
            credential_status_refreshed_at: None,
            strategy_nats_signal_count: 0,
            strategy_nats_decision_count: 0,
            strategy_nats_last: String::new(),
            event_rx,
            snapshot_rx,
            config_rx,
            health_rx,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.needs_redraw = true;
        self.dirty_flags.mark_all();
    }

    /// Marks [`needs_redraw`](Self::needs_redraw) and records which UI regions changed for selective [`crate::ui::render`].
    pub fn mark_regions(&mut self, update: impl FnOnce(&mut crate::dirty_flags::DirtyFlags)) {
        self.needs_redraw = true;
        update(&mut self.dirty_flags);
    }

    /// Returns a reference to the current snapshot (for rendering).
    #[inline]
    pub fn snapshot(&self) -> &Option<TuiSnapshot> {
        unsafe { &*self.snapshot.get() }
    }

    pub fn set_last_main_area_size(&self, width: u16, height: u16) {
        self.last_main_area_size.set((width, height));
    }

    pub fn visible_workspace(&self) -> VisibleWorkspace {
        let (width, height) = self.last_main_area_size.get();
        if self.split_pane && SPLIT_PANE_TABS.contains(&self.active_tab) {
            return VisibleWorkspace::SplitPane;
        }
        for mode in [
            VisibleWorkspace::Market,
            VisibleWorkspace::Operations,
            VisibleWorkspace::Credit,
        ] {
            if let Some(spec) = mode.spec() {
                if spec.tabs.contains(&self.active_tab)
                    && width >= spec.min_width
                    && height >= spec.min_height
                {
                    return mode;
                }
            }
        }
        VisibleWorkspace::None
    }

    pub fn visible_workspace_spec(&self) -> Option<WorkspaceSpec> {
        self.visible_workspace().spec()
    }

    pub fn secondary_focus(&self) -> SecondaryFocus {
        match self.active_tab {
            Tab::Settings => match self.settings_section {
                SettingsSection::Health => {
                    SecondaryFocus::SettingsHealth(self.settings_health_focus)
                }
                section => SecondaryFocus::Settings(section),
            },
            _ => SecondaryFocus::None,
        }
    }

    pub fn focus_label(&self) -> String {
        self.secondary_focus()
            .title()
            .unwrap_or_else(|| self.active_tab.title().to_string())
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
        } else if self.settings_credential_buffer.is_some() {
            InputMode::SettingsCredentialEntry
        } else if self.settings_add_symbol_input.is_some() {
            if self.settings_edit_config_key.is_some() {
                InputMode::SettingsEditConfig
            } else {
                InputMode::SettingsAddSymbol
            }
        } else if self.loan_entry.is_some() {
            InputMode::LoanForm
        } else if self.loan_import_path.is_some() {
            InputMode::LoanImportPath
        } else if self.chart_search_visible {
            InputMode::ChartSearch
        } else if self.order_filter_active {
            InputMode::OrdersFilter
        } else if self.show_log_panel {
            InputMode::LogPanel
        } else if self.show_tree_panel {
            InputMode::TreePanel
        } else {
            InputMode::Normal
        }
    }

    /// Map focus/input surface to high-level [`AppMode`] (single place for routing and tests).
    pub fn app_mode_for_input_mode(input_mode: InputMode) -> AppMode {
        match input_mode {
            InputMode::SettingsEditConfig
            | InputMode::SettingsAddSymbol
            | InputMode::SettingsCredentialEntry
            | InputMode::LoanForm
            | InputMode::LoanImportPath
            | InputMode::ChartSearch
            | InputMode::OrdersFilter => AppMode::Edit,
            InputMode::Help
            | InputMode::DetailPopup
            | InputMode::LogPanel
            | InputMode::TreePanel => AppMode::View,
            InputMode::Normal => AppMode::Navigation,
        }
    }

    /// Update `app_mode` from current [`InputMode`]. On change, pushes a short info toast.
    pub fn update_app_mode(&mut self) {
        let next = Self::app_mode_for_input_mode(self.input_mode());
        if next != self.app_mode {
            self.push_toast(
                format!("Mode {} → {}", self.app_mode.label(), next.label()),
                crate::ui::ToastLevel::Info,
            );
            // Mode icon/label is on the status bar; toast only marks overlay.
            self.mark_regions(|d| d.mark_status_bar());
        }
        self.app_mode = next;
    }

    /// Legacy: set yield data from NATS request/reply fetch (curve + benchmarks).
    /// Kept for test compatibility; production path now uses KV watch + periodic benchmarks fetch.
    pub fn set_yield_data(
        &mut self,
        res: Result<(HashMap<String, CurveResponse>, BenchmarksResponse), String>,
    ) {
        self.yield_refresh_pending = false;
        self.yield_error = None;
        match res {
            Ok((curves, benchmarks)) => {
                self.yield_curves_all.extend(curves);
                self.sync_yield_curve_from_cache();
                self.yield_benchmarks = Some(benchmarks);
            }
            Err(e) => {
                self.yield_error = Some(e);
            }
        }
        self.mark_regions(|d| {
            d.mark_content();
            d.mark_status_bar();
        });
    }

    /// Trigger a yield refresh: publishes to `api.yield_curve.refresh` (fire-and-forget).
    /// The KV watcher receives the writer's response and updates `yield_curves_all` via AppEvent.
    pub fn request_yield_fetch(&mut self, _symbol: &str) {
        if self.yield_refresh_pending {
            return;
        }
        if let Some(ref tx) = self.yield_refresh_tx {
            if !self.config.watchlist.is_empty() && tx.send(()).is_ok() {
                self.yield_refresh_pending = true;
                // Create a tracking task
                self.yield_task_id_counter += 1;
                let symbols_label = self.config.watchlist.join(",");
                const MAX_TASKS: usize = 8;
                if self.yield_tasks.len() >= MAX_TASKS {
                    self.yield_tasks.pop_back();
                }
                self.yield_tasks.push_front(RefreshTask {
                    id: self.yield_task_id_counter,
                    label: format!("Yield refresh {}", symbols_label),
                    triggered_at: chrono::Utc::now(),
                    completed_at: None,
                    status: RefreshTaskStatus::Pending,
                });
            }
        }
    }

    /// Update `yield_curve` (detail view) to match `yield_symbol_index` from cached data.
    /// Called after navigation or after a KV update arrives for the selected symbol.
    pub fn sync_yield_curve_from_cache(&mut self) {
        let idx = self
            .yield_symbol_index
            .min(self.config.watchlist.len().saturating_sub(1));
        self.yield_curve = self
            .config
            .watchlist
            .get(idx)
            .and_then(|s| self.yield_curves_all.get(s))
            .cloned();
    }

    /// Set loans list from a worker message (plain refresh, bulk import, or error).
    pub fn apply_loans_outcome(&mut self, outcome: LoansUiOutcome) {
        self.loans_fetch_pending = false;
        match outcome {
            LoansUiOutcome::Plain(res) => {
                if self.loans_bulk_import_inflight {
                    self.loans_bulk_import_inflight = false;
                    if let Err(ref e) = res {
                        self.push_toast(format!("Bulk import failed: {}", e), ToastLevel::Error);
                    }
                }
                self.loans_list = Some(res);
                self.mark_regions(|d| d.mark_content());
            }
            LoansUiOutcome::BulkSuccess { summary, list } => {
                self.loans_bulk_import_inflight = false;
                let n_skip = summary.errors.len();
                if n_skip > 0 {
                    self.push_toast(
                        format!(
                            "Bulk import: applied {} loan(s); {} row(s) skipped.",
                            summary.applied, n_skip
                        ),
                        ToastLevel::Warning,
                    );
                } else {
                    self.push_toast(
                        format!("Bulk import: applied {} loan(s).", summary.applied),
                        ToastLevel::Info,
                    );
                }
                if let Err(ref e) = list {
                    self.push_toast(
                        format!("Loan list refresh failed: {}", e),
                        ToastLevel::Warning,
                    );
                }
                self.loans_list = Some(list);
                self.mark_regions(|d| d.mark_content());
            }
        }
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

    /// Request `api.loans.list` only when there is no successful in-memory list yet.
    ///
    /// After writes, workers push fresh lists via `apply_loans_outcome`. Use this from tab
    /// switches and bulk-import UI so refocusing Loans does not refetch every time.
    pub fn request_loans_fetch_if_uncached(&mut self) {
        if matches!(self.loans_list, Some(Ok(_))) {
            return;
        }
        self.request_loans_fetch();
    }

    /// Set Discount Bank data from NATS fetch (balance + transactions).
    pub fn set_discount_bank_data(
        &mut self,
        balance: Result<DiscountBankBalanceDto, String>,
        txns: Result<DiscountBankTransactionsListDto, String>,
    ) {
        self.discount_bank_fetch_pending = false;
        self.discount_bank_balance = Some(balance);
        self.discount_bank_transactions = Some(txns);
        self.mark_regions(|d| d.mark_content());
    }

    /// Request a Discount Bank fetch (no-op if already in flight or tx not wired).
    pub fn request_discount_bank_fetch(&mut self) {
        if self.discount_bank_fetch_pending {
            return;
        }
        if let Some(ref tx) = self.discount_bank_fetch_tx {
            if tx.send(()).is_ok() {
                self.discount_bank_fetch_pending = true;
            }
        }
    }

    /// Apply incoming FMP data result.
    pub fn set_fmp_data(&mut self, result: Result<FmpDetail, String>) {
        match result {
            Ok(data) => {
                self.detail_popup = Some(DetailPopupContent::FmpSymbol(data));
            }
            Err(e) => {
                self.push_toast(format!("FMP: {}", e), ToastLevel::Error);
            }
        }
        self.mark_regions(|d| d.mark_overlay());
    }

    /// Apply incoming greeks result — updates the open Position popup if present.
    pub fn set_greeks_data(&mut self, result: Result<GreeksDisplay, String>) {
        if let Some(DetailPopupContent::Position(_, greeks_slot)) = &mut self.detail_popup {
            *greeks_slot = result.ok();
        }
        self.mark_regions(|d| d.mark_overlay());
    }

    /// Trigger FMP fetch for a symbol.
    pub fn fetch_fmp(&mut self, symbol: String) {
        if let Some(ref tx) = self.fmp_fetch_tx {
            let _ = tx.send(symbol);
        }
    }

    /// Trigger greeks fetch for an option position if parseable.
    pub fn fetch_greeks_for_position(&mut self, pos: &RuntimePositionDto) {
        use crate::option_symbol::parse_option_symbol;
        let parsed = match parse_option_symbol(&pos.symbol) {
            Some(p) => p,
            None => return,
        };
        let underlying = self
            .snapshot()
            .as_ref()
            .and_then(|s| {
                s.inner
                    .symbols
                    .iter()
                    .find(|sym| pos.symbol.starts_with(&sym.symbol))
            })
            .map(|s| s.last)
            .unwrap_or(0.0);
        if underlying == 0.0 {
            return;
        }
        let rate = self
            .yield_curve
            .as_ref()
            .and_then(|c| c.points.first())
            .map(|p| p.mid_rate)
            .unwrap_or(0.045);
        let req = GreeksFetchRequest {
            underlying,
            strike: parsed.strike,
            tte_years: parsed.tte_years,
            rate,
            market_price: pos.mark,
            option_type: parsed.option_type,
        };
        if let Some(ref tx) = self.greeks_fetch_tx {
            let _ = tx.send(req);
        }
    }

    /// Set the last operator action result for the status bar (strategy hotkeys map to disabled no-ops in exploration mode).
    pub fn set_command_status(&mut self, status: CommandStatusView) {
        if status.toast_on_failure && matches!(status.status, CommandStatus::Failed) {
            if let Some(ref err) = status.error {
                self.push_toast(format!("{} — {}", status.action, err), ToastLevel::Error);
            } else if let Some(ref msg) = status.message {
                self.push_toast(format!("{} — {}", status.action, msg), ToastLevel::Warning);
            }
        }
        self.last_command_status = Some(status);
        self.mark_regions(|d| {
            d.mark_hint_bar();
            d.mark_status_bar();
        });
    }

    /// Push a transient toast notification. Toasts expire after TOAST_TTL_SECS seconds.
    pub fn push_toast(&mut self, msg: impl Into<String>, level: ToastLevel) {
        self.toast_manager.push(msg, level);
        self.mark_regions(|d| d.mark_overlay());
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

impl App {
    /// Config key/value for Settings config list.
    pub fn config_key_value_at(&self, index: usize) -> Option<(String, String)> {
        config_key_value_at(&self.config, index)
    }

    pub fn config_key_count(&self) -> usize {
        (0usize..)
            .take_while(|index| self.config_key_value_at(*index).is_some())
            .count()
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
        self.mark_dirty();

        match self.input_mode() {
            InputMode::Help => {
                self.show_help = false;
                return;
            }
            InputMode::DetailPopup => {
                self.detail_popup = None;
                return;
            }
            InputMode::SettingsCredentialEntry => {
                if let Some(ref mut buf) = self.settings_credential_buffer {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(ck) = self.settings_credential_edit_key.take() {
                                let val = buf.trim();
                                let res = if val.is_empty() {
                                    api::credentials::delete_credential(ck)
                                } else {
                                    api::credentials::set_credential(ck, val)
                                };
                                match res {
                                    Ok(()) => {
                                        let msg = if val.is_empty() {
                                            format!("Cleared {}.", ck.display_name())
                                        } else {
                                            format!("Saved {}.", ck.display_name())
                                        };
                                        self.set_command_status(CommandStatusView::success(
                                            "settings", msg,
                                        ));
                                    }
                                    Err(e) => {
                                        self.set_command_status(CommandStatusView::failure(
                                            "settings",
                                            format!("Credential error: {e}"),
                                        ));
                                    }
                                }
                            }
                            self.settings_credential_buffer = None;
                        }
                        KeyCode::Esc => {
                            self.set_command_status(CommandStatusView::success(
                                "settings",
                                "Credential edit cancelled.",
                            ));
                            self.settings_credential_edit_key = None;
                            self.settings_credential_buffer = None;
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

    pub fn handle_action(&mut self, action: crate::input::Action) {
        self.mark_dirty();
        crate::input::apply_action(self, action);
    }
}

#[path = "app_updates.rs"]
mod updates;

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
