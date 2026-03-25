//! tui_service — Ratatui terminal UI for the ib-platform.
//!
//! Subscribes to NATS snapshot subject and renders read-only portfolio and
//! market state for exploration. Replaces the retired Python/Textual TUI.
//!
//! # Usage
//!
//!   NATS_URL=nats://localhost:4222 BACKEND_ID=ib cargo run -p tui_service
//!
//! # Configuration
//!
//! Reads the shared config file first (`IB_BOX_SPREAD_CONFIG`, home config,
//! then project defaults) and applies these env vars as final overrides:
//!
//!   NATS_URL              NATS server (default: nats://localhost:4222)
//!   BACKEND_ID            Snapshot subject suffix (default: ib)
//!   WATCHLIST             Comma-separated symbols to highlight (default: SPX,XSP,NDX)
//!   TICK_MS               UI redraw interval ms (default: 250)
//!   SNAPSHOT_TTL_SECS     Seconds before data is shown as stale (default: 30)
//!
//! # Mosh (mobile shell)
//!
//! When running under mosh (detected via TERM containing "mosh" or MOSH_TTY set),
//! the TUI skips the alternate screen buffer so redraws and scrollback behave
//! better. You can force this with `TUI_NO_ALT_SCREEN=1` in any terminal.
//!
//! Tracing output goes to two sinks simultaneously:
//!   1. In-TUI Logs tab (tui-logger widget — scrollable, level-filtered)
//!   2. File: /tmp/tui_service.log  (override: LOG_FILE env var)
//!
//! Config file changes are detected every 5s and applied without restart.

use std::{
    panic,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    time::Duration,
};

