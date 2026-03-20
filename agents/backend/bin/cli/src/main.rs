//! Aether CLI - Command-line interface for the trading platform
//!
//! Usage:
//!   cargo run -p cli -- init-config config/config.json
//!   cargo run -p cli -- validate
//!   cargo run -p cli -- run --dry-run
//!   cargo run -p cli -- snapshot
//!
//! Backward compatibility: --init-config and --validate flags still work.

use std::path::PathBuf;

use anyhow::Result;
use api::finance_rates::{
    build_curve, get_sofr_rates, get_treasury_rates, BenchmarksResponse, CurveRequest,
    CurveResponse,
};
use api::loans::LoanRecord;
use chrono::Utc;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use nats_adapter::{
    request_json_with_retry, request_json_with_retry_timeout, NatsClient, RetryConfig,
};
use reqwest::Client as ReqwestClient;
use std::time::Duration;
use tracing::{info, warn};

mod config;

#[derive(Parser, Debug)]
#[command(name = "aether")]
#[command(about = "Aether - Multi-asset synthetic financing platform", long_about = None)]
struct Cli {
    /// Configuration file path (used by validate, run, snapshot). Same discovery as TUI: shared JSON first, then this path. Prefer config/config.json; TOML supported as CLI-only override.
    #[arg(short, long, global = true, default_value = "config/config.json")]
    config: PathBuf,

    /// Initialize a sample configuration file and exit (deprecated: prefer 'aether init-config')
    #[arg(long, global = true)]
    init_config: Option<PathBuf>,

    /// Validate configuration and exit (deprecated: prefer 'aether validate')
    #[arg(long, global = true)]
    validate: bool,

    /// Generate shell completion script and exit (for scripts/generate_completions.sh)
    #[arg(long, global = true, hide = true, value_enum)]
    generate_completion: Option<CompletionShell>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Write a sample config file (JSON or TOML by path extension) and exit
    InitConfig {
        /// Output path (default: config/config.json)
        #[arg(default_value = "config/config.json")]
        path: PathBuf,
    },
    /// Validate shared JSON config (same discovery as TUI: IB_BOX_SPREAD_CONFIG, then config/config.json) and exit 0/1
    Validate,
    /// Load config and run the trading loop (placeholder; use tui_service for full UI)
    Run {
        /// Dry run mode - simulate trading without executing orders
        #[arg(long)]
        dry_run: bool,

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
    },
    /// Validate config and print snapshot path/status (no publishing)
    Snapshot {
        /// Path for JSON snapshot file
        #[arg(long, default_value = "data/snapshot.json")]
        snapshot_path: PathBuf,

        /// Disable snapshot (report disabled)
        #[arg(long)]
        no_snapshot: bool,
    },
    /// Fetch FRED benchmarks (SOFR + Treasury) for testing without backend/TUI. Requires FRED_API_KEY.
    Benchmarks {
        /// Output as JSON (default: pretty-printed summary)
        #[arg(long)]
        json: bool,
    },
    /// Display box spread yield curve. Use --source to choose TWS (direct) or NATS (backend).
    YieldCurve {
        /// Symbol (default SPX)
        #[arg(short, long, default_value = "SPX")]
        symbol: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Trigger backend to refresh yield curve KV before fetching (only when source is nats)
        #[arg(long)]
        refresh: bool,
        /// Data source: nats (backend via NATS) or tws (direct TWS, no backend)
        #[arg(long, default_value = "nats", value_parser = ["nats", "tws"])]
        source: String,
        /// After building curve, publish result to NATS subject yield_curve.direct.{symbol}
        #[arg(long)]
        publish_to_nats: bool,
    },
    /// Force backend to write current snapshot to NATS (point-in-time). Requires NATS and backend_service.
    SnapshotWrite {
        /// Output reply as JSON
        #[arg(long)]
        json: bool,
    },
    /// Loans API (list from backend via NATS api.loans.list).
    Loans {
        #[command(subcommand)]
        sub: LoansCmd,
    },
}

#[derive(Subcommand, Debug)]
enum LoansCmd {
    /// List all loans (NATS api.loans.list).
    List {
        /// Output as JSON (default: table)
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
}

impl From<CompletionShell> for Shell {
    fn from(s: CompletionShell) -> Self {
        match s {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
        }
    }
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

    // Generate shell completion and exit (used by scripts/generate_completions.sh)
    if let Some(shell) = cli.generate_completion {
        let mut cmd = Cli::command();
        let gen: Shell = shell.into();
        clap_complete::generate(gen, &mut cmd, "aether", &mut std::io::stdout());
        return Ok(());
    }

