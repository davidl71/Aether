//! tui_service — Ratatui terminal UI for the ib-platform.
//!
//! Subscribes to NATS snapshot subject and renders live trading state.
//! Falls back to REST polling when NATS is unavailable.
//! Runs without Python; replaces python/tui/.
//!
//! # Usage
//!
//!   NATS_URL=nats://localhost:4222 BACKEND_ID=ib cargo run -p tui_service
//!
//! # Environment variables
//!
//!   NATS_URL         NATS server (default: nats://localhost:4222)
//!   BACKEND_ID       Snapshot subject suffix (default: ib)
//!   REST_URL         REST fallback base URL (default: http://localhost:9090)
//!   WATCHLIST        Comma-separated symbols to highlight (default: SPX,XSP,NDX)
//!   TICK_MS          UI redraw interval ms (default: 250)
//!   REST_POLL_MS     REST polling interval ms (default: 2000)
//!   REST_FALLBACK    Enable REST fallback (default: true)

use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, Event};
use tokio::sync::watch;
use tracing::info;

mod app;
mod config;
mod models;
mod nats;
mod rest;
mod ui;

use app::App;
use config::TuiConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let config = TuiConfig::from_env();
    info!(
        backend_id = %config.backend_id,
        nats_url = %config.nats_url,
        rest_url = %config.rest_url,
        rest_fallback = config.rest_fallback,
        "tui_service starting"
    );

    let (snap_tx, snap_rx) = watch::channel(None);

    // Spawn NATS subscriber in the background
    let nats_config = config.clone();
    let nats_tx = snap_tx.clone();
    tokio::spawn(async move {
        nats::run(nats_config, nats_tx).await;
    });

    // Spawn REST fallback if enabled
    if config.rest_fallback {
        let rest_config = config.clone();
        let rest_tx = snap_tx.clone();
        tokio::spawn(async move {
            rest::run(rest_config, rest_tx).await;
        });
    }

    // Set up terminal
    let mut terminal = ratatui::init();
    let mut app = App::new(config, snap_rx);

    let result = run_loop(&mut terminal, &mut app);

    // Always restore terminal on exit
    ratatui::restore();

    result.context("TUI event loop error")
}

fn run_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> anyhow::Result<()> {
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
