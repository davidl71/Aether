//! Service Control Module
//!
//! Handles starting, stopping, and monitoring backend services.
//! Uses NATS for secure service control commands.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
  pub name: String,
  pub port: u16,
  pub start_script: PathBuf,
  pub stop_script: PathBuf,
  pub enabled: bool,
  pub auto_restart: bool,
  pub max_restart_attempts: u32,
  pub restart_delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
  pub name: String,
  pub running: bool,
  pub pid: Option<u32>,
  pub enabled: bool,
  pub restart_count: u32,
  pub last_started: Option<chrono::DateTime<chrono::Utc>>,
  pub last_error: Option<String>,
}

pub struct ServiceController {
  configs: Arc<RwLock<HashMap<String, ServiceConfig>>>,
  statuses: Arc<RwLock<HashMap<String, ServiceStatus>>>,
  config_file: PathBuf,
}

impl ServiceController {
  pub fn new(config_file: PathBuf) -> Self {
    Self {
      configs: Arc::new(RwLock::new(HashMap::new())),
      statuses: Arc::new(RwLock::new(HashMap::new())),
      config_file,
    }
  }

  /// Load service configurations from file
  pub async fn load_configs(&self) -> anyhow::Result<()> {
    if !self.config_file.exists() {
      // Create default configs
      let default_configs = self.default_configs();
      self.save_configs(&default_configs).await?;
      return Ok(());
    }

    let content = fs::read_to_string(&self.config_file).await?;
    let configs: HashMap<String, ServiceConfig> = toml::from_str(&content)?;

    let mut configs_map = self.configs.write().await;
    *configs_map = configs;

    // Initialize statuses for all configs
    let mut statuses_map = self.statuses.write().await;
    for (name, config) in configs_map.iter() {
      if !statuses_map.contains_key(name) {
        statuses_map.insert(
          name.clone(),
          ServiceStatus {
            name: name.clone(),
            running: false,
            pid: None,
            enabled: config.enabled,
            restart_count: 0,
            last_started: None,
            last_error: None,
          },
        );
      }
    }

    Ok(())
  }

  async fn save_configs(&self, configs: &HashMap<String, ServiceConfig>) -> anyhow::Result<()> {
    let toml_content = toml::to_string_pretty(configs)?;
    fs::write(&self.config_file, toml_content).await?;
    Ok(())
  }

  fn default_configs(&self) -> HashMap<String, ServiceConfig> {
    let mut configs = HashMap::new();

    let services = vec![
      ("alpaca", 8000, "start_alpaca_service.sh", "stop_alpaca_service.sh"),
      ("ib", 8002, "start_ib_service.sh", "stop_ib_service.sh"),
      ("discount_bank", 8003, "start_discount_bank_service.sh", "stop_discount_bank_service.sh"),
      ("risk_free_rate", 8004, "start_risk_free_rate_service.sh", "stop_risk_free_rate_service.sh"),
      ("tastytrade", 8005, "start_tastytrade_service.sh", "stop_tastytrade_service.sh"),
    ];

    for (name, port, start, stop) in services {
      configs.insert(
        name.to_string(),
        ServiceConfig {
          name: name.to_string(),
          port,
          start_script: PathBuf::from(format!("scripts/{}", start)),
          stop_script: PathBuf::from(format!("scripts/{}", stop)),
          enabled: true,
          auto_restart: true,
          max_restart_attempts: 5,
          restart_delay_seconds: 10,
        },
      );
    }

    configs
  }