    // Backward compatibility: flags take precedence when no subcommand
    if cli.command.is_none() {
        if let Some(path) = cli.init_config {
            run_init_config(&path)?;
            return Ok(());
        }
        if cli.validate {
            run_validate(&cli.config)?;
            return Ok(());
        }
        // Default: run with no extra flags (config only)
        run_run(
            &cli.config,
            RunOpts {
                dry_run: false,
                mock_tws: false,
                no_snapshot: false,
                snapshot_path: PathBuf::from("data/snapshot.json"),
                log_level: None,
                verbose: false,
            },
        )?;
        return Ok(());
    }

    match cli.command.unwrap() {
        Commands::InitConfig { path } => run_init_config(&path)?,
        Commands::Validate => run_validate(&cli.config)?,
        Commands::Run {
            dry_run,
            mock_tws,
            no_snapshot,
            snapshot_path,
            log_level,
            verbose,
        } => run_run(
            &cli.config,
            RunOpts {
                dry_run,
                mock_tws,
                no_snapshot,
                snapshot_path,
                log_level,
                verbose,
            },
        )?,
        Commands::Snapshot {
            snapshot_path,
            no_snapshot,
        } => run_snapshot(&cli.config, &snapshot_path, no_snapshot)?,
        Commands::Benchmarks { json } => run_benchmarks(json)?,
        Commands::YieldCurve {
            symbol,
            json,
            refresh,
            source,
            publish_to_nats,
        } => run_yield_curve(&symbol, json, refresh, &source, publish_to_nats)?,
        Commands::SnapshotWrite { json } => run_snapshot_write(json)?,
        Commands::Loans { sub } => match sub {
            LoansCmd::List { json } => run_loans_list(json)?,
        },
    }

    Ok(())
}

fn run_init_config(path: &PathBuf) -> Result<()> {
    if path.extension().map(|e| e == "json").unwrap_or(false) {
        api::write_example_shared_config(path)?;
        println!("Sample shared JSON config written to: {}", path.display());
    } else {
        let cfg = config::Config::default();
        config::save_sample_config(path, &cfg)?;
        println!("Sample TOML configuration written to: {}", path.display());
    }
    println!("Update the values before running live.");
    Ok(())
}

fn run_validate(config_path: &PathBuf) -> Result<()> {
    // Validate shared JSON config first (same discovery and rules as TUI)
    if let Ok(Some(ref loaded)) = api::load_shared_config() {
        if let Err(errors) = api::validate_shared_config(loaded) {
            for e in &errors {
                eprintln!("error: {}", e);
            }
            std::process::exit(1);
        }
    }
    let cfg = config::load_config(config_path)?;
    config::validate_config(&cfg)?;
    println!("Configuration validation successful!");
    Ok(())
}

struct RunOpts {
    dry_run: bool,
    mock_tws: bool,
    no_snapshot: bool,
    snapshot_path: PathBuf,
    log_level: Option<LogLevel>,
    verbose: bool,
}

fn run_run(config_path: &PathBuf, opts: RunOpts) -> Result<()> {
    let mut cfg = config::load_config(config_path)?;

    if opts.dry_run {
        cfg.dry_run = true;
    }
    if opts.mock_tws {
        cfg.tws.use_mock = true;
    }
    if opts.no_snapshot {
        cfg.snapshot.enabled = false;
    } else {
        cfg.snapshot.path = opts.snapshot_path.to_string_lossy().to_string();
    }
    if opts.verbose {
        cfg.logging.level = "debug".to_string();
    }
    if let Some(level) = opts.log_level {
        cfg.logging.level = match level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
        .to_string();
    }

    config::validate_config(&cfg)?;

    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cfg.logging.level));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    print_banner();

    if cfg.dry_run {
        warn!("DRY RUN MODE - No real trades will be executed");
    } else {
        warn!("LIVE TRADING MODE - Real money at risk!");
        warn!("Press Ctrl+C to stop safely");
    }

    info!("Initializing components...");
    trading_loop(&cfg)?;

    Ok(())
}

fn run_snapshot(config_path: &PathBuf, snapshot_path: &PathBuf, no_snapshot: bool) -> Result<()> {
    let cfg = config::load_config(config_path)?;
    config::validate_config(&cfg)?;

    let enabled = cfg.snapshot.enabled && !no_snapshot;
    let path = if no_snapshot {
        snapshot_path.clone()
    } else {
        PathBuf::from(&cfg.snapshot.path)
    };

    println!("Snapshot enabled: {}", enabled);
    println!("Snapshot path: {}", path.display());
    Ok(())
}

