//! Persist Charts tab symbol search history under [`api::credentials::aether_config_dir`].

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use api::credentials::aether_config_dir;
use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "tui_chart_search_history.json";

pub const CHART_SEARCH_HISTORY_MAX: usize = 10;

#[derive(Debug, Serialize, Deserialize, Default)]
struct FilePayload {
    symbols: Vec<String>,
}

pub fn chart_search_history_path() -> PathBuf {
    aether_config_dir().join(FILE_NAME)
}

pub fn load_chart_search_history() -> VecDeque<String> {
    let path = chart_search_history_path();
    let Ok(data) = fs::read_to_string(&path) else {
        return VecDeque::with_capacity(CHART_SEARCH_HISTORY_MAX);
    };
    let Ok(payload) = serde_json::from_str::<FilePayload>(&data) else {
        tracing::warn!(
            path = %path.display(),
            "invalid chart search history JSON, ignoring"
        );
        return VecDeque::with_capacity(CHART_SEARCH_HISTORY_MAX);
    };
    let mut dq = VecDeque::with_capacity(CHART_SEARCH_HISTORY_MAX.min(payload.symbols.len()));
    for s in payload.symbols.into_iter().take(CHART_SEARCH_HISTORY_MAX) {
        let t = s.trim().to_string();
        if !t.is_empty() && !dq.contains(&t) {
            dq.push_back(t);
        }
    }
    dq
}

pub fn save_chart_search_history(history: &VecDeque<String>) {
    let dir = aether_config_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        tracing::warn!(error = %e, "failed to create aether config dir for chart history");
        return;
    }
    let path = chart_search_history_path();
    let symbols: Vec<String> = history
        .iter()
        .take(CHART_SEARCH_HISTORY_MAX)
        .cloned()
        .collect();
    let payload = FilePayload { symbols };
    let Ok(json) = serde_json::to_string_pretty(&payload) else {
        tracing::warn!("failed to serialize chart search history");
        return;
    };
    if let Err(e) = fs::write(&path, json) {
        tracing::warn!(
            error = %e,
            path = %path.display(),
            "failed to write chart search history"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_payload_roundtrip() {
        let p = FilePayload {
            symbols: vec!["SPY".into(), "QQQ".into()],
        };
        let j = serde_json::to_string(&p).unwrap();
        let q: FilePayload = serde_json::from_str(&j).unwrap();
        assert_eq!(q.symbols, p.symbols);
    }
}
