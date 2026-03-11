//! REST polling fallback for TUI.
//!
//! When NATS is unavailable, polls the REST API for snapshots.

use std::time::Duration;

use api::RuntimeSnapshotDto;
use reqwest::Client;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info, warn};

use crate::config::TuiConfig;
use crate::events::{
    AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget, LogEntry, LogLevel,
};
use crate::models::{SnapshotSource, TuiSnapshot};

/// Run the REST polling fallback. Sends `TuiSnapshot` updates on `tx`.
/// Polls at the configured interval when NATS is unavailable.
pub async fn run(
    config: TuiConfig,
    tx: watch::Sender<Option<TuiSnapshot>>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) {
    if !config.rest_fallback {
        info!("REST fallback disabled");
        emit_status(
            &event_tx,
            ConnectionState::Disabled,
            "REST fallback disabled",
        );
        emit_log(&event_tx, LogLevel::Info, "REST fallback disabled");
        return;
    }

    let url = format!("{}/api/v1/snapshot", config.rest_url);
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTP client");

    info!(
        url = %url,
        poll_ms = config.rest_poll_ms,
        "REST fallback starting"
    );
    emit_status(
        &event_tx,
        ConnectionState::Starting,
        format!("Polling {url}"),
    );
    emit_log(
        &event_tx,
        LogLevel::Info,
        format!("REST fallback polling {url}"),
    );

    loop {
        if !config.rest_fallback {
            break;
        }

        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<RuntimeSnapshotDto>().await {
                        Ok(dto) => {
                            let snap = TuiSnapshot::new(dto, SnapshotSource::Rest);
                            emit_status(
                                &event_tx,
                                ConnectionState::Connected,
                                format!("Snapshot polling healthy: {url}"),
                            );
                            if tx.send(Some(snap)).is_err() {
                                emit_log(
                                    &event_tx,
                                    LogLevel::Warn,
                                    "REST fallback stopped: UI receiver dropped",
                                );
                                break;
                            }
                            debug!("REST fallback: got snapshot");
                        }
                        Err(e) => {
                            warn!(error = %e, "REST fallback: failed to parse snapshot");
                            emit_status(
                                &event_tx,
                                ConnectionState::Retrying,
                                format!("Parse failed: {e}"),
                            );
                            emit_log(
                                &event_tx,
                                LogLevel::Warn,
                                format!("REST snapshot parse failed: {e}"),
                            );
                        }
                    }
                } else {
                    warn!(status = %resp.status(), "REST fallback: HTTP error");
                    emit_status(
                        &event_tx,
                        ConnectionState::Retrying,
                        format!("HTTP {}", resp.status()),
                    );
                    emit_log(
                        &event_tx,
                        LogLevel::Warn,
                        format!("REST HTTP error: {}", resp.status()),
                    );
                }
            }
            Err(e) => {
                debug!(error = %e, "REST fallback: request failed");
                emit_status(&event_tx, ConnectionState::Retrying, e.to_string());
                emit_log(
                    &event_tx,
                    LogLevel::Warn,
                    format!("REST request failed: {e}"),
                );
            }
        }

        tokio::time::sleep(Duration::from_millis(config.rest_poll_ms)).await;
    }
}

fn emit_status(
    event_tx: &mpsc::UnboundedSender<AppEvent>,
    state: ConnectionState,
    detail: impl Into<String>,
) {
    let _ = event_tx.send(AppEvent::Connection {
        target: ConnectionTarget::Rest,
        status: ConnectionStatus::new(state, detail),
    });
}

fn emit_log(
    event_tx: &mpsc::UnboundedSender<AppEvent>,
    level: LogLevel,
    message: impl Into<String>,
) {
    let _ = event_tx.send(AppEvent::Log(LogEntry::new(
        level,
        Some(ConnectionTarget::Rest),
        message,
    )));
}
