//! Configuration module for Aether CLI

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub dry_run: bool,

    #[serde(default)]
    pub tws: TwsConfig,

    #[serde(default)]
    pub strategy: StrategyConfig,

    #[serde(default)]
    pub broker: BrokerConfig,

    #[serde(default)]
    pub commission: CommissionConfig,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub snapshot: SnapshotConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dry_run: true,
            tws: TwsConfig::default(),
            strategy: StrategyConfig::default(),
            broker: BrokerConfig::default(),
            commission: CommissionConfig::default(),
            logging: LoggingConfig::default(),
            snapshot: SnapshotConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TwsConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_client_id")]
    pub client_id: u32,
    #[serde(default = "default_timeout")]
    pub connection_timeout_ms: u64,
    #[serde(default)]
    pub auto_reconnect: bool,
    #[serde(default = "default_max_reconnect")]
    pub max_reconnect_attempts: u32,
    #[serde(default)]
    pub use_mock: bool,
    #[serde(default)]
    pub log_raw_messages: bool,
    #[serde(default)]
    pub connect_options: String,
    #[serde(default)]
    pub enable_pcap_capture: bool,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    7497
}
fn default_client_id() -> u32 {
    1
}
fn default_timeout() -> u64 {
    60000
}
fn default_max_reconnect() -> u32 {
    10
}

impl Default for TwsConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            client_id: default_client_id(),
            connection_timeout_ms: default_timeout(),
            auto_reconnect: true,
            max_reconnect_attempts: default_max_reconnect(),
            use_mock: false,
            log_raw_messages: false,
            connect_options: "+PACEAPI".to_string(),
            enable_pcap_capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StrategyConfig {
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(default = "default_min_dte")]
    pub min_dte: u32,
    #[serde(default = "default_max_dte")]
    pub max_dte: u32,
    #[serde(default = "default_min_strike")]
    pub min_strike_width: f64,
    #[serde(default = "default_max_position")]
    pub max_position_size: u32,
    #[serde(default = "default_risk_tolerance")]
    pub risk_tolerance: f64,
}

fn default_min_dte() -> u32 {
    30
}
fn default_max_dte() -> u32 {
    60
}
fn default_min_strike() -> f64 {
    5.0
}
fn default_max_position() -> u32 {
    10
}
fn default_risk_tolerance() -> f64 {
    0.02
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            symbols: vec!["SPX".to_string(), "XSP".to_string()],
            min_dte: default_min_dte(),
            max_dte: default_max_dte(),
            min_strike_width: default_min_strike(),
            max_position_size: default_max_position(),
            risk_tolerance: default_risk_tolerance(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BrokerConfig {
    #[serde(default)]
    pub priorities: Vec<String>,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            priorities: vec!["mock".to_string(), "ib".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CommissionConfig {
    #[serde(default = "default_true")]
    pub use_ibkr_pro_rates: bool,
    #[serde(default = "default_fee")]
    pub per_contract_fee: f64,
    #[serde(default = "default_min_fee")]
    pub minimum_order_fee: f64,
}

fn default_true() -> bool {
    true
}
fn default_fee() -> f64 {
    0.65
}
fn default_min_fee() -> f64 {
    1.00
}

impl Default for CommissionConfig {
    fn default() -> Self {
        Self {
            use_ibkr_pro_rates: true,
            per_contract_fee: default_fee(),
            minimum_order_fee: default_min_fee(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub file: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapshotConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_snapshot_interval")]
    pub interval_secs: u64,
    #[serde(default = "default_snapshot_path")]
    pub path: String,
}

fn default_snapshot_interval() -> u64 {
    60
}
fn default_snapshot_path() -> String {
    "data/snapshot.json".to_string()
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: default_snapshot_interval(),
            path: default_snapshot_path(),
        }
    }
}

pub fn load_config(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;
    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config from {}", path.display()))?;
    Ok(config)
}

pub fn validate_config(config: &Config) -> Result<()> {
    if config.strategy.symbols.is_empty() {
        anyhow::bail!("At least one symbol must be configured");
    }
    if config.strategy.min_dte > config.strategy.max_dte {
        anyhow::bail!("min_dte must be less than or equal to max_dte");
    }
    if config.tws.port == 0 {
        anyhow::bail!("Invalid port number");
    }
    if config.tws.client_id > 999 {
        anyhow::bail!("Client ID must be between 0 and 999");
    }
    Ok(())
}

pub fn save_sample_config(path: &Path, config: &Config) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(config)?;
    fs::write(path, contents)?;
    Ok(())
}
