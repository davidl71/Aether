//! Aether CLI - Command-line interface for the Aether financing platform
//!
//! Usage:
//!   cargo run -p cli -- init-config config/config.json
//!   cargo run -p cli -- validate
//!   cargo run -p cli -- run --dry-run
//!   cargo run -p cli -- snapshot
//!
//! Output formats:
//!   --format table (default) - human-readable output
//!   --format json - machine-readable JSON for AI agents and scripting
//!
//! Backward compatibility: --init-config and --validate flags still work.

use std::path::PathBuf;

use anyhow::{Context, Result};
use api::finance_rates::{
    build_curve, get_sofr_rates, get_treasury_rates, BenchmarksResponse, CurveRequest,
    CurveResponse,
};
use api::loans::LoanRecord;
use chrono::Utc;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use market_data::{BoxSpreadResult, OptionsDataSource};
use nats_adapter::{
    request_json_with_retry, request_json_with_retry_timeout, NatsClient, RetryConfig,
};
use reqwest::Client as ReqwestClient;
use serde::Serialize;
use std::time::Duration;
use tracing::{info, warn};

mod config;

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

impl OutputFormat {
    pub fn print_json<T: Serialize>(&self, value: &T) -> Result<()> {
        if *self == OutputFormat::Json {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
        Ok(())
    }
}

async fn run_box(symbol: String, dte: f64, width: f64, json: bool) -> Result<()> {
    use market_data::yahoo::{YahooHistorySource, YahooOptionsSource};
    use yfinance_rs::{core::conversions::money_to_f64, Interval, Range};

    println!(
        "Calculating box spread for {} (DTE: {}, width: ${})",
        symbol, dte as i32, width
    );

    let options = YahooOptionsSource::new();
    let history = YahooHistorySource::new();
    let yahoo_symbol = symbol.to_uppercase();

    // Get underlying price
    let candles = history
        .get_history(&yahoo_symbol, Range::D1, Interval::D1)
        .await?;
    let spot = candles
        .first()
        .map(|c| money_to_f64(&c.close))
        .unwrap_or(0.0);
    if spot <= 0.0 {
        return Err(anyhow::anyhow!(
            "Could not determine underlying price for {}",
            symbol
        ));
    }
    println!("Underlying: ${:.2}", spot);

    // Get expirations and find the one closest to requested DTE
    let expirations = options.get_expirations(&yahoo_symbol).await?;
    let today = chrono::Utc::now().date_naive();

    let target_date = today + chrono::Duration::days(dte as i64);
    let (expiration_ts, expiration_date) = expirations
        .into_iter()
        .filter_map(|ts| chrono::DateTime::from_timestamp(ts, 0).map(|dt| (ts, dt.date_naive())))
        .min_by_key(|(_, date)| (*date - target_date).abs())
        .ok_or_else(|| anyhow::anyhow!("No valid expirations found for {}", symbol))?;

    let actual_dte = (expiration_date - today).num_days() as f64;
    println!(
        "Using expiration: {} (DTE: {})",
        expiration_date, actual_dte as i32
    );

    // Get option chain
    let chain = options.get_chain(&yahoo_symbol, expiration_ts).await?;
    println!("Got {} calls, {} puts", chain.calls.len(), chain.puts.len());

    // Find box legs (ITM call, OTM call, ITM put, OTM put)
    let strike_low = (spot / width).floor() * width;
    let strike_high = strike_low + width;

    let c_low = chain
        .calls
        .iter()
        .find(|o| (o.strike - strike_low).abs() < 0.01 && o.bid > 0.0 && o.ask > 0.0)
        .ok_or_else(|| anyhow::anyhow!("Low call not found at strike {}", strike_low))?;
    let c_high = chain
        .calls
        .iter()
        .find(|o| (o.strike - strike_high).abs() < 0.01 && o.bid > 0.0 && o.ask > 0.0)
        .ok_or_else(|| anyhow::anyhow!("High call not found at strike {}", strike_high))?;
    let p_low = chain
        .puts
        .iter()
        .find(|o| (o.strike - strike_low).abs() < 0.01 && o.bid > 0.0 && o.ask > 0.0)
        .ok_or_else(|| anyhow::anyhow!("Low put not found at strike {}", strike_low))?;
    let p_high = chain
        .puts
        .iter()
        .find(|o| (o.strike - strike_high).abs() < 0.01 && o.bid > 0.0 && o.ask > 0.0)
        .ok_or_else(|| anyhow::anyhow!("High put not found at strike {}", strike_high))?;

    println!(
        "Legs: C${:.0} ${:.2}/${:.2} | C${:.0} ${:.2}/${:.2} | P${:.0} ${:.2}/${:.2} | P${:.0} ${:.2}/${:.2}",
        c_low.strike, c_low.bid, c_low.ask,
        c_high.strike, c_high.bid, c_high.ask,
        p_low.strike, p_low.bid, p_low.ask,
        p_high.strike, p_high.bid, p_high.ask
    );

    // Calculate box spread
    let result = BoxSpreadResult::from_quotes(
        (c_low.bid, c_low.ask),
        (c_high.bid, c_high.ask),
        (p_low.bid, p_low.ask),
        (p_high.bid, p_high.ask),
        width,
        actual_dte,
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to calculate box spread"))?;

    if json {
        println!("{{");
        println!("  \"symbol\": \"{}\",", symbol);
        println!("  \"dte\": {},", actual_dte as i32);
        println!("  \"width\": {},", width);
        println!("  \"strike_low\": {},", strike_low);
        println!("  \"strike_high\": {},", strike_high);
        println!("  \"buy_rate\": {:.4},", result.buy_rate * 100.0);
        println!("  \"sell_rate\": {:.4},", result.sell_rate * 100.0);
        println!("  \"mid_rate\": {:.4},", result.mid_rate * 100.0);
        println!("  \"net_debit\": {:.2},", result.net_debit);
        println!("  \"net_credit\": {:.2},", result.net_credit);
        println!("}}");
    } else {
        println!();
        println!("=== Box Spread Result ===");
        println!("Symbol:   {}", symbol);
        println!("DTE:      {}", actual_dte as i32);
        println!("Width:    ${}", width);
        println!("Strikes:  ${:.0} / ${:.0}", strike_low, strike_high);
        println!();
        println!("Buy rate:  {:.2}% (annualized)", result.buy_rate * 100.0);
        println!("Sell rate: {:.2}% (annualized)", result.sell_rate * 100.0);
        println!("Mid rate:  {:.2}% (annualized)", result.mid_rate * 100.0);
        println!();
        println!("Net debit:  ${:.2}", result.net_debit);
        println!("Net credit: ${:.2}", result.net_credit);
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "aether")]
#[command(about = "Aether - Multi-asset synthetic financing platform", long_about = None)]
#[command(after_long_help = "Output formats:
  --format table (default) - human-readable output
  --format json            - machine-readable JSON for AI agents and scripting

Examples:
  aether --format json loans list
  aether --format json yield-curve --symbol SPX
  aether validate --format json")]
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

    /// Output format: 'table' for human-readable, 'json' for AI agents and scripting
    #[arg(long, global = true, default_value = "table", value_enum)]
    format: OutputFormat,

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
    /// Load config and run the service loop (read-only; use tui_service for full UI)
    // [execution-disabled: see docs/DATA_EXPLORATION_MODE.md]
    Run {
        /// Dry run mode - read-only simulation (no order execution)
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
        #[arg(long, default_value = "nats", value_parser = ["nats", "tws", "synthetic", "local"])]
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
    /// Discount Bank file parser (Osh Matching fixed-width format).
    DiscountBank {
        #[command(subcommand)]
        sub: DiscountBankCmd,
    },
    /// Manage secure credentials. Uses config file (~/.config/aether/) + keyring fallback.
    Cred {
        #[command(subcommand)]
        sub: CredCmd,
    },
    /// Alpaca API commands - fetch quotes, positions, and account info
    Alpaca {
        #[command(subcommand)]
        sub: AlpacaCmd,
    },
    // /// Calculate box spread synthetic yield for a given symbol/DTE/width
    // Box {
    //     /// Symbol (e.g., SPX, SPY)
    //     #[arg(short, long, default_value = "SPX")]
    //     symbol: String,
    //     /// Days to expiration (e.g., 30, 45, 60)
    //     #[arg(short, long, default_value_t = 30_f64)]
    //     dte: f64,
    //     /// Strike width in dollars (e.g., 25, 50)
    //     #[arg(short, long, default_value_t = 25.0)]
    //     width: f64,
    //     /// Output as JSON
    //     #[arg(long)]
    //     json: bool,
    // },
}

#[derive(Subcommand, Debug)]
enum CredCmd {
    /// Set a credential value (prompts for input if not provided)
    Set {
        /// Credential name: fred, fmp, polygon, alpaca-paper-key, alpaca-paper-secret, alpaca-live-key, alpaca-live-secret, tastytrade-key, tastytrade-account
        name: String,
        /// Value to store (uses a hidden prompt if not provided)
        value: Option<String>,
    },
    /// Get a credential value (shows masked for security)
    Get {
        /// Credential name: fred, fmp, polygon, alpaca-paper-key, alpaca-paper-secret, alpaca-live-key, alpaca-live-secret, tastytrade-key, tastytrade-account
        name: String,
    },
    /// Delete a stored credential
    Delete {
        /// Credential name: fred, fmp, polygon, alpaca-paper-key, alpaca-paper-secret, alpaca-live-key, alpaca-live-secret, tastytrade-key, tastytrade-account
        name: String,
    },
    /// List available credential names
    List,
}

#[derive(Subcommand, Debug)]
enum LoansCmd {
    /// List all loans (NATS api.loans.list).
    List {
        /// Output as JSON (default: table)
        #[arg(long)]
        json: bool,
    },
    /// Export loans (from backend via NATS api.loans.list) to a CSV file.
    ExportCsv {
        /// Output path (use '-' for stdout)
        #[arg(long, default_value = "-")]
        out: PathBuf,
    },
    /// Import loans from a JSON file into the loan database.
    Import {
        /// Path to JSON file (config/loans.json or custom)
        #[arg(default_value = "config/loans.json")]
        path: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Import loans from a CSV file. CSV should have headers: bank_name, account_number, loan_type, principal, interest_rate, spread, origination_date, first_payment_date, num_payments, monthly_payment
    ImportCsv {
        /// Path to CSV file
        path: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Extract loan data from a PDF (e.g., bank statement). Uses regex patterns to find loan details.
    ImportPdf {
        /// Path to PDF file
        path: PathBuf,
        /// Bank name to associate with extracted loans
        #[arg(long, default_value = "Unknown Bank")]
        bank_name: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum DiscountBankCmd {
    /// Parse a Discount Bank Osh Matching file and output ledger transactions.
    Import {
        /// Path to Discount Bank fixed-width file
        path: PathBuf,
        /// Optional exchange rate for ILS → USD conversion
        #[arg(long)]
        exchange_rate: Option<f64>,
        /// Output as JSON (default: human-readable summary)
        #[arg(long)]
        json: bool,
    },

    /// Export latest Discount Bank transactions to a CSV file.
    ExportCsv {
        /// Max number of rows (most recent first)
        #[arg(long, default_value_t = 500)]
        limit: usize,
        /// Output path (use '-' for stdout)
        #[arg(long, default_value = "-")]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum AlpacaCmd {
    /// Fetch latest quote for a symbol
    Quote {
        /// Symbol to look up (e.g., AAPL, SPY)
        symbol: String,
        /// Use paper trading environment (default) or live
        #[arg(long)]
        live: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Fetch account information
    Account {
        /// Use paper trading environment (default) or live
        #[arg(long)]
        live: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List open positions
    Positions {
        /// Use paper trading environment (default) or live
        #[arg(long)]
        live: bool,
        /// Output as JSON
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

    let format = cli.format;

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
        Commands::Benchmarks { json } => run_benchmarks(format, json)?,
        Commands::YieldCurve {
            symbol,
            json,
            refresh,
            source,
            publish_to_nats,
        } => run_yield_curve(&symbol, format, json, refresh, &source, publish_to_nats)?,
        Commands::SnapshotWrite { json } => run_snapshot_write(format, json)?,
        Commands::Loans { sub } => match sub {
            LoansCmd::List { json } => run_loans_list(format, json)?,
            LoansCmd::ExportCsv { out } => run_loans_export_csv(&out)?,
            LoansCmd::Import { path, json } => run_loans_import(&path, format, json)?,
            LoansCmd::ImportCsv { path, json } => run_loans_import_csv(&path, format, json)?,
            LoansCmd::ImportPdf {
                path,
                bank_name,
                json,
            } => run_loans_import_pdf(&path, &bank_name, format, json)?,
        },
        Commands::DiscountBank { sub } => match sub {
            DiscountBankCmd::Import {
                path,
                exchange_rate,
                json,
            } => run_discount_bank_import(&path, exchange_rate, format, json)?,
            DiscountBankCmd::ExportCsv { limit, out } => run_discount_bank_export_csv(limit, &out)?,
        },
        Commands::Cred { sub } => run_cred(sub)?,
        Commands::Alpaca { sub } => tokio::runtime::Runtime::new()?.block_on(run_alpaca(sub))?,
        // Commands::Box { symbol, dte, width, json } => {
        //     tokio::runtime::Runtime::new()?.block_on(run_box(symbol, dte, width, json))?
        // }
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
        warn!("DRY RUN MODE - read-only exploration, no execution");
    } else {
        // [execution-disabled: see docs/DATA_EXPLORATION_MODE.md]
        warn!("READ-ONLY MODE - execution paths are disabled");
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

fn run_benchmarks(format: OutputFormat, json: bool) -> Result<()> {
    if api::credentials::fred_api_key().is_none() {
        eprintln!("warning: FRED_API_KEY not set; SOFR/Treasury data may be empty. Run 'just cred-set-fred' to configure.");
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
            shir: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    });

    let use_json = json || format == OutputFormat::Json;
    if use_json {
        format.print_json(&response)?;
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
    format: OutputFormat,
    json: bool,
    refresh: bool,
    source: &str,
    publish_to_nats: bool,
) -> Result<()> {
    let curve: CurveResponse = if source == "synthetic" || source == "local" {
        // Local synthetic: no NATS, no TWS. Use built-in synthetic data.
        api::finance_rates::build_synthetic_curve(symbol, None)
    } else if source == "tws" {
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

    let use_json = json || format == OutputFormat::Json;
    if use_json {
        format.print_json(&curve)?;
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

fn run_snapshot_write(format: OutputFormat, json: bool) -> Result<()> {
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
    let use_json = json || format == OutputFormat::Json;
    if use_json {
        format.print_json(&raw)?;
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

fn run_cred(sub: CredCmd) -> Result<()> {
    use api::credentials::{self, CredentialKey};

    fn parse_key(name: &str) -> Result<CredentialKey, String> {
        CredentialKey::from_name(name).ok_or_else(|| {
            format!(
                "Unknown credential '{}'. Use: {}",
                name,
                list_credential_names()
            )
        })
    }

    fn list_credential_names() -> String {
        let creds = credentials::list_credentials();
        creds.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
    }

    fn mask_value(v: &str) -> String {
        if v.len() > 8 {
            format!("{}...{}", &v[..4], &v[v.len() - 4..])
        } else {
            v.to_string()
        }
    }

    match sub {
        CredCmd::Set { name, value } => {
            let key = parse_key(&name).map_err(|e| anyhow::anyhow!("{}", e))?;
            let display_name = key.display_name();

            let value = match value {
                Some(v) => v,
                None => rpassword::prompt_password(format!("Enter {}: ", display_name))
                    .map(|input| input.trim().to_string())
                    .map_err(|e| anyhow::anyhow!("failed to read credential: {}", e))?,
            };

            if value.is_empty() {
                anyhow::bail!("Value cannot be empty");
            }

            credentials::set_credential(key, &value).map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("{} stored securely", display_name);
        }
        CredCmd::Get { name } => {
            let key = parse_key(&name).map_err(|e| anyhow::anyhow!("{}", e))?;
            let display_name = key.display_name();

            match credentials::get_credential(key) {
                Some(v) => {
                    println!("{}: {}", display_name, mask_value(&v));
                }
                None => println!(
                    "No {} found (set via 'just cred-set {} <value>')",
                    display_name, name
                ),
            }
        }
        CredCmd::Delete { name } => {
            let key = parse_key(&name).map_err(|e| anyhow::anyhow!("{}", e))?;
            let display_name = key.display_name();

            match credentials::delete_credential(key) {
                Ok(()) => println!("{} deleted", display_name),
                Err(e) => println!("Note: {}", e),
            }
        }
        CredCmd::List => {
            println!("Available credentials:");
            let creds = credentials::list_credentials();
            for (key_str, desc) in &creds {
                let test_key =
                    CredentialKey::from_name(key_str).unwrap_or(CredentialKey::FredApiKey);
                let status = if credentials::get_credential(test_key).is_some() {
                    "✓"
                } else {
                    "○"
                };
                println!("  {:20} {}  {}", key_str, status, desc);
            }
            println!("\nCommands:");
            println!("  just cred-set <name> <value>  # Store credential");
            println!("  just cred-get <name>          # Show (masked)");
            println!("  just cred-delete <name>       # Remove");
        }
    }
    Ok(())
}

fn run_loans_list(format: OutputFormat, json: bool) -> Result<()> {
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
            let use_json = json || format == OutputFormat::Json;
            if use_json {
                format.print_json(&list)?;
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

fn run_loans_export_csv(out: &PathBuf) -> Result<()> {
    #[derive(serde::Serialize)]
    struct LoanCsvRow {
        loan_id: String,
        bank_name: String,
        account_number: String,
        loan_type: String,
        principal: f64,
        original_principal: f64,
        interest_rate: f64,
        spread: f64,
        base_cpi: f64,
        current_cpi: f64,
        origination_date: String,
        maturity_date: String,
        next_payment_date: String,
        monthly_payment: f64,
        payment_frequency_months: i32,
        status: String,
        last_update: String,
        currency: String,
    }

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let loans: Vec<LoanRecord> = tokio::runtime::Runtime::new()?.block_on(async {
        let nc = NatsClient::connect(&nats_url)
            .await
            .map_err(|e| anyhow::anyhow!("NATS: {e}"))?;

        let res: Result<Vec<LoanRecord>, String> =
            request_json_with_retry::<(), Result<Vec<LoanRecord>, String>>(&nc, "api.loans.list", &())
                .await
                .map_err(|e| anyhow::anyhow!("api.loans.list: {e}"))?;

        res.map_err(|e| anyhow::anyhow!("api.loans.list: {e}"))
    })?;

    let rows = loans.into_iter().map(|l| LoanCsvRow {
        loan_id: l.loan_id,
        bank_name: l.bank_name,
        account_number: l.account_number,
        loan_type: match l.loan_type {
            api::loans::LoanType::ShirBased => "SHIR".to_string(),
            api::loans::LoanType::CpiLinked => "CPI".to_string(),
        },
        principal: l.principal,
        original_principal: l.original_principal,
        interest_rate: l.interest_rate,
        spread: l.spread,
        base_cpi: l.base_cpi,
        current_cpi: l.current_cpi,
        origination_date: l.origination_date,
        maturity_date: l.maturity_date,
        next_payment_date: l.next_payment_date,
        monthly_payment: l.monthly_payment,
        payment_frequency_months: l.payment_frequency_months,
        status: match l.status {
            api::loans::LoanStatus::Active => "ACTIVE".to_string(),
            api::loans::LoanStatus::PaidOff => "PAID_OFF".to_string(),
            api::loans::LoanStatus::Defaulted => "DEFAULTED".to_string(),
        },
        last_update: l.last_update,
        currency: l.currency,
    });

    write_csv_rows(out, rows)
}

fn write_csv_rows<I, T>(out: &PathBuf, rows: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Serialize,
{
    if out.as_os_str() == "-" {
        let stdout = std::io::stdout();
        let handle = stdout.lock();
        let mut wtr = csv::WriterBuilder::new().has_headers(true).from_writer(handle);
        for row in rows {
            wtr.serialize(row)?;
        }
        wtr.flush()?;
        return Ok(());
    }

    let file = std::fs::File::create(out)?;
    let mut wtr = csv::WriterBuilder::new().has_headers(true).from_writer(file);
    for row in rows {
        wtr.serialize(row)?;
    }
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod csv_export_tests {
    use super::*;

    #[test]
    fn write_csv_rows_writes_headers_and_rows() {
        #[derive(serde::Serialize)]
        struct Row {
            a: i32,
            b: String,
        }

        let out = std::env::temp_dir().join(format!(
            "aether_cli_write_csv_rows_test_{}.csv",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&out);

        write_csv_rows(
            &out,
            vec![Row {
                a: 1,
                b: "x".to_string(),
            }],
        )
        .expect("write_csv_rows should succeed");

        let text = std::fs::read_to_string(&out).expect("output file should exist");
        assert!(
            text.starts_with("a,b\n1,x\n") || text.starts_with("a,b\r\n1,x\r\n"),
            "unexpected CSV output: {text:?}"
        );

        let _ = std::fs::remove_file(&out);
    }
}

fn run_loans_import(path: &PathBuf, format: OutputFormat, json: bool) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Loans file not found: {}", path.display());
    }

    let text = std::fs::read_to_string(path)?;

    #[derive(serde::Deserialize)]
    struct LoanFile {
        loans: Vec<api::loans::LoanRecord>,
    }

    let file: LoanFile = serde_json::from_str(&text).context("failed to parse loans JSON")?;

    let repo =
        tokio::runtime::Runtime::new()?.block_on(api::loans::LoanRepository::load(path.clone()))?;

    let mut imported = 0;
    let mut errors = Vec::new();

    for loan in &file.loans {
        match loan.validate() {
            Ok(()) => match tokio::runtime::Runtime::new()?.block_on(repo.create(loan.clone())) {
                Ok(()) => imported += 1,
                Err(e) => errors.push(format!("{}: {}", loan.loan_id, e)),
            },
            Err(errs) => {
                errors.push(format!(
                    "{}: validation failed - {}",
                    loan.loan_id,
                    errs.join("; ")
                ));
            }
        }
    }

    let use_json = json || format == OutputFormat::Json;
    if use_json {
        #[derive(serde::Serialize)]
        struct ImportResult {
            total: usize,
            imported: usize,
            errors: Vec<String>,
        }
        format.print_json(&ImportResult {
            total: file.loans.len(),
            imported,
            errors,
        })?;
    } else {
        println!("Loans import from {}", path.display());
        println!(
            "Total: {}, Imported: {}, Errors: {}",
            file.loans.len(),
            imported,
            errors.len()
        );
        if !errors.is_empty() {
            println!("\nErrors:");
            for e in &errors {
                println!("  - {}", e);
            }
        }
    }

    Ok(())
}

fn run_loans_import_csv(path: &PathBuf, format: OutputFormat, json: bool) -> Result<()> {
    use std::io::BufRead;

    if !path.exists() {
        anyhow::bail!("CSV file not found: {}", path.display());
    }

    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    let headers = match lines.next() {
        Some(Ok(line)) => line
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect::<Vec<_>>(),
        Some(Err(e)) => anyhow::bail!("Failed to read CSV header: {}", e),
        None => anyhow::bail!("Empty CSV file"),
    };

    // Expected columns
    let col =
        |name: &str| -> usize { headers.iter().position(|h| h == name).unwrap_or(usize::MAX) };
    let idx_bank = col("bank_name");
    let idx_account = col("account_number");
    let idx_type = col("loan_type");
    let idx_principal = col("principal");
    let idx_rate = col("interest_rate");
    let idx_spread = col("spread");
    let idx_start = col("origination_date");
    let idx_first = col("first_payment_date");
    let idx_monthly = col("monthly_payment");

    let mut loans: Vec<LoanRecord> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (line_num, line) in lines.enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                errors.push(format!("line {}: read error - {}", line_num + 2, e));
                continue;
            }
        };

        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let get = |idx: usize| -> &str { fields.get(idx).unwrap_or(&"").trim_matches('"') };

        let bank_name = get(idx_bank).to_string();
        let account_number = get(idx_account).to_string();
        let loan_type_str = get(idx_type).to_uppercase();
        let loan_type = match loan_type_str.as_str() {
            "SHIR" | "SHIR_BASED" => api::loans::LoanType::ShirBased,
            "CPI" | "CPI_LINKED" => api::loans::LoanType::CpiLinked,
            _ => {
                errors.push(format!(
                    "line {}: unknown loan type '{}'",
                    line_num + 2,
                    loan_type_str
                ));
                continue;
            }
        };
        let is_cpi = matches!(loan_type, api::loans::LoanType::CpiLinked);

        let principal: f64 = match get(idx_principal).parse() {
            Ok(v) => v,
            Err(_) => {
                errors.push(format!(
                    "line {}: invalid principal '{}'",
                    line_num + 2,
                    get(idx_principal)
                ));
                continue;
            }
        };
        let interest_rate: f64 = match get(idx_rate).parse() {
            Ok(v) => v,
            Err(_) => {
                errors.push(format!(
                    "line {}: invalid interest rate '{}'",
                    line_num + 2,
                    get(idx_rate)
                ));
                continue;
            }
        };
        let spread: f64 = get(idx_spread).parse().unwrap_or(0.0);

        let now = chrono::Utc::now();
        let last_update = now.to_rfc3339();

        let loan_id = format!(
            "loan-{}-{}-{}",
            bank_name.to_lowercase().replace(' ', "-"),
            account_number,
            now.timestamp()
        );

        loans.push(LoanRecord {
            loan_id,
            bank_name,
            account_number,
            loan_type,
            principal,
            original_principal: principal,
            interest_rate,
            spread,
            base_cpi: if is_cpi { 250.0 } else { 0.0 },
            current_cpi: if is_cpi { 255.0 } else { 0.0 },
            origination_date: get(idx_start).to_string(),
            maturity_date: String::new(), // Will be calculated
            next_payment_date: get(idx_first).to_string(),
            monthly_payment: get(idx_monthly).parse().unwrap_or(0.0),
            payment_frequency_months: 1,
            status: api::loans::LoanStatus::Active,
            last_update,
            currency: "ILS".to_string(),
        });
    }

    // Calculate maturity dates
    for loan in &mut loans {
        if !loan.next_payment_date.is_empty() {
            if let Ok(base) = chrono::NaiveDate::parse_from_str(&loan.next_payment_date, "%Y-%m-%d")
            {
                let days_to_add = (loan.payment_frequency_months as i64 * 30) - 1;
                let maturity = base + chrono::Duration::days(days_to_add);
                loan.maturity_date = maturity.format("%Y-%m-%d").to_string();
            }
        }
    }

    // Insert into database
    let repo =
        tokio::runtime::Runtime::new()?.block_on(api::loans::LoanRepository::load(path.clone()))?;

    let mut imported = 0;
    for loan in &loans {
        if let Err(e) = tokio::runtime::Runtime::new()?.block_on(repo.create(loan.clone())) {
            errors.push(format!("{}: {}", loan.loan_id, e));
        } else {
            imported += 1;
        }
    }

    let use_json = json || format == OutputFormat::Json;
    if use_json {
        #[derive(serde::Serialize)]
        struct ImportResult {
            total: usize,
            imported: usize,
            errors: Vec<String>,
        }
        format.print_json(&ImportResult {
            total: loans.len(),
            imported,
            errors,
        })?;
    } else {
        println!("Loans CSV import from {}", path.display());
        println!(
            "Total: {}, Imported: {}, Errors: {}",
            loans.len(),
            imported,
            errors.len()
        );
        if !errors.is_empty() {
            println!("\nErrors:");
            for e in &errors {
                println!("  - {}", e);
            }
        }
    }

    Ok(())
}

fn run_loans_import_pdf(
    path: &PathBuf,
    bank_name: &str,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    use pdf_extract::extract_text;

    if !path.exists() {
        anyhow::bail!("PDF file not found: {}", path.display());
    }

    println!("Extracting text from PDF...");
    let text = match extract_text(path) {
        Ok(t) => t,
        Err(e) => anyhow::bail!("Failed to extract text from PDF: {}. Make sure poppler-utils is installed (apt-get install poppler-utils)", e),
    };

    println!("PDF text length: {} characters", text.len());

    // Regex patterns for common loan data extraction
    let principal_re = regex::Regex::new(
        r"(?i)(principal|loan amount|balance)[\s:]*[\$₪]?\s*([\d,]+(?:\.\d{2})?)",
    )
    .unwrap();
    let rate_re = regex::Regex::new(r"(?i)(interest rate|rate)[\s:]*(\d+(?:\.\d+)?)\s*%?").unwrap();
    let account_re = regex::Regex::new(r"(?i)(account|loan number)[\s:]*([A-Z0-9-]+)").unwrap();
    let date_re =
        regex::Regex::new(r"(?i)(date|start)[\s:]*(\d{1,2}[/-]\d{1,2}[/-]\d{2,4})").unwrap();

    let mut extracted_loans: Vec<LoanRecord> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Try to find loan blocks (sections between headers or sections with loan identifiers)
    let lines: Vec<&str> = text.lines().collect();
    let mut current_block = String::new();
    let mut block_start = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Detect new loan block (starts with "Loan", "Account", "Credit", etc.)
        let is_new_block = regex::Regex::new(r"(?i)^(loan|account|credit|הלוואה|משכנתא)")
            .unwrap()
            .is_match(trimmed);

        if is_new_block && !current_block.is_empty() {
            // Process the previous block
            let block_text = current_block.clone();

            let principal = principal_re
                .captures(&block_text)
                .and_then(|c| c.get(2))
                .and_then(|m| m.as_str().replace(',', "").parse::<f64>().ok());

            let rate = rate_re
                .captures(&block_text)
                .and_then(|c| c.get(2))
                .and_then(|m| m.as_str().parse::<f64>().ok());

            let account = account_re
                .captures(&block_text)
                .and_then(|c| c.get(2))
                .map(|m| m.as_str().to_string());

            let date = date_re
                .captures(&block_text)
                .and_then(|c| c.get(2))
                .map(|m| m.as_str().to_string());

            if let (Some(p), Some(r)) = (principal, rate) {
                let now = chrono::Utc::now();
                let loan_id = format!(
                    "loan-{}-{}-{}",
                    bank_name.to_lowercase().replace(' ', "-"),
                    account.as_deref().unwrap_or("unknown"),
                    now.timestamp()
                );

                let maturity = date.as_ref().and_then(|d| {
                    chrono::NaiveDate::parse_from_str(d, "%d/%m/%Y")
                        .ok()
                        .or_else(|| chrono::NaiveDate::parse_from_str(d, "%m/%d/%Y").ok())
                        .map(|base| base + chrono::Duration::days(30 * 12 * 20 - 1))
                    // Default 20 years
                });

                extracted_loans.push(LoanRecord {
                    loan_id,
                    bank_name: bank_name.to_string(),
                    account_number: account.unwrap_or_else(|| "unknown".to_string()),
                    loan_type: api::loans::LoanType::ShirBased,
                    principal: p,
                    original_principal: p,
                    interest_rate: r,
                    spread: 0.0,
                    base_cpi: 0.0,
                    current_cpi: 0.0,
                    origination_date: date
                        .clone()
                        .unwrap_or_else(|| now.format("%Y-%m-%d").to_string()),
                    maturity_date: maturity
                        .map(|m| m.format("%Y-%m-%d").to_string())
                        .unwrap_or_default(),
                    next_payment_date: now.format("%Y-%m-%d").to_string(),
                    monthly_payment: 0.0, // Would need more parsing
                    payment_frequency_months: 1,
                    status: api::loans::LoanStatus::Active,
                    last_update: now.to_rfc3339(),
                    currency: "ILS".to_string(),
                });
            } else {
                if principal.is_none() && rate.is_none() {
                    errors.push(format!("Block at line {}: no loan data found", block_start));
                } else {
                    let missing = if principal.is_none() {
                        "principal"
                    } else {
                        "interest rate"
                    };
                    errors.push(format!(
                        "Block at line {}: missing {}",
                        block_start, missing
                    ));
                }
            }

            current_block.clear();
            block_start = i;
        }

        current_block.push_str(line);
        current_block.push('\n');
    }

    // Process last block
    if !current_block.is_empty() {
        let block_text = current_block;

        let principal = principal_re
            .captures(&block_text)
            .and_then(|c| c.get(2))
            .and_then(|m| m.as_str().replace(',', "").parse::<f64>().ok());

        let rate = rate_re
            .captures(&block_text)
            .and_then(|c| c.get(2))
            .and_then(|m| m.as_str().parse::<f64>().ok());

        let account = account_re
            .captures(&block_text)
            .and_then(|c| c.get(2))
            .map(|m| m.as_str().to_string());

        let date = date_re
            .captures(&block_text)
            .and_then(|c| c.get(2))
            .map(|m| m.as_str().to_string());

        if let (Some(p), Some(r)) = (principal, rate) {
            let now = chrono::Utc::now();
            let loan_id = format!(
                "loan-{}-{}-{}",
                bank_name.to_lowercase().replace(' ', "-"),
                account.as_deref().unwrap_or("unknown"),
                now.timestamp()
            );

            let maturity = date.as_ref().and_then(|d| {
                chrono::NaiveDate::parse_from_str(d, "%d/%m/%Y")
                    .ok()
                    .or_else(|| chrono::NaiveDate::parse_from_str(d, "%m/%d/%Y").ok())
                    .map(|base| base + chrono::Duration::days(30 * 12 * 20 - 1))
            });

            extracted_loans.push(LoanRecord {
                loan_id,
                bank_name: bank_name.to_string(),
                account_number: account.unwrap_or_else(|| "unknown".to_string()),
                loan_type: api::loans::LoanType::ShirBased,
                principal: p,
                original_principal: p,
                interest_rate: r,
                spread: 0.0,
                base_cpi: 0.0,
                current_cpi: 0.0,
                origination_date: date
                    .clone()
                    .unwrap_or_else(|| now.format("%Y-%m-%d").to_string()),
                maturity_date: maturity
                    .map(|m| m.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                next_payment_date: now.format("%Y-%m-%d").to_string(),
                monthly_payment: 0.0,
                payment_frequency_months: 1,
                status: api::loans::LoanStatus::Active,
                last_update: now.to_rfc3339(),
                currency: "ILS".to_string(),
            });
        }
    }

    // Insert into database
    let repo =
        tokio::runtime::Runtime::new()?.block_on(api::loans::LoanRepository::load(path.clone()))?;

    let mut imported = 0;
    for loan in &extracted_loans {
        if let Err(e) = tokio::runtime::Runtime::new()?.block_on(repo.create(loan.clone())) {
            errors.push(format!("{}: {}", loan.loan_id, e));
        } else {
            imported += 1;
        }
    }

    let use_json = json || format == OutputFormat::Json;
    if use_json {
        #[derive(serde::Serialize)]
        struct ImportResult {
            total: usize,
            imported: usize,
            errors: Vec<String>,
        }
        format.print_json(&ImportResult {
            total: extracted_loans.len(),
            imported,
            errors,
        })?;
    } else {
        println!("Loans PDF import from {}", path.display());
        println!(
            "Total extracted: {}, Imported: {}, Errors: {}",
            extracted_loans.len(),
            imported,
            errors.len()
        );
        if !extracted_loans.is_empty() {
            println!("\nExtracted loans:");
            for loan in &extracted_loans {
                println!(
                    "  - {}: {} at {}%",
                    loan.loan_id, loan.principal, loan.interest_rate
                );
            }
        }
        if !errors.is_empty() {
            println!("\nErrors:");
            for e in &errors {
                println!("  - {}", e);
            }
        }
    }

    Ok(())
}

fn run_discount_bank_export_csv(limit: usize, out: &PathBuf) -> Result<()> {
    #[derive(serde::Serialize)]
    struct DiscountBankTxnCsvRow {
        account_id: String,
        institution: String,
        account_number: String,
        branch_number: String,
        section_number: String,
        currency: String,
        value_date: String,
        amount: f64,
        is_debit: bool,
        reference: String,
    }

    let dto = tokio::runtime::Runtime::new()?.block_on(async {
        api::discount_bank::get_transactions(limit)
            .await
            .map_err(|e| anyhow::anyhow!("discount_bank.get_transactions: {e}"))
    })?;

    let acct = dto.account;
    let rows = dto.transactions.into_iter().map(|t| DiscountBankTxnCsvRow {
        account_id: acct.id.clone(),
        institution: acct.institution.clone(),
        account_number: acct.account_number.clone(),
        branch_number: acct.branch_number.clone(),
        section_number: acct.section_number.clone(),
        currency: acct.currency.clone(),
        value_date: t.value_date,
        amount: t.amount,
        is_debit: t.is_debit,
        reference: t.reference,
    });

    write_csv_rows(out, rows)
}

fn run_discount_bank_import(
    path: &PathBuf,
    exchange_rate: Option<f64>,
    format: OutputFormat,
    json: bool,
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Discount Bank file not found: {}", path.display());
    }

    let parsed = tokio::runtime::Runtime::new()?
        .block_on(discount_bank_parser::DiscountBankParser::parse_file(path))?;

    let exchange_rate_decimal = exchange_rate
        .map(rust_decimal::Decimal::try_from)
        .transpose()?;

    let transactions =
        discount_bank_parser::convert_to_transactions(&parsed, exchange_rate_decimal, None)?;

    let use_json = json || format == OutputFormat::Json;
    if use_json {
        format.print_json(&transactions)?;
    } else {
        println!("Discount Bank file: {}", path.display());
        println!("Account: {:?}", parsed.account_number());
        println!("Currency: {:?}", parsed.currency_code());
        println!("Transactions: {}", transactions.len());

        if let Some(summary) = &parsed.summary {
            println!("Summary: {} transactions", summary.transaction_count);
        }

        if !transactions.is_empty() {
            println!("\nLedger transactions would be:");
            for txn in &transactions {
                println!(
                    "  {} - {} ({} postings)",
                    txn.date,
                    txn.description,
                    txn.postings.len()
                );
            }
        }
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
    println!("║              Operator Console (Read-Only)                  ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
}

fn trading_loop(_config: &config::Config) -> Result<()> {
    info!("Trading loop started - use tui_service for full interface");
    info!("Run 'cargo run -p tui_service' for the terminal UI");

    std::thread::park();

    Ok(())
}

async fn run_alpaca(sub: AlpacaCmd) -> Result<()> {
    use api::alpaca_positions::AlpacaPositionSource;
    use api::credentials::{get_credential, CredentialKey};

    match sub {
        AlpacaCmd::Quote { symbol, live, json } => {
            use market_data::alpaca::AlpacaSource;

            let is_paper = !live;
            let (key_id, secret) = if live {
                let key_id = get_credential(CredentialKey::AlpacaLiveApiKeyId)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca live key not found"))?;
                let secret = get_credential(CredentialKey::AlpacaLiveSecretKey)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca live secret not found"))?;
                (key_id, secret)
            } else {
                let key_id = get_credential(CredentialKey::AlpacaPaperApiKeyId)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca paper key not found"))?;
                let secret = get_credential(CredentialKey::AlpacaPaperSecretKey)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca paper secret not found"))?;
                (key_id, secret)
            };

            // Set environment variables for the Alpaca API client
            std::env::set_var("APCA_API_KEY_ID", key_id);
            std::env::set_var("APCA_API_SECRET_KEY", secret);
            let base_url = if live {
                "https://api.alpaca.markets"
            } else {
                "https://paper-api.alpaca.markets"
            };
            std::env::set_var("APCA_API_BASE_URL", base_url);

            let source =
                AlpacaSource::new(is_paper, vec![&symbol], std::time::Duration::from_secs(1))?;

            println!(
                "Fetching quote for {} from Alpaca {}...",
                symbol,
                if live { "LIVE" } else { "PAPER" }
            );
            match source.get_quote_sync(&symbol) {
                Ok(quote) => {
                    if json {
                        println!("{{");
                        println!("  \"symbol\": \"{}\",", quote.symbol);
                        println!("  \"bid_price\": {},", quote.bid_price);
                        println!("  \"ask_price\": {},", quote.ask_price);
                        println!("  \"bid_size\": {},", quote.bid_size);
                        println!("  \"ask_size\": {},", quote.ask_size);
                        println!("  \"timestamp\": \"{}\",", quote.timestamp);
                        println!("  \"source\": \"{}\"", source.source_name());
                        println!("}}");
                    } else {
                        println!("✓ Quote received from {}", source.source_name());
                        println!();
                        println!("Symbol: {}", quote.symbol);
                        println!("  Bid: ${:.2} (size: {})", quote.bid_price, quote.bid_size);
                        println!("  Ask: ${:.2} (size: {})", quote.ask_price, quote.ask_size);
                        println!("  Timestamp: {}", quote.timestamp);
                    }
                }
                Err(e) => {
                    eprintln!("✗ Error fetching quote: {}", e);
                    eprintln!();
                    eprintln!("Note: Alpaca market data limitations:");
                    eprintln!("  - Indices like ^SPX are NOT available");
                    eprintln!("  - Use ETFs like SPY, QQQ, IWM instead");
                    eprintln!("  - Free tier only covers IEX exchange");
                    std::process::exit(1);
                }
            }
        }
        AlpacaCmd::Account { live, json } => {
            let source = if live {
                let key_id = get_credential(CredentialKey::AlpacaLiveApiKeyId)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca live key not found"))?;
                let secret = get_credential(CredentialKey::AlpacaLiveSecretKey)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca live secret not found"))?;
                std::env::set_var("APCA_API_KEY_ID", key_id);
                std::env::set_var("APCA_API_SECRET_KEY", secret);
                std::env::set_var("APCA_API_BASE_URL", "https://api.alpaca.markets");
                AlpacaPositionSource::from_env()
                    .ok_or_else(|| anyhow::anyhow!("Failed to create Alpaca source"))?
            } else {
                let key_id = get_credential(CredentialKey::AlpacaPaperApiKeyId)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca paper key not found"))?;
                let secret = get_credential(CredentialKey::AlpacaPaperSecretKey)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca paper secret not found"))?;
                std::env::set_var("APCA_API_KEY_ID", key_id);
                std::env::set_var("APCA_API_SECRET_KEY", secret);
                std::env::set_var("APCA_API_BASE_URL", "https://paper-api.alpaca.markets");
                AlpacaPositionSource::from_env()
                    .ok_or_else(|| anyhow::anyhow!("Failed to create Alpaca source"))?
            };

            println!("Fetching account info from Alpaca...");
            match source.fetch_account().await {
                Ok(account) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&account)?);
                    } else {
                        println!("Account ID: {}", account.account_id);
                        println!("Cash: ${:.2}", account.cash);
                        println!("Buying Power: ${:.2}", account.buying_power);
                        println!("Equity: ${:.2}", account.equity);
                        println!("Portfolio Value: ${:.2}", account.portfolio_value);
                        println!(
                            "Environment: {}",
                            if source.is_paper() { "Paper" } else { "Live" }
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching account: {}", e);
                    std::process::exit(1);
                }
            }
        }
        AlpacaCmd::Positions { live, json } => {
            let source = if live {
                let key_id = get_credential(CredentialKey::AlpacaLiveApiKeyId)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca live key not found"))?;
                let secret = get_credential(CredentialKey::AlpacaLiveSecretKey)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca live secret not found"))?;
                std::env::set_var("APCA_API_KEY_ID", key_id);
                std::env::set_var("APCA_API_SECRET_KEY", secret);
                std::env::set_var("APCA_API_BASE_URL", "https://api.alpaca.markets");
                AlpacaPositionSource::from_env()
                    .ok_or_else(|| anyhow::anyhow!("Failed to create Alpaca source"))?
            } else {
                let key_id = get_credential(CredentialKey::AlpacaPaperApiKeyId)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca paper key not found"))?;
                let secret = get_credential(CredentialKey::AlpacaPaperSecretKey)
                    .ok_or_else(|| anyhow::anyhow!("Alpaca paper secret not found"))?;
                std::env::set_var("APCA_API_KEY_ID", key_id);
                std::env::set_var("APCA_API_SECRET_KEY", secret);
                std::env::set_var("APCA_API_BASE_URL", "https://paper-api.alpaca.markets");
                AlpacaPositionSource::from_env()
                    .ok_or_else(|| anyhow::anyhow!("Failed to create Alpaca source"))?
            };

            println!("Fetching positions from Alpaca...");
            match source.fetch_positions().await {
                Ok(positions) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&positions)?);
                    } else {
                        if positions.is_empty() {
                            println!("No open positions");
                        } else {
                            println!("Open Positions:");
                            for pos in positions {
                                println!(
                                    "  {}: {} shares @ ${:.2} (market value: ${:.2})",
                                    pos.symbol, pos.quantity, pos.cost_basis, pos.market_value
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching positions: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