  /// Start a service
  pub async fn start_service(&self, service_name: &str) -> anyhow::Result<ServiceStatus> {
    let config = {
      let configs = self.configs.read().await;
      configs.get(service_name).cloned()
        .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_name))?
    };

    if !config.enabled {
      return Err(anyhow::anyhow!("Service {} is disabled", service_name));
    }

    info!(service = %service_name, "Starting service");

    // Check if already running
    if self.is_service_running(&config).await {
      warn!(service = %service_name, "Service already running");
      return self.get_status(service_name).await;
    }

    // Execute start script
    let script_path = self.resolve_script_path(&config.start_script)?;
    let output = Command::new("bash")
      .arg(&script_path)
      .current_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
      .output()?;

    if !output.status.success() {
      let error_msg = String::from_utf8_lossy(&output.stderr);
      error!(service = %service_name, error = %error_msg, "Failed to start service");

      let mut statuses = self.statuses.write().await;
      if let Some(status) = statuses.get_mut(service_name) {
        status.last_error = Some(error_msg.to_string());
      }

      return Err(anyhow::anyhow!("Failed to start service: {}", error_msg));
    }

    // Update status
    let mut statuses = self.statuses.write().await;
    if let Some(status) = statuses.get_mut(service_name) {
      status.running = true;
      status.last_started = Some(chrono::Utc::now());
      status.last_error = None;
      status.pid = self.get_service_pid(&config).await;
    }

    info!(service = %service_name, "Service started successfully");
    Ok(self.get_status(service_name).await)
  }

  /// Stop a service
  pub async fn stop_service(&self, service_name: &str) -> anyhow::Result<ServiceStatus> {
    let config = {
      let configs = self.configs.read().await;
      configs.get(service_name).cloned()
        .ok_or_else(|| anyhow::anyhow!("Service not found: {}", service_name))?
    };

    info!(service = %service_name, "Stopping service");

    // Execute stop script
    let script_path = self.resolve_script_path(&config.stop_script)?;
    let output = Command::new("bash")
      .arg(&script_path)
      .current_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
      .output()?;

    // Update status
    let mut statuses = self.statuses.write().await;
    if let Some(status) = statuses.get_mut(service_name) {
      status.running = false;
      status.pid = None;
    }

    info!(service = %service_name, "Service stopped");
    Ok(self.get_status(service_name).await)
  }

  /// Enable a service
  pub async fn enable_service(&self, service_name: &str) -> anyhow::Result<()> {
    let mut configs = self.configs.write().await;
    if let Some(config) = configs.get_mut(service_name) {
      config.enabled = true;
      self.save_configs(&configs).await?;
      info!(service = %service_name, "Service enabled");
      Ok(())
    } else {
      Err(anyhow::anyhow!("Service not found: {}", service_name))
    }
  }

  /// Disable a service
  pub async fn disable_service(&self, service_name: &str) -> anyhow::Result<()> {
    let mut configs = self.configs.write().await;
    if let Some(config) = configs.get_mut(service_name) {
      config.enabled = false;
      self.save_configs(&configs).await?;

      // Also stop the service if it's running
      drop(configs);
      if self.is_service_running_by_name(service_name).await {
        let _ = self.stop_service(service_name).await;
      }

      info!(service = %service_name, "Service disabled");
      Ok(())
    } else {
      Err(anyhow::anyhow!("Service not found: {}", service_name))
    }
  }

  /// Get service status
  pub async fn get_status(&self, service_name: &str) -> ServiceStatus {
    let statuses = self.statuses.read().await;
    statuses.get(service_name).cloned().unwrap_or_else(|| ServiceStatus {
      name: service_name.to_string(),
      running: false,
      pid: None,
      enabled: false,
      restart_count: 0,
      last_started: None,
      last_error: None,
    })
  }

  /// Check if service is running
  async fn is_service_running(&self, config: &ServiceConfig) -> bool {
    // Check if port is in use
    let output = Command::new("lsof")
      .arg("-ti")
      .arg(format!(":{}", config.port))
      .output();

    match output {
      Ok(result) => result.status.success(),
      Err(_) => false,
    }
  }

  async fn is_service_running_by_name(&self, service_name: &str) -> bool {
    let configs = self.configs.read().await;
    if let Some(config) = configs.get(service_name) {
      self.is_service_running(config).await
    } else {
      false
    }
  }

  async fn get_service_pid(&self, config: &ServiceConfig) -> Option<u32> {
    let output = Command::new("lsof")
      .arg("-ti")
      .arg(format!(":{}", config.port))
      .output()
      .ok()?;

    if output.status.success() {
      let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
      pid_str.parse().ok()
    } else {
      None
    }
  }

  fn resolve_script_path(&self, script: &PathBuf) -> anyhow::Result<PathBuf> {
    // Try relative to current directory
    if script.is_absolute() {
      return Ok(script.clone());
    }

    // Try relative to project root
    let project_root = std::env::current_dir()?;
    let full_path = project_root.join(script);

    if full_path.exists() {
      Ok(full_path)
    } else {
      Err(anyhow::anyhow!("Script not found: {}", full_path.display()))
    }
  }
}
