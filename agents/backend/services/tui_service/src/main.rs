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
//! Tracing output goes to two sinks simultaneously:
//!   1. In-TUI Logs tab (tui-logger widget — scrollable, level-filtered)
//!   2. File: /tmp/tui_service.log  (override: LOG_FILE env var)
//!
//! Config file changes are detected every 5s and applied without restart.

use std::time::Duration;

use color_eyre::eyre::Context;
use crossterm::{
    event::{EventStream, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use tokio::sync::{mpsc, watch};
use tracing::{error, info};
use tracing_subscriber::{fmt::writer::BoxMakeWriter, layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod circuit_breaker;
mod config;
mod config_watcher;
mod events;
mod models;
mod nats;
mod ui;
// mod rest; // Removed - NATS-only, no REST fallback

use app::App;
use config::TuiConfig;
use crossterm::tty::IsTty;

fn is_interactive_terminal() -> bool {
    std::io::stdout().is_tty()
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
    tui_logger::set_env_filter_from_env(None); // respects RUST_LOG

    // Also write to a file for persistence (no ANSI so it's grep-friendly).
    // Override with LOG_FILE env var (e.g. LOG_FILE=/dev/stderr for debugging).
    let log_path = std::env::var("LOG_FILE")
        .unwrap_or_else(|_| "/tmp/tui_service.log".to_string());
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

    // Spawn NATS subscriber in the background
    let nats_config = config.clone();
    let nats_tx = snap_tx.clone();
    let nats_event_tx = event_tx.clone();
    tokio::spawn(async move {
        nats::run(nats_config, nats_tx, nats_event_tx).await;
    });

    // Spawn config file watcher — hot-reloads TuiConfig on disk changes
    tokio::spawn(async move {
        config_watcher::run(config_tx).await;
    });

    // Set up terminal
    let mut terminal = init_terminal().context("Failed to initialize TUI terminal")?;
    let mut app = App::new(config, snap_rx, event_rx, config_rx);

    let result = run_loop(&mut terminal, &mut app).await;

    // Always restore terminal on exit (color-eyre panic hook also restores on panic)
    if let Err(err) = restore_terminal() {
        error!(error = %err, "Failed to restore terminal state");
    }

    result.context("TUI event loop error")
}

fn init_terminal() -> color_eyre::Result<ratatui::DefaultTerminal> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))
        .context("create terminal backend")
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    execute!(std::io::stdout(), LeaveAlternateScreen).context("leave alternate screen")?;
    Ok(())
}

/// Async event loop using tokio::select! to multiplex terminal key events and
/// the redraw tick without blocking on either.
///
/// The old synchronous event::poll(tick) approach delayed key responses by up to
/// tick_ms and couldn't react to NATS snapshots mid-tick without extra complexity.
/// EventStream (crossterm "event-stream" feature) yields futures that compose
/// naturally with tokio::select!, giving immediate key response at any tick rate.
///
/// TODO(exarp): T-1773357423959019000 — if the app grows to need per-component
/// event routing, consider adopting the full ratatui/async-template component
/// model.
async fn run_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> color_eyre::Result<()> {
    let mut event_stream = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(app.config.tick_ms));
    // Skip missed ticks rather than bursting to catch up after a slow render
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        terminal.draw(|f| ui::render(f, app))?;

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
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
