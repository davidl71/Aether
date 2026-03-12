//! tui_service — Ratatui terminal UI for the ib-platform.
//!
//! Subscribes to NATS snapshot subject and renders live trading state.
//! Runs without Python; replaces python/tui/.
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
//! Tracing output is written to a log file (not stdout) to avoid clobbering
//! the TUI. Default log path: /tmp/tui_service.log  (override: LOG_FILE env var).
//! Config file changes are detected every 5s and applied without restart.

use std::time::Duration;

use anyhow::Context;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::{mpsc, watch};
use tracing::{error, info};
use tracing_subscriber::fmt::writer::BoxMakeWriter;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Write tracing to a file so error messages don't clobber the TUI terminal.
    // Use LOG_FILE env var to override (e.g. LOG_FILE=/dev/stderr for debugging).
    let log_path = std::env::var("LOG_FILE")
        .unwrap_or_else(|_| "/tmp/tui_service.log".to_string());
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("Failed to open log file: {log_path}"))?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .with_ansi(false)
        .with_writer(BoxMakeWriter::new(log_file))
        .init();

    let config = TuiConfig::load();
    info!(
        backend_id = %config.backend_id,
        nats_url = %config.nats_url,
        snapshot_ttl_secs = %config.snapshot_ttl_secs,
        "tui_service starting"
    );

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

    let result = run_loop(&mut terminal, &mut app);

    // Always restore terminal on exit
    if let Err(err) = restore_terminal() {
        error!(error = %err, "Failed to restore terminal state");
    }

    result.context("TUI event loop error")
}

fn init_terminal() -> anyhow::Result<ratatui::DefaultTerminal> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))
        .context("create terminal backend")
}

fn restore_terminal() -> anyhow::Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    execute!(std::io::stdout(), LeaveAlternateScreen).context("leave alternate screen")?;
    Ok(())
}

fn run_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        let tick = Duration::from_millis(app.config.tick_ms);
        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }

        app.tick();

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