use api::discount_bank::{DiscountBankBalanceDto, DiscountBankTransactionsListDto};
use api::finance_rates::{BenchmarksResponse, CurveResponse};
use api::loans::LoanRecord;
use color_eyre::eyre::Context;
use crossterm::{
    event::{EventStream, KeyEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use futures::StreamExt;
use nats_adapter::{
    request_json_with_retry, request_json_with_retry_timeout, NatsClient, RetryConfig,
};
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::{mpsc, watch};
use tracing::{error, info};
use tracing_subscriber::{
    fmt::writer::BoxMakeWriter, layer::SubscriberExt, util::SubscriberInitExt,
};

mod app;
mod circuit_breaker;
mod config;
mod config_watcher;
mod events;
mod expiry_buckets;
mod input;
mod input_settings;
mod models;
mod nats;
mod option_symbol;
mod ui;
mod workspace;

use app::{App, FmpDetail, GreeksDisplay, GreeksFetchRequest};
use config::TuiConfig;
use crossterm::tty::IsTty;
use events::AppEvent;

static RAW_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALT_SCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);

fn is_interactive_terminal() -> bool {
    std::io::stdout().is_tty()
}

/// True if we should skip the alternate screen (mosh or TUI_NO_ALT_SCREEN).
/// Improves behavior in mosh and preserves scrollback when requested.
fn skip_alternate_screen() -> bool {
    if std::env::var("TUI_NO_ALT_SCREEN")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    if std::env::var("MOSH_TTY").is_ok() {
        return true;
    }
    if let Ok(term) = std::env::var("TERM") {
        if term.to_lowercase().contains("mosh") {
            return true;
        }
    }
    false
}

fn run_noninteractive(config: TuiConfig) -> color_eyre::Result<()> {
    println!("tui_service: Non-interactive terminal detected.");
    println!("This service requires a TTY for the Ratatui interface.");
    println!();
    println!("To run in background or scripts, use the API instead:");
    println!("  curl http://localhost:8080/api/v1/snapshot");
    println!();
    println!("Backend ID: {}", config.backend_id);
    Ok(())
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Install color-eyre: pretty-prints errors and — critically — restores the terminal
    // before printing panic backtraces so the shell isn't left in raw mode.
    color_eyre::install()?;
    install_panic_hook();

    // tui-logger: captures all tracing events into an in-memory ring buffer
    // that the Logs tab widget reads. Visible inside the TUI immediately.
    tui_logger::init_logger(log::LevelFilter::Trace).expect("tui-logger init");
    tui_logger::set_default_level(log::LevelFilter::Debug);
    tui_logger::set_buffer_depth(10_000);
    tui_logger::set_env_filter_from_env(None); // respects RUST_LOG

    // Also write to a file for persistence (no ANSI so it's grep-friendly).
    // Override with LOG_FILE env var (e.g. LOG_FILE=/dev/stderr for debugging).
    let log_path = std::env::var("LOG_FILE").unwrap_or_else(|_| "/tmp/tui_service.log".to_string());
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("Failed to open log file: {log_path}"))?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tui_logger::TuiTracingSubscriberLayer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .with_writer(BoxMakeWriter::new(log_file)),
        )
        .init();

    let config = TuiConfig::load();
    let require_nats = !is_interactive_terminal();
    if let Err(errors) = config.validate(require_nats) {
        let msg = errors.join("; ");
        error!("Config validation failed: {}", msg);
        return Err(color_eyre::eyre::eyre!("Config validation failed: {}", msg));
    }
    info!(
        backend_id = %config.backend_id,
        nats_url = %config.nats_url,
        snapshot_ttl_secs = %config.snapshot_ttl_secs,
        "tui_service starting"
    );

    if !is_interactive_terminal() {
        return run_noninteractive(config);
    }

    let (snap_tx, snap_rx) = watch::channel(None);
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (config_tx, config_rx) = watch::channel(config.clone());
    let (health_tx, health_rx) = watch::channel(std::collections::HashMap::new());
    // Yield refresh: TUI → background task → publishes to api.yield_curve.refresh.
    // Results come back via NATS KV watch (AppEvent::YieldCurveKvUpdate through event_tx).
    let (yield_refresh_tx, yield_refresh_rx) = mpsc::unbounded_channel::<()>();
    // Benchmarks (SOFR/Treasury) are fetched periodically on a separate slow interval.
    // Result comes back via event_tx as AppEvent::BenchmarksUpdate.
    // Keep a dummy channel so run_loop signature stays simple.
    let (_yield_result_tx_unused, yield_result_rx) = mpsc::unbounded_channel::<
        Result<(HashMap<String, CurveResponse>, BenchmarksResponse), String>,
    >();
    let (loans_fetch_tx, loans_fetch_rx) = mpsc::unbounded_channel();
    let (loans_result_tx, loans_result_rx) = mpsc::unbounded_channel();
    let (loan_create_tx, loan_create_rx) = mpsc::unbounded_channel();
    let (fmp_fetch_tx, fmp_fetch_rx) = mpsc::unbounded_channel::<String>();
    let (fmp_result_tx, fmp_result_rx) = mpsc::unbounded_channel::<Result<FmpDetail, String>>();
    let (greeks_fetch_tx, greeks_fetch_rx) = mpsc::unbounded_channel::<GreeksFetchRequest>();
    let (greeks_result_tx, greeks_result_rx) =
        mpsc::unbounded_channel::<Result<GreeksDisplay, String>>();
    let (discount_bank_fetch_tx, discount_bank_fetch_rx) = mpsc::unbounded_channel::<()>();
    let (discount_bank_result_tx, discount_bank_result_rx) = mpsc::unbounded_channel::<(
        Result<DiscountBankBalanceDto, String>,
        Result<DiscountBankTransactionsListDto, String>,
    )>();
    let health_publish_interval_secs: u64 = std::env::var("HEALTH_PUBLISH_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);

    let health_publish_url = config.nats_url.clone();
    let health_publish_backend_id = config.backend_id.clone();
    tokio::spawn(async move {
        loop {
            match nats_adapter::NatsClient::connect(&health_publish_url).await {
                Ok(client) => {
                    let mut extra = HashMap::new();
                    extra.insert("pid".to_string(), std::process::id().to_string());
                    extra.insert("service".to_string(), "tui_service".to_string());
                    extra.insert("backend_id".to_string(), health_publish_backend_id.clone());
                    nats_adapter::spawn_health_publisher(
                        Arc::new(client),
                        "tui_service".to_string(),
                        health_publish_interval_secs,
                        extra,
                    );
                    break;
                }
                Err(err) => {
                    tracing::warn!(error = %err, "Failed to connect TUI health publisher to NATS");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    // Spawn NATS subscriber in the background (snapshot + system.health)
    let nats_config = config.clone();
    let nats_tx = snap_tx.clone();
    let nats_event_tx = event_tx.clone();
    let nats_health_tx = health_tx.clone();
    tokio::spawn(async move {
        nats::run(nats_config, nats_tx, nats_event_tx, nats_health_tx).await;
    });

    // Spawn NATS tick subscriber for real-time market data (market-data.>)
    let nats_url_tick = config.nats_url.clone();
    let tick_event_tx = event_tx.clone();
    tokio::spawn(async move {
        match nats_adapter::NatsClient::connect(&nats_url_tick).await {
            Ok(client) => {
                if let Err(e) = nats::run_tick_subscriber(client, tick_event_tx).await {
                    tracing::warn!(error = %e, "Tick subscriber ended");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to connect tick subscriber to NATS");
            }
        }
    });

    // Spawn NATS candle subscriber for real chart OHLCV updates.
    let nats_url_candle = config.nats_url.clone();
    let candle_event_tx = event_tx.clone();
    tokio::spawn(async move {
        match nats_adapter::NatsClient::connect(&nats_url_candle).await {
            Ok(client) => {
                if let Err(e) = nats::run_candle_subscriber(client, candle_event_tx).await {
                    tracing::warn!(error = %e, "Candle subscriber ended");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to connect candle subscriber to NATS");
            }
        }
    });

    let nats_url_alerts = config.nats_url.clone();
    let alert_event_tx = event_tx.clone();
    tokio::spawn(async move {
        match nats_adapter::NatsClient::connect(&nats_url_alerts).await {
            Ok(client) => {
                if let Err(e) = nats::run_alert_subscriber(client, alert_event_tx).await {
                    tracing::warn!(error = %e, "Alert subscriber ended");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to connect alert subscriber to NATS");
            }
        }
    });

    let nats_url_commands = config.nats_url.clone();
    let command_event_tx = event_tx.clone();
    tokio::spawn(async move {
        match nats_adapter::NatsClient::connect(&nats_url_commands).await {
            Ok(client) => {
                if let Err(e) = nats::run_command_subscriber(client, command_event_tx).await {
                    tracing::warn!(error = %e, "Command-event subscriber ended");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to connect command-event subscriber to NATS");
            }
        }
    });

    // Spawn yield KV watcher: watches NATS KV for yield curve updates, emits AppEvent.
    let nats_url_kv = config.nats_url.clone();
    let kv_bucket = config.yield_kv_bucket.clone();
    let kv_event_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            match nats_adapter::NatsClient::connect(&nats_url_kv).await {
                Ok(client) => {
                    if let Err(e) =
                        nats::run_yield_kv_watcher(client, kv_bucket.clone(), kv_event_tx.clone())
                            .await
                    {
                        tracing::warn!(error = %e, "Yield KV watcher ended, reconnecting");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to connect yield KV watcher to NATS");
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Spawn yield refresh publisher: sends api.yield_curve.refresh when triggered.
    let nats_url_refresh = config.nats_url.clone();
    let refresh_event_tx = event_tx.clone();
    tokio::spawn(async move {
        run_yield_refresher(nats_url_refresh, yield_refresh_rx, refresh_event_tx).await;
    });

    // Spawn benchmarks periodic fetcher (SOFR/Treasury from FRED — slow interval).
    let nats_url_bench = config.nats_url.clone();
    let bench_interval = Duration::from_secs(config.benchmarks_refresh_secs);
    let bench_event_tx = event_tx.clone();
    tokio::spawn(async move {
        run_benchmarks_fetcher(nats_url_bench, bench_interval, bench_event_tx).await;
    });

    // Spawn loans fetcher: requests api.loans.list, forwards result
    let nats_url_loans = config.nats_url.clone();
    let loans_result_tx_for_fetcher = loans_result_tx.clone();
    tokio::spawn(async move {
        run_loans_fetcher(nats_url_loans, loans_fetch_rx, loans_result_tx_for_fetcher).await;
    });

    // Spawn loan creator: receives LoanRecord, sends to api.loans.create, refreshes list
    let nats_url_loan_create = config.nats_url.clone();
    tokio::spawn(async move {
        run_loan_creator(nats_url_loan_create, loan_create_rx, loans_result_tx).await;
    });

    // Spawn FMP fetcher: receives symbol, requests quote + income statement, forwards result
    let nats_url_fmp = config.nats_url.clone();
    tokio::spawn(async move {
        run_fmp_fetcher(nats_url_fmp, fmp_fetch_rx, fmp_result_tx).await;
    });

    // Spawn greeks fetcher: receives GreeksFetchRequest, computes IV then greeks, forwards result
    let nats_url_greeks = config.nats_url.clone();
    tokio::spawn(async move {
        run_greeks_fetcher(nats_url_greeks, greeks_fetch_rx, greeks_result_tx).await;
    });

    // Spawn Discount Bank fetcher: requests balance + transactions concurrently, forwards result
    let nats_url_discount_bank = config.nats_url.clone();
    tokio::spawn(async move {
        run_discount_bank_fetcher(
            nats_url_discount_bank,
            discount_bank_fetch_rx,
            discount_bank_result_tx,
        )
        .await;
    });

    // Spawn config file watcher — hot-reloads TuiConfig on disk changes
    tokio::spawn(async move {
        config_watcher::run(config_tx).await;
    });

    // Set up terminal (skip alternate screen in mosh or when TUI_NO_ALT_SCREEN=1)
    let use_alternate_screen = !skip_alternate_screen();
    if use_alternate_screen {
        info!("TUI using alternate screen buffer");
    } else {
        info!("TUI using main screen (mosh-friendly or TUI_NO_ALT_SCREEN=1)");
    }
    let mut terminal =
        init_terminal(use_alternate_screen).context("Failed to initialize TUI terminal")?;
    let mut app = App::new(
        config,
        snap_rx,
        event_rx,
        config_rx,
        health_rx,
        Some(yield_refresh_tx),
        Some(loans_fetch_tx),
        Some(loan_create_tx),
        Some(fmp_fetch_tx),
        Some(greeks_fetch_tx),
        Some(discount_bank_fetch_tx),
    );

    let result = run_loop(
        &mut terminal,
        &mut app,
        yield_result_rx,
        loans_result_rx,
        fmp_result_rx,
        greeks_result_rx,
        discount_bank_result_rx,
    )
    .await;

    // Always restore terminal on exit (color-eyre panic hook also restores on panic)
    if let Err(err) = restore_terminal(use_alternate_screen) {
        error!(error = %err, "Failed to restore terminal state");
    }

    result.context("TUI event loop error")
}

fn init_terminal(use_alternate_screen: bool) -> color_eyre::Result<ratatui::DefaultTerminal> {
    enable_raw_mode().context("enable raw mode")?;
    RAW_MODE_ACTIVE.store(true, Ordering::SeqCst);
    let init_result = (|| {
        let mut stdout = std::io::stdout();
        if use_alternate_screen {
            execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
            ALT_SCREEN_ACTIVE.store(true, Ordering::SeqCst);
        }
        // Clear screen before first draw (like `reset` does) so no prior output remains.
        execute!(stdout, Clear(ClearType::All)).context("clear screen")?;
        ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))
            .context("create terminal backend")
    })();
    if init_result.is_err() {
        let _ = restore_terminal(use_alternate_screen);
    }
    init_result
}

fn restore_terminal(use_alternate_screen: bool) -> color_eyre::Result<()> {
    if RAW_MODE_ACTIVE.swap(false, Ordering::SeqCst) {
        disable_raw_mode().context("disable raw mode")?;
    }
    if use_alternate_screen && ALT_SCREEN_ACTIVE.swap(false, Ordering::SeqCst) {
        execute!(std::io::stdout(), LeaveAlternateScreen).context("leave alternate screen")?;
    }
    Ok(())
}

fn install_panic_hook() {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let use_alternate_screen = ALT_SCREEN_ACTIVE.load(Ordering::SeqCst);
        let _ = restore_terminal(use_alternate_screen);
        previous_hook(panic_info);
    }));
}

/// Fetches Discount Bank balance and transactions concurrently via NATS, sends result to TUI.
async fn run_discount_bank_fetcher(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<()>,
    result_tx: mpsc::UnboundedSender<(
        Result<DiscountBankBalanceDto, String>,
        Result<DiscountBankTransactionsListDto, String>,
    )>,
) {
    while rx.recv().await.is_some() {
        let balance_res = match NatsClient::connect(&nats_url).await {
            Ok(nc) => request_json_with_retry::<(), Result<DiscountBankBalanceDto, String>>(
                &nc,
                "api.discount_bank.balance",
                &(),
            )
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r),
            Err(e) => Err(e.to_string()),
        };
        let txns_res = match NatsClient::connect(&nats_url).await {
            Ok(nc) => request_json_with_retry::<
                serde_json::Value,
                Result<DiscountBankTransactionsListDto, String>,
            >(
                &nc,
                "api.discount_bank.transactions",
                &json!({ "limit": 50 }),
            )
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r),
            Err(e) => Err(e.to_string()),
        };
        let _ = result_tx.send((balance_res, txns_res));
    }
}

/// Fetches loans list via NATS api.loans.list, sends result to TUI.
async fn run_loans_fetcher(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<()>,
    result_tx: mpsc::UnboundedSender<Result<Vec<LoanRecord>, String>>,
) {
    while rx.recv().await.is_some() {
        let res = match NatsClient::connect(&nats_url).await {
            Ok(nc) => request_json_with_retry::<(), Result<Vec<LoanRecord>, String>>(
                &nc,
                "api.loans.list",
                &(),
            )
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r),
            Err(e) => Err(e.to_string()),
        };
        let _ = result_tx.send(res);
    }
}

/// Creates a new loan via NATS api.loans.create, then refreshes the loans list.
async fn run_loan_creator(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<LoanRecord>,
    result_tx: mpsc::UnboundedSender<Result<Vec<LoanRecord>, String>>,
) {
    while let Some(loan) = rx.recv().await {
        let res: Result<Result<(), String>, String> = match NatsClient::connect(&nats_url).await {
            Ok(nc) => request_json_with_retry(&nc, "api.loans.create", &loan)
                .await
                .map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };

        let inner = match res {
            Err(e) => {
                let _ = result_tx.send(Err(e));
                continue;
            }
            Ok(v) => v,
        };

        match inner {
            Err(e) => {
                let _ = result_tx.send(Err(e));
            }
            Ok(()) => {
                let refresh_res = match NatsClient::connect(&nats_url).await {
                    Ok(nc) => request_json_with_retry::<(), Result<Vec<LoanRecord>, String>>(
                        &nc,
                        "api.loans.list",
                        &(),
                    )
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r),
                    Err(e) => Err(e.to_string()),
                };
                let _ = result_tx.send(refresh_res);
            }
        }
    }
}

/// Publishes a yield refresh request to `api.yield_curve.refresh` whenever triggered.
/// The KV watcher picks up the writer's response and updates curves via AppEvent.
async fn run_yield_refresher(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<()>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) {
    while rx.recv().await.is_some() {
        match NatsClient::connect(&nats_url).await {
            Ok(client) => {
                let ok = nats::send_yield_refresh(&client).await.is_ok();
                let _ = event_tx.send(AppEvent::YieldRefreshAck { ok });
            }
            Err(e) => {
                tracing::warn!(error = %e, "yield refresher: NATS connect failed");
                let _ = event_tx.send(AppEvent::YieldRefreshAck { ok: false });
            }
        }
    }
}

/// Fetches SOFR + Treasury benchmarks periodically and emits AppEvent::BenchmarksUpdate.
async fn run_benchmarks_fetcher(
    nats_url: String,
    interval: Duration,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) {
    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        ticker.tick().await;
        match NatsClient::connect(&nats_url).await {
            Ok(nc) => {
                const BENCHMARKS_TIMEOUT: Duration = Duration::from_secs(15);
                match request_json_with_retry_timeout::<(), BenchmarksResponse>(
                    &nc,
                    "api.finance_rates.benchmarks",
                    &(),
                    BENCHMARKS_TIMEOUT,
                    RetryConfig::default(),
                )
                .await
                {
                    Ok(benchmarks) => {
                        let _ = event_tx.send(AppEvent::BenchmarksUpdate(benchmarks));
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "benchmarks fetch failed");
                    }
                }
            }
            Err(e) => {
                tracing::debug!(error = %e, "benchmarks fetcher: NATS connect failed");
            }
        }
    }
}

/// Async event loop using tokio::select! to multiplex terminal key events and
/// the redraw tick without blocking on either.
///
/// The old synchronous event::poll(tick) approach delayed key responses by up to
/// tick_ms and couldn't react to NATS snapshots mid-tick without extra complexity.
/// EventStream (crossterm "event-stream" feature) yields futures that compose
/// naturally with tokio::select!, giving immediate key response at any tick rate.
///
/// TODO(T-1773509396768932000): if the app grows to need per-component
/// event routing, consider adopting the full ratatui/async-template component
/// model.
async fn run_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    mut yield_result_rx: mpsc::UnboundedReceiver<
        Result<(HashMap<String, CurveResponse>, BenchmarksResponse), String>,
    >,
    mut loans_result_rx: mpsc::UnboundedReceiver<Result<Vec<LoanRecord>, String>>,
    mut fmp_result_rx: mpsc::UnboundedReceiver<Result<FmpDetail, String>>,
    mut greeks_result_rx: mpsc::UnboundedReceiver<Result<GreeksDisplay, String>>,
    mut discount_bank_result_rx: mpsc::UnboundedReceiver<(
        Result<DiscountBankBalanceDto, String>,
        Result<DiscountBankTransactionsListDto, String>,
    )>,
) -> color_eyre::Result<()> {
    let mut event_stream = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(app.config.tick_ms));
    // Skip missed ticks rather than bursting to catch up after a slow render
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        // Only redraw when state has changed (dirty flag optimization)
        if app.needs_redraw {
            terminal.draw(|f| ui::render(f, app))?;
            app.needs_redraw = false;
        }

        tokio::select! {
            // Redraw timer — drives app.tick() for snapshot/config polling
            _ = tick_interval.tick() => {
                app.tick();
            }

            // Key events from the terminal — react immediately, no tick delay
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(crossterm::event::Event::Key(key))) => {
                        // Only process Press; ignore Repeat/Release (crossterm 0.27+)
                        if key.kind == KeyEventKind::Press {
                            app.handle_key(key);
                        }
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "Terminal event stream error");
                    }
                    None => break, // stream closed — terminal gone
                    _ => {}        // resize, focus, mouse — not used yet
                }
            }

            // Yield fetch result (curve + benchmarks)
            Some(res) = yield_result_rx.recv() => {
                app.set_yield_data(res);
            }

            // Loans fetch result
            Some(res) = loans_result_rx.recv() => {
                app.set_loans_data(res);
            }

            // FMP quote + fundamentals result
            Some(res) = fmp_result_rx.recv() => {
                app.set_fmp_data(res);
            }

            // Greeks computation result
            Some(res) = greeks_result_rx.recv() => {
                app.set_greeks_data(res);
            }

            // Discount Bank fetch result (balance + transactions)
            Some((balance, txns)) = discount_bank_result_rx.recv() => {
                app.set_discount_bank_data(balance, txns);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Fetches FMP quote + latest income statement for a symbol via NATS, sends result to TUI.
async fn run_fmp_fetcher(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<String>,
    result_tx: mpsc::UnboundedSender<Result<FmpDetail, String>>,
) {
    while let Some(symbol) = rx.recv().await {
        let res = match NatsClient::connect(&nats_url).await {
            Ok(nc) => {
                // Fetch quote
                let quote_res = request_json_with_retry::<_, serde_json::Value>(
                    &nc,
                    "api.fmp.quote",
                    &serde_json::json!({ "symbol": symbol, "limit": 1 }),
                )
                .await;
                // Fetch income statement
                let income_res = request_json_with_retry::<_, serde_json::Value>(
                    &nc,
                    "api.fmp.income_statement",
                    &serde_json::json!({ "symbol": symbol, "limit": 1 }),
                )
                .await;
                match (quote_res, income_res) {
                    (Ok(q), income) => {
                        let income_arr = income
                            .ok()
                            .and_then(|v| v.as_array().cloned())
                            .unwrap_or_default();
                        let first = income_arr.first();
                        Ok(FmpDetail {
                            symbol: symbol.clone(),
                            price: q.get("price").and_then(|v| v.as_f64()),
                            day_high: q.get("dayHigh").and_then(|v| v.as_f64()),
                            day_low: q.get("dayLow").and_then(|v| v.as_f64()),
                            prev_close: q.get("previousClose").and_then(|v| v.as_f64()),
                            eps: first.and_then(|r| r.get("eps")).and_then(|v| v.as_f64()),
                            revenue: first
                                .and_then(|r| r.get("revenue"))
                                .and_then(|v| v.as_f64()),
                            net_income: first
                                .and_then(|r| r.get("netIncome"))
                                .and_then(|v| v.as_f64()),
                        })
                    }
                    (Err(e), _) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        };
        let _ = result_tx.send(res);
    }
}

/// Computes IV then greeks for an option position via NATS, sends result to TUI.
async fn run_greeks_fetcher(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<GreeksFetchRequest>,
    result_tx: mpsc::UnboundedSender<Result<GreeksDisplay, String>>,
) {
    while let Some(req) = rx.recv().await {
        let res = match NatsClient::connect(&nats_url).await {
            Ok(nc) => {
                // Step 1: compute IV
                let iv_res = request_json_with_retry::<_, serde_json::Value>(
                    &nc,
                    "api.calculate.iv",
                    &serde_json::json!({
                        "market_price": req.market_price,
                        "underlying_price": req.underlying,
                        "strike_price": req.strike,
                        "time_to_expiry": req.tte_years,
                        "risk_free_rate": req.rate,
                        "option_type": req.option_type,
                    }),
                )
                .await;
                let iv = match iv_res {
                    Ok(v) => v
                        .get("implied_volatility")
                        .and_then(|x| x.as_f64())
                        .unwrap_or(0.2),
                    Err(_) => 0.2, // fallback 20% vol
                };
                // Step 2: compute greeks with IV
                let greeks_res = request_json_with_retry::<_, serde_json::Value>(
                    &nc,
                    "api.calculate.greeks",
                    &serde_json::json!({
                        "underlying_price": req.underlying,
                        "strike_price": req.strike,
                        "time_to_expiry": req.tte_years,
                        "risk_free_rate": req.rate,
                        "volatility": iv,
                        "option_type": req.option_type,
                    }),
                )
                .await
                .map_err(|e| e.to_string());
                match greeks_res {
                    Err(e) => Err(e),
                    Ok(greeks_val) => match greeks_val.get("greeks").cloned() {
                        None => Err("no greeks field".to_string()),
                        Some(greeks) => Ok(GreeksDisplay {
                            iv,
                            delta: greeks.get("delta").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            gamma: greeks.get("gamma").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            theta: greeks.get("theta").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            vega: greeks.get("vega").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            rho: greeks.get("rho").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        }),
                    },
                }
            }
            Err(e) => Err(e.to_string()),
        };
        let _ = result_tx.send(res);
    }
}
