//! tui_service — Ratatui terminal UI for the ib-platform.
//!
//! Subscribes to NATS snapshot subject and renders live trading state.
//! Runs without Python; replaces the retired Python/Textual TUI.
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

use std::time::Duration;

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
use serde::Deserialize;
use serde_json::json;
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
mod models;
mod nats;
mod ui;

use app::App;
use config::TuiConfig;
use crossterm::tty::IsTty;
use events::AppEvent;
use events::StrategyCommand;
use market_data::{MarketDataSource, PolygonWsMarketDataSource};

/// Backend reply for api.strategy.start / api.strategy.stop / api.strategy.cancel_all
#[derive(Debug, Deserialize)]
struct StrategyResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

/// Backend reply for api.snapshot.publish_now
#[derive(Debug, Deserialize)]
struct SnapshotPublishResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    generated_at: Option<String>,
    #[serde(default)]
    subject: Option<String>,
}

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
    let (strategy_cmd_tx, strategy_cmd_rx) = mpsc::unbounded_channel();
    let (strategy_result_tx, strategy_result_rx) = mpsc::unbounded_channel();
    let (yield_fetch_tx, yield_fetch_rx) = mpsc::unbounded_channel();
    let (yield_result_tx, yield_result_rx) = mpsc::unbounded_channel();
    let (loans_fetch_tx, loans_fetch_rx) = mpsc::unbounded_channel();
    let (loans_result_tx, loans_result_rx) = mpsc::unbounded_channel();
    let (loan_create_tx, loan_create_rx) = mpsc::unbounded_channel();

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

    // Spawn Polygon WebSocket market data source (direct, bypassing backend_service NATS)
    if let Some(key) = config.polygon_api_key.clone() {
        let tick_event_tx = event_tx.clone();
        tokio::spawn(async move {
            match PolygonWsMarketDataSource::new(["SPX", "XSP", "NDX"], key, None) {
                Ok(source) => {
                    let src = source.clone();
                    tokio::spawn(async move {
                        src.run().await;
                    });
                    let src = source.clone();
                    info!("Polygon WebSocket market data source started");
                    while let Ok(event) = src.next().await {
                        let tick = AppEvent::MarketTick {
                            symbol: event.symbol,
                            bid: event.bid,
                            ask: event.ask,
                            last: event.last,
                        };
                        let _ = tick_event_tx.send(tick);
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to create Polygon WebSocket source");
                }
            }
        });
    }

    // Spawn strategy command handler: receives S/T commands, sends NATS request, forwards result
    let nats_url = config.nats_url.clone();
    tokio::spawn(async move {
        run_strategy_commands(nats_url, strategy_cmd_rx, strategy_result_tx).await;
    });

    // Spawn yield fetcher: receives symbol, requests build_curve + benchmarks, forwards result
    let nats_url_yield = config.nats_url.clone();
    tokio::spawn(async move {
        run_yield_fetcher(nats_url_yield, yield_fetch_rx, yield_result_tx).await;
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
        Some(strategy_cmd_tx),
        Some(yield_fetch_tx),
        Some(loans_fetch_tx),
        Some(loan_create_tx),
    );

    let result = run_loop(
        &mut terminal,
        &mut app,
        strategy_result_rx,
        yield_result_rx,
        loans_result_rx,
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
    let mut stdout = std::io::stdout();
    if use_alternate_screen {
        execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    }
    // Clear screen before first draw (like `reset` does) so no prior output remains.
    execute!(stdout, Clear(ClearType::All)).context("clear screen")?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))
        .context("create terminal backend")
}

fn restore_terminal(use_alternate_screen: bool) -> color_eyre::Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    if use_alternate_screen {
        execute!(std::io::stdout(), LeaveAlternateScreen).context("leave alternate screen")?;
    }
    Ok(())
}

