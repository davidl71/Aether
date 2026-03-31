//! TUI configuration loaded from the shared config file with env overrides.

use std::path::PathBuf;

use api::project_paths::shared_config_candidate_paths;
use serde::Deserialize;
use tracing::{info, warn};

const DEFAULT_NATS_URL: &str = "nats://localhost:4222";
const DEFAULT_BACKEND_ID: &str = "ib";
const DEFAULT_REST_URL: &str = "http://localhost:9090";
const DEFAULT_WATCHLIST: &str = "SPX,XSP,NDX";
const DEFAULT_SNAPSHOT_TTL_SECS: u64 = 30;
const DEFAULT_TUI_THEME: &str = "default";

/// Overall color theme (operator preference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiTheme {
    /// Default palette (current styling).
    #[default]
    Default,
    /// Higher-contrast palette for bright terminals / accessibility.
    HighContrast,
}

impl TuiTheme {
    pub fn parse_env(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "default" | "normal" => Some(Self::Default),
            "high_contrast" | "high-contrast" | "contrast" | "hc" => Some(Self::HighContrast),
            _ => None,
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Default => Self::HighContrast,
            Self::HighContrast => Self::Default,
        }
    }

    pub const fn as_setting_value(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::HighContrast => "high_contrast",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::HighContrast => "high contrast",
        }
    }
}

/// How the Positions tab orders rows (operator preference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionsSortMode {
    /// Snapshot / backend order (no reordering).
    #[default]
    AsReceived,
    /// Alphabetical by full symbol string.
    Symbol,
    /// Highest absolute unrealized P&L first (ties: symbol).
    UnrealizedPnlDesc,
}

impl PositionsSortMode {
    pub fn parse_env(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "as_received" | "default" | "backend" => Some(Self::AsReceived),
            "symbol" | "sym" => Some(Self::Symbol),
            "pnl" | "unrealized_pnl" | "unrealized" => Some(Self::UnrealizedPnlDesc),
            _ => None,
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::AsReceived => Self::Symbol,
            Self::Symbol => Self::UnrealizedPnlDesc,
            Self::UnrealizedPnlDesc => Self::AsReceived,
        }
    }

    pub fn as_setting_value(self) -> &'static str {
        match self {
            Self::AsReceived => "as_received",
            Self::Symbol => "symbol",
            Self::UnrealizedPnlDesc => "pnl",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::AsReceived => "as received",
            Self::Symbol => "symbol",
            Self::UnrealizedPnlDesc => "pnl desc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingScope {
    Editable,
    EnvOnly,
    BuiltIn,
}

impl SettingScope {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Editable => "editable",
            Self::EnvOnly => "env-only",
            Self::BuiltIn => "built-in",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TuiConfig {
    /// NATS server URL (env: NATS_URL)
    pub nats_url: String,
    /// Backend identifier used as the snapshot topic suffix (env: BACKEND_ID)
    pub backend_id: String,
    /// Legacy REST endpoint compatibility setting. The active TUI runtime is NATS-first.
    pub rest_url: String,
    /// Symbols to highlight in the dashboard (env: WATCHLIST, comma-separated)
    pub watchlist: Vec<String>,
    /// UI redraw interval in milliseconds (env: TICK_MS)
    pub tick_ms: u64,
    /// Legacy REST polling compatibility setting.
    pub rest_poll_ms: u64,
    /// Legacy REST fallback compatibility flag.
    pub rest_fallback: bool,
    /// Seconds before a snapshot is considered stale for display purposes (env: SNAPSHOT_TTL_SECS)
    pub snapshot_ttl_secs: u64,
    /// Enable split-pane layout (env: SPLIT_PANE=1)
    pub split_pane: bool,
    /// How often to re-fetch benchmark rates (SOFR/Treasury from FRED) in seconds.
    /// FRED updates at most once per business day, so a long interval is fine.
    /// (env: BENCHMARKS_REFRESH_SECS, default: 3600)
    pub benchmarks_refresh_secs: u64,
    /// NATS KV bucket name for yield curves (env: NATS_KV_BUCKET, default: LIVE_STATE)
    pub yield_kv_bucket: String,
    /// Positions table sort (env: TUI_POSITIONS_SORT: as_received | symbol | pnl)
    pub positions_sort: PositionsSortMode,
    /// Theme (env: TUI_THEME: default | high_contrast)
    pub theme: TuiTheme,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            nats_url: DEFAULT_NATS_URL.into(),
            backend_id: DEFAULT_BACKEND_ID.into(),
            rest_url: DEFAULT_REST_URL.into(),
            watchlist: parse_watchlist(DEFAULT_WATCHLIST),
            tick_ms: 250,
            rest_poll_ms: 2000,
            rest_fallback: false,
            snapshot_ttl_secs: DEFAULT_SNAPSHOT_TTL_SECS,
            split_pane: false,
            benchmarks_refresh_secs: 3600,
            yield_kv_bucket: "LIVE_STATE".into(),
            positions_sort: PositionsSortMode::default(),
            theme: TuiTheme::parse_env(DEFAULT_TUI_THEME).unwrap_or_default(),
        }
    }
}

