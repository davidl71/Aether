//! Aether CLI - Command-line interface for the trading platform
//!
//! Usage:
//!   cargo run -p cli -- --config config.toml --dry-run
//!   cargo run -p cli -- --init-config config.toml

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use tracing::{info, warn};

mod config;

#[derive(Parser, Debug)]
#[command(name = "aether")]
#[command(about = "Aether - Multi-asset synthetic financing platform", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config/config.toml")]
    config: PathBuf,

    /// Initialize a sample configuration file and exit
    #[arg(long)]
    init_config: Option<PathBuf>,

    /// Dry run mode - simulate trading without executing orders
    #[arg(long)]
    dry_run: bool,

    /// Validate configuration and exit
    #[arg(long)]
    validate: bool,

    /// Use mock TWS client (no live IB connection)
    #[arg(long)]
    mock_tws: bool,

    /// Disable periodic JSON snapshot writing
    #[arg(long)]
    no_snapshot: bool,

    /// Path for JSON snapshot file
    #[arg(long, default_value = "data/snapshot.json")]
    snapshot_path: PathBuf,

    /// Override logging level
    #[arg(long, value_enum)]
    log_level: Option<LogLevel>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Debug, ValueEnum)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle init-config first
    if let Some(path) = cli.init_config {
        let config = config::Config::default();
        config::save_sample_config(&path, &config)?;
        println!("Sample configuration written to: {}", path.display());
        println!("Update the values before running live.");
        return Ok(());
    }

    // Load configuration
    let mut config = config::load_config(&cli.config)?;

    // Apply CLI overrides
    if cli.dry_run {
        config.dry_run = true;
    }
    if cli.mock_tws {
        config.tws.use_mock = true;
    }
    if cli.no_snapshot {
        config.snapshot.enabled = false;
    }
    if cli.verbose {
        config.logging.level = "debug".to_string();
    }
    if let Some(level) = cli.log_level {
        config.logging.level = match level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }.to_string();
    }

    // Validate configuration
    config::validate_config(&config)?;

    if cli.validate {
        println!("Configuration validation successful!");
        return Ok(());
    }

    // Setup logging
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    // Print banner
    print_banner();

    if config.dry_run {
        warn!("DRY RUN MODE - No real trades will be executed");
    } else {
        warn!("LIVE TRADING MODE - Real money at risk!");
        warn!("Press Ctrl+C to stop safely");
    }

    // Run the trading loop
    info!("Initializing components...");
    trading_loop(&config)?;

    Ok(())
}

fn print_banner() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║         Aether - Multi-Asset Synthetic Financing          ║");
    println!("║                    Trading Platform                        ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
}

fn trading_loop(_config: &config::Config) -> Result<()> {
    info!("Trading loop started - use tui_service for full interface");
    info!("Run 'cargo run -p tui_service' for the terminal UI");
    
    std::thread::park();
    
    Ok(())
}
