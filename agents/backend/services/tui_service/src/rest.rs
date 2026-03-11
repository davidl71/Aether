//! REST polling fallback for TUI.
//!
//! When NATS is unavailable, polls the REST API for snapshots.

use std::time::Duration;

use api::RuntimeSnapshotDto;
use reqwest::Client;
use tokio::sync::watch;
use tracing::{debug, info, warn};

use crate::config::TuiConfig;
use crate::models::{SnapshotSource, TuiSnapshot};

/// Run the REST polling fallback. Sends `TuiSnapshot` updates on `tx`.
/// Polls at the configured interval when NATS is unavailable.
pub async fn run(config: TuiConfig, tx: watch::Sender<Option<TuiSnapshot>>) {
    if !config.rest_fallback {
        info!("REST fallback disabled");
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
                            if tx.send(Some(snap)).is_err() {
                                break;
                            }
                            debug!("REST fallback: got snapshot");
                        }
                        Err(e) => {
                            warn!(error = %e, "REST fallback: failed to parse snapshot");
                        }
                    }
                } else {
                    warn!(status = %resp.status(), "REST fallback: HTTP error");
                }
            }
            Err(e) => {
                debug!(error = %e, "REST fallback: request failed");
            }
        }

        tokio::time::sleep(Duration::from_millis(config.rest_poll_ms)).await;
    }
}
