//! Config file hot-reload watcher.
//!
//! Polls the shared config file's modification time every CONFIG_POLL_SECS.
//! When the mtime changes, reloads TuiConfig and sends it on the config channel.
//! The app picks it up on the next tick and applies the new values.

use std::time::{Duration, SystemTime};

use api::project_paths::shared_config_candidate_paths;
use tokio::sync::watch;
use tracing::{info, warn};

use crate::config::TuiConfig;

const CONFIG_POLL_SECS: u64 = 5;

/// Run the config watcher loop. Sends updated `TuiConfig` on `tx` when the
/// config file changes on disk. Runs indefinitely until the process exits.
pub async fn run(tx: watch::Sender<TuiConfig>) {
    let mut last_mtime: Option<SystemTime> = probe_mtime();

    loop {
        tokio::time::sleep(Duration::from_secs(CONFIG_POLL_SECS)).await;

        let mtime = probe_mtime();

        let changed = match (mtime, last_mtime) {
            (Some(now), Some(prev)) => now != prev,
            _ => false,
        };

        last_mtime = mtime;

        if changed {
            let new_config = TuiConfig::load();
            info!("Config file changed on disk, hot-reloading TUI config");
            if tx.send(new_config).is_err() {
                warn!("Config watcher: app receiver dropped, stopping watcher");
                break;
            }
        }
    }
}

fn probe_mtime() -> Option<SystemTime> {
    for path in shared_config_candidate_paths() {
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                return Some(mtime);
            }
        }
    }
    None
}