impl TuiConfig {
    pub fn load() -> Self {
        let mut config = Self::default();

        match load_shared_config() {
            Ok(Some(loaded)) => {
                config.apply_shared_config(&loaded.config);
                info!(path = %loaded.path.display(), "Loaded shared TUI config");
            }
            Ok(None) => {
                info!("No shared config found; using defaults and env overrides");
            }
            Err(err) => {
                warn!(error = %err, "Failed to load shared config; using defaults and env overrides");
            }
        }

        config.apply_env_overrides();
        config
    }

    #[allow(dead_code)] // TBD: expose via API or TUI status bar
    pub fn snapshot_stale_after_secs(&self) -> i64 {
        self.rest_poll_ms.max(1000).div_ceil(1000) as i64
    }

    pub fn validate(&self, require_nats: bool) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if require_nats && !self.rest_fallback && !self.nats_url.starts_with("nats://") {
            errors.push("NATS URL must use nats:// scheme when NATS is required".to_string());
        }
        if self.tick_ms == 0 {
            errors.push("TICK_MS must be > 0".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn apply_shared_config(&mut self, shared: &SharedConfigFile) {
        if let Some(nats_url) = shared.tws.nats_url.as_deref() {
            self.nats_url = nats_url.trim().to_string();
        }

        if let Some(backend_id) = shared
            .data_sources
            .primary
            .as_deref()
            .or(shared.broker.primary.as_deref())
        {
            let normalized = backend_id.trim().to_lowercase();
            if !normalized.is_empty() {
                self.backend_id = normalized;
            }
        }

        if let Some(rest_url) = shared
            .tui
            .api_base_url
            .as_deref()
            .or(shared.tui.rest_endpoint.as_deref())
            .map(normalize_rest_base_url)
            .filter(|value| !value.is_empty())
        {
            self.rest_url = rest_url;
        }

        if let Some(tick_ms) = shared.tui.refresh_rate_ms {
            self.tick_ms = tick_ms.max(1);
        }

        if let Some(rest_poll_ms) = shared.tui.update_interval_ms {
            self.rest_poll_ms = rest_poll_ms.max(1);
        }

        if let Some(symbols) = shared.strategy.symbols.as_ref() {
            let watchlist = normalize_symbols(symbols.iter().map(String::as_str));
            if !watchlist.is_empty() {
                self.watchlist = watchlist;
            }
        }

        if let Some(explicit_rest_fallback) = shared.tui.rest_fallback {
            self.rest_fallback = explicit_rest_fallback;
        } else if let Some(provider_type) = shared.tui.provider_type.as_deref() {
            match provider_type.trim().to_ascii_lowercase().as_str() {
                "nats" => self.rest_fallback = false,
                "rest" => self.rest_fallback = true,
                _ => {}
            }
        }

        if let Some(theme) = shared.tui.theme.as_deref() {
            if let Some(parsed) = TuiTheme::parse_env(theme) {
                self.theme = parsed;
            }
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(value) = std::env::var("NATS_URL") {
            let value = value.trim();
            if !value.is_empty() {
                self.nats_url = value.to_string();
            }
        }

        if let Ok(value) = std::env::var("BACKEND_ID") {
            let value = value.trim().to_lowercase();
            if !value.is_empty() {
                self.backend_id = value;
            }
        }

        if let Ok(value) = std::env::var("REST_URL") {
            let value = normalize_rest_base_url(&value);
            if !value.is_empty() {
                self.rest_url = value;
            }
        }

        if let Ok(value) = std::env::var("WATCHLIST") {
            let value = parse_watchlist(&value);
            if !value.is_empty() {
                self.watchlist = value;
            }
        }

        if let Ok(value) = std::env::var("TICK_MS") {
            if let Ok(parsed) = value.parse::<u64>() {
                self.tick_ms = parsed.max(1);
            }
        }

        if let Ok(value) = std::env::var("REST_POLL_MS") {
            if let Ok(parsed) = value.parse::<u64>() {
                self.rest_poll_ms = parsed.max(1);
            }
        }

        if let Ok(value) = std::env::var("REST_FALLBACK") {
            if let Some(parsed) = parse_bool(&value) {
                self.rest_fallback = parsed;
            }
        }

        if let Ok(value) = std::env::var("SNAPSHOT_TTL_SECS") {
            if let Ok(parsed) = value.parse::<u64>() {
                self.snapshot_ttl_secs = parsed.max(1);
            }
        }

        if let Ok(value) = std::env::var("SPLIT_PANE") {
            if let Some(parsed) = parse_bool(&value) {
                self.split_pane = parsed;
            }
        }

        if let Ok(value) = std::env::var("BENCHMARKS_REFRESH_SECS") {
            if let Ok(parsed) = value.parse::<u64>() {
                self.benchmarks_refresh_secs = parsed.max(60);
            }
        }

        if let Ok(value) = std::env::var("NATS_KV_BUCKET") {
            let value = value.trim().to_string();
            if !value.is_empty() {
                self.yield_kv_bucket = value;
            }
        }

        if let Ok(value) = std::env::var("TUI_POSITIONS_SORT") {
            if let Some(mode) = PositionsSortMode::parse_env(&value) {
                self.positions_sort = mode;
            }
        }

        if let Ok(value) = std::env::var("TUI_THEME") {
            if let Some(theme) = TuiTheme::parse_env(&value) {
                self.theme = theme;
            }
        }
    }

    #[cfg(test)]
    fn from_shared_str_for_test(raw: &str) -> Self {
        let value = jsonc_parser::parse_to_serde_value(raw, &Default::default())
            .expect("parse ok")
            .expect("valid test config");
        let shared: SharedConfigFile = serde_json::from_value(value).expect("valid test config");
        let mut config = Self::default();
        config.apply_shared_config(&shared);
        config
    }
}

pub fn config_key_scope(key: &str) -> SettingScope {
    match key {
        "NATS_URL"
        | "BACKEND_ID"
        | "TICK_MS"
        | "SNAPSHOT_TTL_SECS"
        | "SPLIT_PANE"
        | "BENCHMARKS_REFRESH_SECS"
        | "NATS_KV_BUCKET"
        | "TUI_POSITIONS_SORT"
        | "TUI_THEME" => SettingScope::Editable,
        // Legacy REST compatibility knobs are still parsed from file/env, but
        // they are not operator-editable from the active NATS-first Settings UI.
        "REST_URL" | "REST_POLL_MS" | "REST_FALLBACK" => SettingScope::EnvOnly,
        _ => SettingScope::EnvOnly,
    }
}

pub fn credential_scope(provider: &str) -> SettingScope {
    match provider {
        "yahoo" | "mock" | "tws" => SettingScope::BuiltIn,
        "fmp" | "polygon" | "tase" | "fred" => SettingScope::EnvOnly,
        _ => SettingScope::EnvOnly,
    }
}

#[derive(Debug, Deserialize, Default)]
struct SharedConfigFile {
    #[serde(default, alias = "dataSources")]
    data_sources: SharedDataSources,
    #[serde(default)]
    tui: SharedTuiConfig,
    #[serde(default)]
    tws: SharedTwsConfig,
    #[serde(default)]
    strategy: SharedStrategyConfig,
    #[serde(default)]
    broker: SharedBrokerConfig,
}

#[derive(Debug, Deserialize, Default)]
struct SharedDataSources {
    #[serde(default)]
    primary: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct SharedTuiConfig {
    #[serde(default, alias = "providerType")]
    provider_type: Option<String>,
    #[serde(default, alias = "updateIntervalMs")]
    update_interval_ms: Option<u64>,
    #[serde(default, alias = "refreshRateMs")]
    refresh_rate_ms: Option<u64>,
    #[serde(default, alias = "restEndpoint")]
    rest_endpoint: Option<String>,
    #[serde(default, alias = "apiBaseUrl")]
    api_base_url: Option<String>,
    #[serde(default, alias = "restFallback")]
    rest_fallback: Option<bool>,
    #[serde(default)]
    theme: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct SharedTwsConfig {
    #[serde(default, alias = "natsUrl")]
    nats_url: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct SharedStrategyConfig {
    #[serde(default)]
    symbols: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct SharedBrokerConfig {
    #[serde(default)]
    primary: Option<String>,
}

#[derive(Debug)]
struct LoadedSharedConfig {
    path: PathBuf,
    config: SharedConfigFile,
}

fn load_shared_config() -> Result<Option<LoadedSharedConfig>, String> {
    let candidates = shared_config_candidate_paths();
    let mut last_error = None;

    for candidate in candidates {
        if !candidate.exists() || !candidate.is_file() {
            continue;
        }

        match std::fs::read_to_string(&candidate) {
            Ok(raw) => {
                let value = jsonc_parser::parse_to_serde_value(&raw, &Default::default())
                    .map_err(|e| {
                        format!(
                            "Failed to parse shared config at {}: {e}",
                            candidate.display()
                        )
                    })?
                    .ok_or_else(|| format!("Empty shared config at {}", candidate.display()))?;
                let config = serde_json::from_value::<SharedConfigFile>(value).map_err(|err| {
                    format!(
                        "Failed to parse shared config at {}: {err}",
                        candidate.display()
                    )
                })?;
                return Ok(Some(LoadedSharedConfig {
                    path: candidate,
                    config,
                }));
            }
            Err(err) => {
                last_error = Some(format!("{}: {err}", candidate.display()));
            }
        }
    }

    if let Some(err) = last_error {
        return Err(err);
    }

    Ok(None)
}

fn normalize_rest_base_url(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    for suffix in ["/api/v1/snapshot", "/api/snapshot", "/api"] {
        if let Some(stripped) = trimmed.strip_suffix(suffix) {
            return stripped.trim_end_matches('/').to_string();
        }
    }

    trimmed.to_string()
}

fn parse_watchlist(value: &str) -> Vec<String> {
    normalize_symbols(value.split(','))
}

fn normalize_symbols<'a>(symbols: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    symbols
        .into_iter()
        .map(|symbol| symbol.trim().to_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        sync::{Mutex, OnceLock},
    };

    use api::project_paths::discover_workspace_root;

    use super::{config_key_scope, shared_config_candidate_paths, SettingScope, TuiConfig};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn shared_config_maps_nats_primary_and_rest_fallback() {
        let config = TuiConfig::from_shared_str_for_test(
            r#"
            {
              "dataSources": {
                "primary": "ib"
              },
              "tws": {
                "nats_url": "nats://shared:4222"
              },
              "strategy": {
                "symbols": ["spx", "xsp"]
              },
              "tui": {
                "providerType": "rest",
                "apiBaseUrl": "http://shared:8080",
                "refreshRateMs": 125,
                "updateIntervalMs": 1400
              }
            }
            "#,
        );

        assert_eq!(config.backend_id, "ib");
        assert_eq!(config.nats_url, "nats://shared:4222");
        assert_eq!(config.rest_url, "http://shared:8080");
        assert_eq!(config.watchlist, vec!["SPX", "XSP"]);
        assert_eq!(config.tick_ms, 125);
        assert_eq!(config.rest_poll_ms, 1400);
        assert!(config.rest_fallback);
    }

    #[test]
    fn shared_config_nats_provider_disables_rest_fallback_and_normalizes_endpoint() {
        let config = TuiConfig::from_shared_str_for_test(
            r#"
            {
              "broker": {
                "primary": "IB"
              },
              "tui": {
                "provider_type": "nats",
                "rest_endpoint": "http://localhost:8080/api/v1/snapshot"
              }
            }
            "#,
        );

        assert_eq!(config.backend_id, "ib");
        assert_eq!(config.rest_url, "http://localhost:8080");
        assert!(!config.rest_fallback);
    }

    #[test]
    fn candidate_paths_include_workspace_config_from_manifest_root() {
        let workspace_root = discover_workspace_root().expect("workspace root");
        let expected = workspace_root.join("config/config.example.json");

        assert!(expected.is_file());
        assert!(shared_config_candidate_paths().contains(&expected));
    }

    #[test]
    fn load_prefers_shared_config_then_env_overrides() {
        let _guard = env_lock().lock().expect("env lock");
        let config_path = env::temp_dir().join(format!(
            "tui-config-{}-{}.json",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ));
        let raw = r#"
        {
          "dataSources": {
            "primary": "portal"
          },
          "broker": {
            "primary": "ib"
          },
          "tws": {
            "natsUrl": "nats://shared-home:4333"
          },
          "strategy": {
            "symbols": ["spy", "qqq", "iwm"]
          },
          "tui": {
            "providerType": "rest",
            "apiBaseUrl": "http://shared-home:9091/api/v1/snapshot",
            "refreshRateMs": 175,
            "updateIntervalMs": 3100
          }
        }
        "#;
        fs::write(&config_path, raw).expect("write config");

        env::set_var("IB_BOX_SPREAD_CONFIG", &config_path);
        env::set_var("NATS_URL", "nats://env-override:4223");
        env::set_var("BACKEND_ID", "alpaca");
        env::set_var("REST_URL", "http://env-override:8181/api");
        env::set_var("WATCHLIST", "msft, nvda");
        env::set_var("REST_FALLBACK", "0");

        let loaded = TuiConfig::load();

        assert_eq!(loaded.nats_url, "nats://env-override:4223");
        assert_eq!(loaded.backend_id, "alpaca");
        assert_eq!(loaded.rest_url, "http://env-override:8181");
        assert_eq!(loaded.watchlist, vec!["MSFT", "NVDA"]);
        assert_eq!(loaded.tick_ms, 175);
        assert_eq!(loaded.rest_poll_ms, 3100);
        assert!(!loaded.rest_fallback);

        env::remove_var("IB_BOX_SPREAD_CONFIG");
        env::remove_var("NATS_URL");
        env::remove_var("BACKEND_ID");
        env::remove_var("REST_URL");
        env::remove_var("WATCHLIST");
        env::remove_var("REST_FALLBACK");
        let _ = fs::remove_file(config_path);
    }

    #[test]
    fn config_scope_marks_legacy_rest_knobs_as_env_only() {
        assert_eq!(config_key_scope("NATS_URL"), SettingScope::Editable);
        assert_eq!(config_key_scope("REST_URL"), SettingScope::EnvOnly);
        assert_eq!(config_key_scope("REST_POLL_MS"), SettingScope::EnvOnly);
        assert_eq!(config_key_scope("REST_FALLBACK"), SettingScope::EnvOnly);
        assert_eq!(
            config_key_scope("TUI_POSITIONS_SORT"),
            SettingScope::Editable
        );
    }
}
