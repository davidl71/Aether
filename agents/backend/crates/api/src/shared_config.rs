//! Shared JSON config loader — same discovery as TUI and backend.
//!
//! Use this for any binary that should read the shared config file
//! (e.g. CLI). Discovery order: `IB_BOX_SPREAD_CONFIG` env, then
//! home/config paths, then workspace `config/config.json`.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::project_paths::shared_config_candidate_paths;

/// Result of loading the shared config: path and parsed JSON (with comments stripped).
#[derive(Debug, Clone)]
pub struct LoadedSharedConfig {
    pub path: PathBuf,
    pub value: Value,
}

/// Load the first existing shared config file. Uses the same candidate list as TUI.
pub fn load_shared_config() -> Result<Option<LoadedSharedConfig>, String> {
    let candidates = shared_config_candidate_paths();
    let mut last_error = None;

    for candidate in candidates {
        if !candidate.exists() || !candidate.is_file() {
            continue;
        }

        match read_shared_config_at(&candidate) {
            Ok(value) => {
                return Ok(Some(LoadedSharedConfig {
                    path: candidate,
                    value,
                }));
            }
            Err(e) => {
                last_error = Some(format!("{}: {}", candidate.display(), e));
            }
        }
    }

    if let Some(err) = last_error {
        return Err(err);
    }

    Ok(None)
}

/// Read and parse shared config from a specific path (JSON/JSONC).
pub fn read_shared_config_at(path: &Path) -> Result<Value, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value = jsonc_parser::parse_to_serde_value(&raw, &Default::default())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Empty config".to_string())?;
    Ok(value)
}

/// Validation errors for shared config (same rules as TUI: BACKEND_ID, NATS_URL, TICK_MS, SNAPSHOT_TTL_SECS).
/// Applies env overrides (NATS_URL, BACKEND_ID) before validating so result matches TUI behavior.
pub fn validate_shared_config(loaded: &LoadedSharedConfig) -> Result<(), Vec<String>> {
    let mut nats_url = loaded
        .value
        .get("tws")
        .and_then(|t| t.get("natsUrl"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if let Ok(v) = std::env::var("NATS_URL") {
        let v = v.trim();
        if !v.is_empty() {
            nats_url = v.to_string();
        }
    }

    let mut backend_id = loaded
        .value
        .get("dataSources")
        .and_then(|d| d.get("primary"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            loaded
                .value
                .get("broker")
                .and_then(|b| b.get("primary"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("")
        .trim()
        .to_lowercase();
    if let Ok(v) = std::env::var("BACKEND_ID") {
        let v = v.trim().to_lowercase();
        if !v.is_empty() {
            backend_id = v;
        }
    }

    let tick_ms = loaded
        .value
        .get("tui")
        .and_then(|t| t.get("refreshRateMs"))
        .and_then(|v| v.as_u64())
        .unwrap_or(250);
    let snapshot_ttl_secs = loaded
        .value
        .get("tui")
        .and_then(|t| t.get("snapshotTtlSecs"))
        .and_then(|v| v.as_u64())
        .unwrap_or(30);

    let mut errors = Vec::new();
    if backend_id.is_empty() {
        errors
            .push("BACKEND_ID (or dataSources.primary / broker.primary) must be non-empty".into());
    }
    if nats_url.is_empty() {
        errors.push("NATS_URL (or tws.natsUrl) must be non-empty for TUI/backend".into());
    }
    if tick_ms == 0 {
        errors.push("tui.refreshRateMs must be >= 1".into());
    }
    if snapshot_ttl_secs == 0 {
        errors.push("tui.snapshotTtlSecs must be >= 1".into());
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Write a minimal shared JSON example to the given path (for CLI init-config).
pub fn write_example_shared_config(path: &Path) -> std::io::Result<()> {
    let example = r#"{
  "dataSources": { "primary": "ib" },
  "tws": { "natsUrl": "nats://localhost:4222" },
  "strategy": { "symbols": ["SPX", "XSP", "NDX"] },
  "tui": { "refreshRateMs": 250, "snapshotTtlSecs": 30 }
}
"#;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, example)
}