/// Handles strategy start/stop/cancel-all: connects to NATS, sends request, forwards result to TUI.
async fn run_strategy_commands(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<StrategyCommand>,
    result_tx: mpsc::UnboundedSender<Result<String, String>>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            StrategyCommand::Start | StrategyCommand::Stop | StrategyCommand::CancelAll => {
                let (subject, action) = match cmd {
                    StrategyCommand::Start => ("api.strategy.start", "start"),
                    StrategyCommand::Stop => ("api.strategy.stop", "stop"),
                    StrategyCommand::CancelAll => ("api.strategy.cancel_all", "cancel_all"),
                    _ => unreachable!(),
                };
                let res = match NatsClient::connect(&nats_url).await {
                    Ok(nc) => {
                        match request_json_with_retry::<(), StrategyResponse>(&nc, subject, &())
                            .await
                        {
                            Ok(resp) => {
                                if resp.ok {
                                    Ok(resp.message.unwrap_or_else(|| "ok".into()))
                                } else {
                                    Err(resp.error.unwrap_or_else(|| "unknown".into()))
                                }
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    Err(e) => Err(e.to_string()),
                };
                if let Err(ref e) = res {
                    error!(action = %action, error = %e, "Strategy command failed");
                }
                let _ = result_tx.send(res);
            }
            StrategyCommand::PublishSnapshot => {
                const SUBJECT_SNAPSHOT_PUBLISH_NOW: &str = "api.snapshot.publish_now";
                let res = match NatsClient::connect(&nats_url).await {
                    Ok(nc) => {
                        match request_json_with_retry::<(), SnapshotPublishResponse>(
                            &nc,
                            SUBJECT_SNAPSHOT_PUBLISH_NOW,
                            &(),
                        )
                        .await
                        {
                            Ok(resp) => {
                                if resp.ok {
                                    let msg = resp
                                        .generated_at
                                        .map(|t| format!("Snapshot published at {}", t))
                                        .or(resp
                                            .subject
                                            .map(|s| format!("Snapshot published to {}", s)))
                                        .unwrap_or_else(|| "Snapshot published".into());
                                    Ok(msg)
                                } else {
                                    Err(resp.error.unwrap_or_else(|| "unknown".into()))
                                }
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    Err(e) => Err(e.to_string()),
                };
                if let Err(ref e) = res {
                    error!(error = %e, "Force snapshot failed");
                }
                let _ = result_tx.send(res);
            }
            StrategyCommand::SetMode(mode) => {
                let res = match NatsClient::connect(&nats_url).await {
                    Ok(nc) => {
                        let body = json!({ "mode": mode });
                        match request_json_with_retry::<_, StrategyResponse>(
                            &nc,
                            "api.admin.set_mode",
                            &body,
                        )
                        .await
                        {
                            Ok(resp) => {
                                if resp.ok {
                                    Ok(resp.message.unwrap_or_else(|| format!("mode {}", mode)))
                                } else {
                                    Err(resp.error.unwrap_or_else(|| "unknown".into()))
                                }
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    Err(e) => Err(e.to_string()),
                };
                if let Err(ref e) = res {
                    error!(mode = %mode, error = %e, "Set mode failed");
                }
                let _ = result_tx.send(res);
            }
            StrategyCommand::ExecuteScenario(scenario) => {
                let res = match NatsClient::connect(&nats_url).await {
                    Ok(nc) => {
                        match request_json_with_retry::<_, StrategyResponse>(
                            &nc,
                            "api.strategy.execute",
                            &scenario,
                        )
                        .await
                        {
                            Ok(resp) => {
                                if resp.ok {
                                    Ok(resp.message.unwrap_or_else(|| "scenario executed".into()))
                                } else {
                                    Err(resp.error.unwrap_or_else(|| "unknown".into()))
                                }
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    Err(e) => Err(e.to_string()),
                };
                if let Err(ref e) = res {
                    error!(symbol = %scenario.symbol, error = %e, "Execute scenario failed");
                }
                let _ = result_tx.send(res);
            }
        }
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
            .and_then(|r| r.map_err(|e| e)),
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

        if let Err(e) = res {
            let _ = result_tx.send(Err(e));
            continue;
        }

        match res.unwrap() {
            Err(e) => {
                let _ = result_tx.send(Err(e));
            }
            Ok(()) => {
                let refresh_res =
                    match NatsClient::connect(&nats_url).await {
                        Ok(nc) => request_json_with_retry::<(), Result<Vec<LoanRecord>, String>>(
                            &nc,
                            "api.loans.list",
                            &(),
                        )
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r.map_err(|e| e)),
                        Err(e) => Err(e.to_string()),
                    };
                let _ = result_tx.send(refresh_res);
            }
        }
    }
}

/// Fetches curve + benchmarks for a symbol via NATS, sends result to TUI.
async fn run_yield_fetcher(
    nats_url: String,
    mut rx: mpsc::UnboundedReceiver<String>,
    result_tx: mpsc::UnboundedSender<Result<(CurveResponse, BenchmarksResponse), String>>,
) {
    while let Some(symbol) = rx.recv().await {
        let res = match NatsClient::connect(&nats_url).await {
            Ok(nc) => {
                let curve_res = request_json_with_retry(
                    &nc,
                    "api.finance_rates.build_curve",
                    &json!({ "opportunities": [], "symbol": symbol }),
                )
                .await
                .map_err(|e| e.to_string());
                let curve = match curve_res {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = result_tx.send(Err(e));
                        continue;
                    }
                };
                // Benchmarks call FRED (SOFR + Treasury); allow longer than default 5s, with retry
                const BENCHMARKS_TIMEOUT: Duration = Duration::from_secs(15);
                let bench_res = request_json_with_retry_timeout::<(), BenchmarksResponse>(
                    &nc,
                    "api.finance_rates.benchmarks",
                    &(),
                    BENCHMARKS_TIMEOUT,
                    RetryConfig::default(),
                )
                .await
                .map_err(|e| e.to_string());
                match bench_res {
                    Ok(benchmarks) => Ok((curve, benchmarks)),
                    Err(e) => {
                        let _ = result_tx.send(Err(e));
                        continue;
                    }
                }
            }
            Err(e) => Err(e.to_string()),
        };
        let _ = result_tx.send(res);
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
    mut strategy_result_rx: mpsc::UnboundedReceiver<Result<String, String>>,
    mut yield_result_rx: mpsc::UnboundedReceiver<
        Result<(CurveResponse, BenchmarksResponse), String>,
    >,
    mut loans_result_rx: mpsc::UnboundedReceiver<Result<Vec<LoanRecord>, String>>,
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

            // Strategy command result from NATS request task
            Some(res) = strategy_result_rx.recv() => {
                app.set_strategy_result(res);
            }

            // Yield fetch result (curve + benchmarks)
            Some(res) = yield_result_rx.recv() => {
                app.set_yield_data(res);
            }

            // Loans fetch result
            Some(res) = loans_result_rx.recv() => {
                app.set_loans_data(res);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