fn run_benchmarks(json: bool) -> Result<()> {
    if std::env::var("FRED_API_KEY")
        .ok()
        .map_or(true, |v| v.trim().is_empty())
    {
        eprintln!("warning: FRED_API_KEY not set; SOFR/Treasury data may be empty. See docs/platform/KEYS_FROM_1PASSWORD.md");
    }
    let client = ReqwestClient::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| ReqwestClient::new());

    let response: BenchmarksResponse = tokio::runtime::Runtime::new()?.block_on(async {
        let sofr = get_sofr_rates(&client).await;
        let treasury = get_treasury_rates(&client).await;
        BenchmarksResponse {
            sofr,
            treasury,
            timestamp: Utc::now().to_rfc3339(),
        }
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("SOFR overnight: {:?}", response.sofr.overnight.rate);
        println!("SOFR term rates: {} tenors", response.sofr.term_rates.len());
        for r in &response.sofr.term_rates {
            println!("  {}: {:.4}%", r.tenor, r.rate);
        }
        println!("Treasury: {} rates", response.treasury.rates.len());
        for r in &response.treasury.rates {
            println!("  {}: {:.4}%", r.tenor, r.rate);
        }
    }
    Ok(())
}

fn run_yield_curve(
    symbol: &str,
    json: bool,
    refresh: bool,
    source: &str,
    publish_to_nats: bool,
) -> Result<()> {
    let curve: CurveResponse = if source == "tws" {
        // Direct TWS: no NATS, no backend. Build curve from option chain + quotes.
        let opportunities = tokio::runtime::Runtime::new()?
            .block_on(tws_yield_curve::fetch_yield_curve_from_tws(symbol))
            .map_err(|e| anyhow::anyhow!("TWS: {}", e))?;
        let request = CurveRequest::Named {
            opportunities,
            symbol: Some(symbol.to_string()),
        };
        let mut curve =
            build_curve(request, None).map_err(|e| anyhow::anyhow!("build_curve: {}", e))?;
        for p in curve.points.iter_mut() {
            p.data_source = Some("TWS".to_string());
        }
        let spot = curve
            .points
            .first()
            .and_then(|pt| {
                pt.strike_low
                    .and_then(|l| pt.strike_high.map(|h| (l + h) / 2.0))
            })
            .unwrap_or(6000.0);
        curve.underlying_price = Some(spot);
        curve
    } else {
        let nats_url =
            std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
        tokio::runtime::Runtime::new()?.block_on(async {
            let nc = NatsClient::connect(&nats_url)
                .await
                .map_err(|e| anyhow::anyhow!("NATS: {}", e))?;
            if refresh {
                let payload = serde_json::json!({});
                let _: serde_json::Value =
                    request_json_with_retry(&nc, "api.yield_curve.refresh", &payload)
                        .await
                        .map_err(|e| anyhow::anyhow!("yield_curve.refresh: {}", e))?;
            }
            let payload = serde_json::json!({ "opportunities": [], "symbol": symbol });
            let raw: serde_json::Value = request_json_with_retry_timeout(
                &nc,
                "api.finance_rates.build_curve",
                &payload,
                Duration::from_secs(90),
                RetryConfig::default(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("build_curve: {}", e))?;
            if let Some(err) = raw.get("error").and_then(|v| v.as_str()) {
                return Err(anyhow::anyhow!("backend error: {}", err));
            }
            let curve: CurveResponse =
                serde_json::from_value(raw).map_err(|e| anyhow::anyhow!("decode curve: {}", e))?;
            if publish_to_nats {
                let subj = format!("yield_curve.direct.{}", symbol);
                let _ = nc
                    .client()
                    .publish(
                        subj.clone(),
                        serde_json::to_vec(&curve).unwrap_or_default().into(),
                    )
                    .await;
                tracing::info!(subject = %subj, "published curve to NATS");
            }
            Ok(curve)
        })?
    };

    if publish_to_nats && source == "tws" {
        let nats_url =
            std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let curve_json = serde_json::to_vec(&curve).unwrap_or_default();
        if let Ok(nc) = tokio::runtime::Runtime::new()?.block_on(NatsClient::connect(&nats_url)) {
            let subj = format!("yield_curve.direct.{}", symbol);
            let _ = tokio::runtime::Runtime::new()?
                .block_on(nc.client().publish(subj.clone(), curve_json.into()));
            tracing::info!(subject = %subj, "published curve to NATS");
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&curve)?);
        return Ok(());
    }

    if curve.points.is_empty() {
        println!(
            "No yield curve points for {} (backend may have no data in KV yield_curve.{})",
            curve.symbol, curve.symbol
        );
        return Ok(());
    }

    // Table: Symbol, Expiry, DTE, Bucket, Width, Strikes (K_low/K_high), Long %, Short %, Spread (bps), APR %, Source
    let width_note = curve
        .strike_width
        .map(|w| format!(" (strike width {:.0} pts)", w))
        .unwrap_or_default();
    println!("Box spread yield curve: {}{}", curve.symbol, width_note);
    if let Some(price) = curve.underlying_price {
        println!("Underlying price at report time: {:.2}", price);
    }
    println!(
        "{:<8} {:<12} {:>5} {:<14} {:>5} {:>11} {:>7} {:>7} {:>8} {:>7} {:>8}",
        "Symbol",
        "Expiry",
        "DTE",
        "Bucket",
        "Width",
        "Strikes",
        "Long %",
        "Short %",
        "Spread",
        "APR %",
        "Source"
    );
    println!("{}", "-".repeat(99));
    for p in &curve.points {
        let long_pct = p.buy_implied_rate * 100.0;
        let short_pct = p.sell_implied_rate * 100.0;
        let spread_bps = (p.sell_implied_rate - p.buy_implied_rate) * 10_000.0; // bid-ask width (positive bps)
        let strikes = match (p.strike_low, p.strike_high) {
            (Some(a), Some(b)) => format!("{:.0}/{:.0}", a, b),
            _ => "—".to_string(),
        };
        let source = p.data_source.as_deref().unwrap_or("—");
        println!(
            "{:<8} {:<12} {:>5} {:<14} {:>5.0} {:>11} {:>7.2} {:>7.2} {:>+8.0} {:>7.2} {:>8}",
            p.symbol,
            p.expiry,
            p.days_to_expiry,
            bucket_label(p.days_to_expiry),
            p.strike_width,
            strikes,
            long_pct,
            short_pct,
            spread_bps,
            p.mid_rate * 100.0,
            source
        );
    }
    Ok(())
}

