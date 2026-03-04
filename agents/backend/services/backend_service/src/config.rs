use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
  #[serde(default = "default_rest_addr")]
  pub rest_addr: SocketAddr,
  #[serde(default)]
  pub market_data: MarketDataSettings,
}

impl Default for BackendConfig {
  fn default() -> Self {
    Self {
      rest_addr: default_rest_addr(),
      market_data: MarketDataSettings::default(),
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketDataSettings {
  #[serde(default = "default_market_provider")]
  pub provider: String,
  #[serde(default = "default_market_symbols")]
  pub symbols: Vec<String>,
  #[serde(default = "default_poll_interval_ms")]
  pub poll_interval_ms: u64,
  #[serde(default)]
  pub polygon: Option<PolygonSettings>,
}

impl Default for MarketDataSettings {
  fn default() -> Self {
    Self {
      provider: default_market_provider(),
      symbols: default_market_symbols(),
      poll_interval_ms: default_poll_interval_ms(),
      polygon: None,
    }
  }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PolygonSettings {
  pub api_key: Option<String>,
  pub api_key_env: Option<String>,
  pub base_url: Option<String>,
}

fn default_rest_addr() -> SocketAddr {
  SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
}

fn default_market_provider() -> String {
  "mock".into()
}

pub fn default_market_symbols() -> Vec<String> {
  vec!["SPX".into(), "XSP".into(), "NDX".into()]
}

fn default_poll_interval_ms() -> u64 {
  800
}

pub fn load() -> anyhow::Result<BackendConfig> {
  let path = std::env::var("BACKEND_CONFIG").unwrap_or_else(|_| "config/default.toml".into());

  if std::path::Path::new(&path).exists() {
    let data = std::fs::read_to_string(&path)
      .with_context(|| format!("unable to read config file {path}"))?;
    let cfg: BackendConfig = toml::from_str(&data)
      .with_context(|| format!("invalid config file {path}"))?;
    Ok(cfg)
  } else {
    Ok(BackendConfig::default())
  }
}

pub fn resolve_polygon_api_key(settings: &PolygonSettings) -> anyhow::Result<String> {
  if let Some(key) = settings.api_key.clone() {
    return Ok(key);
  }

  if let Some(env) = &settings.api_key_env {
    if let Ok(val) = std::env::var(env) {
      anyhow::ensure!(!val.trim().is_empty(), "environment variable {env} is set but empty");
      return Ok(val);
    }
    anyhow::bail!("environment variable {env} not found for polygon API key");
  }

  anyhow::bail!("polygon API key not configured (set market_data.polygon.api_key or api_key_env)");
}