fn run_snapshot_write(json: bool) -> Result<()> {
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let payload = serde_json::json!({});
    let raw: serde_json::Value = tokio::runtime::Runtime::new()?.block_on(async {
        let nc = NatsClient::connect(&nats_url)
            .await
            .map_err(|e| anyhow::anyhow!("NATS: {}", e))?;
        request_json_with_retry(&nc, "api.snapshot.publish_now", &payload)
            .await
            .map_err(|e| anyhow::anyhow!("publish_now: {}", e))
    })?;
    if let Some(err) = raw.get("error").and_then(|v| v.as_str()) {
        anyhow::bail!("backend error: {}", err);
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&raw)?);
    } else {
        let ok = raw.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        let at = raw
            .get("generated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("—");
        let sub = raw.get("subject").and_then(|v| v.as_str()).unwrap_or("—");
        if ok {
            println!("Snapshot written at {} to {}", at, sub);
        } else {
            println!("ok: {}, generated_at: {}, subject: {}", ok, at, sub);
        }
    }
    Ok(())
}

fn run_loans_list(json: bool) -> Result<()> {
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let result: Result<Vec<LoanRecord>, String> = tokio::runtime::Runtime::new()?.block_on(async {
        let nc = NatsClient::connect(&nats_url)
            .await
            .map_err(|e| e.to_string())?;
        request_json_with_retry::<(), Result<Vec<LoanRecord>, String>>(&nc, "api.loans.list", &())
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r)
    });
    match result {
        Ok(list) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else {
                println!(
                    "{:<14} {:<14} {:>6} {:>8} {:>10} {:>10}",
                    "ID", "Bank", "Type", "Principal", "Rate %", "Status"
                );
                println!("{}", "-".repeat(72));
                for l in &list {
                    let typ = match &l.loan_type {
                        api::loans::LoanType::ShirBased => "SHIR",
                        api::loans::LoanType::CpiLinked => "CPI",
                    };
                    let status = match &l.status {
                        api::loans::LoanStatus::Active => "Active",
                        api::loans::LoanStatus::PaidOff => "PaidOff",
                        api::loans::LoanStatus::Defaulted => "Default",
                    };
                    println!(
                        "{:<14} {:<14} {:>6} {:>8.0} {:>10.2} {:>10}",
                        l.loan_id, l.bank_name, typ, l.principal, l.interest_rate, status
                    );
                }
            }
        }
        Err(e) => anyhow::bail!("api.loans.list: {}", e),
    }
    Ok(())
}

/// Expiry bucket label (aligned with TUI expiry_buckets / boxtrades.com).
fn bucket_label(days_to_expiry: i32) -> &'static str {
    match days_to_expiry {
        d if d <= 0 => "expired",
        d if d <= 7 => "5 days",
        d if d <= 25 => "about 1 month",
        d if d <= 45 => "2 months",
        d if d <= 75 => "3 months",
        d if d <= 105 => "4 months",
        d if d <= 135 => "5 months",
        d if d <= 165 => "6 months",
        d if d <= 200 => "7 months",
        d if d <= 235 => "8 months",
        d if d <= 270 => "9 months",
        d if d <= 320 => "10 months",
        d if d <= 350 => "11 months",
        d if d <= 380 => "about 1 year",
        d if d <= 730 => "over 1 year",
        d if d <= 1095 => "almost 2 years",
        d if d <= 1460 => "almost 3 years",
        d if d <= 1825 => "almost 4 years",
        d if d <= 2190 => "almost 5 years",
        _ => "almost 6 years",
    }
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
