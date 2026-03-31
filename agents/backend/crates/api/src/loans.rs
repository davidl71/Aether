//! Loans API: CRUD and aggregation. Exposed via NATS only (subjects `api.loans.*`).
//! Config was REST-only and is not exposed; see docs/platform/NATS_API.md §3.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use csv::Trim;
use nats_adapter::proto::v1::{Loan as ProtoLoan, LoansResponse};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoanType {
    #[serde(rename = "SHIR_BASED", alias = "SHIR")]
    ShirBased,
    #[serde(rename = "CPI_LINKED")]
    CpiLinked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoanStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "PAID_OFF")]
    PaidOff,
    #[serde(rename = "DEFAULTED")]
    Defaulted,
}

fn default_ils() -> String {
    "ILS".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoanRecord {
    pub loan_id: String,
    pub bank_name: String,
    pub account_number: String,
    pub loan_type: LoanType,
    pub principal: f64,
    pub original_principal: f64,
    pub interest_rate: f64,
    pub spread: f64,
    pub base_cpi: f64,
    pub current_cpi: f64,
    pub origination_date: String,
    pub maturity_date: String,
    pub next_payment_date: String,
    pub monthly_payment: f64,
    pub payment_frequency_months: i32,
    pub status: LoanStatus,
    pub last_update: String,
    #[serde(default = "default_ils")]
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoanAggregationInput {
    pub loan_id: Option<String>,
    pub name: String,
    pub instrument_type: String,
    pub principal: f64,
    pub annual_rate: f64,
    pub monthly_payment: Option<f64>,
    pub maturity_date: Option<String>,
}

impl LoanAggregationInput {
    pub fn is_loan_position(&self) -> bool {
        matches!(self.instrument_type.as_str(), "bank_loan" | "pension_loan")
    }

    pub fn monthly_interest_payment(&self) -> f64 {
        (self.principal * self.annual_rate) / 12.0
    }
}

impl LoanRecord {
    pub fn effective_rate(&self, current_shir: Option<f64>) -> f64 {
        match self.loan_type {
            LoanType::ShirBased => {
                let shir = current_shir.unwrap_or(0.0395);
                shir + self.spread / 100.0
            }
            LoanType::CpiLinked => {
                if self.base_cpi > 0.0 {
                    (self.current_cpi / self.base_cpi - 1.0) + self.spread / 100.0
                } else {
                    self.interest_rate
                }
            }
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.loan_id.trim().is_empty() {
            errors.push("Loan ID is required".into());
        }
        if self.bank_name.trim().is_empty() {
            errors.push("Bank name is required".into());
        }
        if self.account_number.trim().is_empty() {
            errors.push("Account number is required".into());
        }
        if self.principal <= 0.0 {
            errors.push("Principal must be > 0".into());
        }
        if self.original_principal <= 0.0 {
            errors.push("Original principal must be > 0".into());
        }
        if self.interest_rate < 0.0 {
            errors.push("Interest rate must be >= 0".into());
        }
        if self.spread < 0.0 {
            errors.push("Spread must be >= 0".into());
        }
        if self.monthly_payment <= 0.0 {
            errors.push("Monthly payment must be > 0".into());
        }
        if self.payment_frequency_months <= 0 {
            errors.push("Payment frequency must be > 0".into());
        }

        let origination =
            parse_loan_datetime(&self.origination_date, "origination_date", &mut errors);
        let maturity = parse_loan_datetime(&self.maturity_date, "maturity_date", &mut errors);
        parse_loan_datetime(&self.next_payment_date, "next_payment_date", &mut errors);
        parse_loan_datetime(&self.last_update, "last_update", &mut errors);

        if let (Some(origination), Some(maturity)) = (origination, maturity) {
            if origination >= maturity {
                errors.push("Origination date must be before maturity date".into());
            }
        }

        if matches!(self.loan_type, LoanType::CpiLinked) {
            if self.base_cpi <= 0.0 {
                errors.push("Base CPI must be > 0 for CPI-linked loans".into());
            }
            if self.current_cpi <= 0.0 {
                errors.push("Current CPI must be > 0 for CPI-linked loans".into());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn to_aggregation_input(&self) -> LoanAggregationInput {
        LoanAggregationInput {
            loan_id: Some(self.loan_id.clone()),
            name: self.bank_name.clone(),
            instrument_type: "bank_loan".into(),
            principal: self.principal,
            annual_rate: self.interest_rate,
            monthly_payment: Some(self.monthly_payment),
            maturity_date: Some(self.maturity_date.clone()),
        }
    }
}

fn parse_loan_datetime(
    value: &str,
    field_name: &str,
    errors: &mut Vec<String>,
) -> Option<DateTime<Utc>> {
    if value.trim().is_empty() {
        errors.push(format!("{field_name} is required"));
        return None;
    }

    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").map(|date| {
                DateTime::<Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(0, 0, 0).expect("midnight should be valid"),
                    Utc,
                )
            })
        })
        .map_err(|err| errors.push(format!("Invalid {field_name}: {err}")))
        .ok()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoanFile {
    version: String,
    last_updated: String,
    loans: Vec<LoanRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoansCsvRow {
    loan_id: String,
    bank_name: String,
    account_number: String,
    loan_type: LoanType,
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
    status: LoanStatus,
    last_update: String,
    #[serde(default = "default_ils")]
    currency: String,
}

/// Request body for NATS `api.loans.import_bulk`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoansBulkImportRequest {
    pub loans: Vec<LoanRecord>,
}

/// Per-row outcome for bulk loan import (validation or DB errors).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BulkImportRowError {
    pub index: usize,
    pub loan_id: Option<String>,
    pub message: String,
}

/// Summary returned after `import_bulk` / NATS `api.loans.import_bulk`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoansBulkImportResponse {
    pub applied: usize,
    pub errors: Vec<BulkImportRowError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedLoansImport {
    pub loans: Vec<LoanRecord>,
    pub parse_errors: Vec<BulkImportRowError>,
    /// When parsing CSV, maps each parsed loan index → CSV data row number (1-based, excludes header).
    /// When parsing JSON, this is empty.
    pub row_map: Vec<usize>,
}

/// Parse a loan bulk import file.
///
/// Supported shapes:
/// - JSON: `{ "loans": [...] }`
/// - JSON legacy: `{ "version", "last_updated", "loans" }`
/// - CSV: header row with `LoanRecord` snake_case field names (plus optional `currency`)
pub fn parse_loans_import_file_json(text: &str) -> Result<Vec<LoanRecord>, String> {
    if let Ok(req) = serde_json::from_str::<LoansBulkImportRequest>(text) {
        return Ok(req.loans);
    }

    if let Ok(file) = serde_json::from_str::<LoanFile>(text) {
        return Ok(file.loans);
    }

    parse_loans_import_file_csv(text)
}

/// Parse a loans bulk import payload and preserve per-row CSV parse errors.
///
/// This is intended for UI/CLI pipelines that want to report row-level parse errors
/// while still applying valid rows via `import_bulk`.
pub fn parse_loans_import_file(text: &str) -> Result<ParsedLoansImport, String> {
    if let Ok(req) = serde_json::from_str::<LoansBulkImportRequest>(text) {
        if req.loans.is_empty() {
            return Err("JSON contained no loans".into());
        }
        return Ok(ParsedLoansImport {
            loans: req.loans,
            parse_errors: Vec::new(),
            row_map: Vec::new(),
        });
    }

    if let Ok(file) = serde_json::from_str::<LoanFile>(text) {
        if file.loans.is_empty() {
            return Err("JSON contained no loans".into());
        }
        return Ok(ParsedLoansImport {
            loans: file.loans,
            parse_errors: Vec::new(),
            row_map: Vec::new(),
        });
    }

    let (loans, parse_errors, row_map) = parse_loans_import_file_csv_with_errors(text)?;
    Ok(ParsedLoansImport {
        loans,
        parse_errors,
        row_map,
    })
}

fn parse_loans_import_file_csv(text: &str) -> Result<Vec<LoanRecord>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(text.as_bytes());

    let mut out = Vec::new();
    for (index, row) in reader.deserialize::<LoansCsvRow>().enumerate() {
        let row = row.map_err(|e| format!("CSV row {}: {}", index + 1, e))?;
        out.push(LoanRecord {
            loan_id: row.loan_id,
            bank_name: row.bank_name,
            account_number: row.account_number,
            loan_type: row.loan_type,
            principal: row.principal,
            original_principal: row.original_principal,
            interest_rate: row.interest_rate,
            spread: row.spread,
            base_cpi: row.base_cpi,
            current_cpi: row.current_cpi,
            origination_date: row.origination_date,
            maturity_date: row.maturity_date,
            next_payment_date: row.next_payment_date,
            monthly_payment: row.monthly_payment,
            payment_frequency_months: row.payment_frequency_months,
            status: row.status,
            last_update: row.last_update,
            currency: row.currency,
        });
    }

    if out.is_empty() {
        return Err("CSV contained no data rows".into());
    }

    Ok(out)
}

fn parse_loans_import_file_csv_with_errors(
    text: &str,
) -> Result<(Vec<LoanRecord>, Vec<BulkImportRowError>, Vec<usize>), String> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(text.as_bytes());

    let mut loans = Vec::new();
    let mut parse_errors = Vec::new();
    let mut row_map = Vec::new();

    for (row_index, row) in reader.deserialize::<LoansCsvRow>().enumerate() {
        let csv_row_number = row_index + 1; // 1-based, excludes header
        match row {
            Ok(row) => {
                row_map.push(csv_row_number);
                loans.push(LoanRecord {
                    loan_id: row.loan_id,
                    bank_name: row.bank_name,
                    account_number: row.account_number,
                    loan_type: row.loan_type,
                    principal: row.principal,
                    original_principal: row.original_principal,
                    interest_rate: row.interest_rate,
                    spread: row.spread,
                    base_cpi: row.base_cpi,
                    current_cpi: row.current_cpi,
                    origination_date: row.origination_date,
                    maturity_date: row.maturity_date,
                    next_payment_date: row.next_payment_date,
                    monthly_payment: row.monthly_payment,
                    payment_frequency_months: row.payment_frequency_months,
                    status: row.status,
                    last_update: row.last_update,
                    currency: row.currency,
                });
            }
            Err(e) => {
                parse_errors.push(BulkImportRowError {
                    index: csv_row_number,
                    loan_id: None,
                    message: e.to_string(),
                });
            }
        }
    }

    if loans.is_empty() && parse_errors.is_empty() {
        return Err("CSV contained no data rows".into());
    }

    Ok((loans, parse_errors, row_map))
}

#[derive(Clone)]
pub struct LoanRepository {
    pool: SqlitePool,
}

impl LoanRepository {
    pub async fn load_default() -> anyhow::Result<Self> {
        let database_url = detect_default_database_url()?;
        let legacy_import_path = legacy_import_path();
        let repo = Self::connect(&database_url).await?;
        repo.seed_from_legacy_if_empty(legacy_import_path.as_deref())
            .await?;
        Ok(repo)
    }

    pub async fn load(path: PathBuf) -> anyhow::Result<Self> {
        let database_url = sqlite_database_url(&path);
        let legacy_import_path = if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            Some(path)
        } else {
            None
        };

        let repo = Self::connect(&database_url).await?;
        repo.seed_from_legacy_if_empty(legacy_import_path.as_deref())
            .await?;
        Ok(repo)
    }

    async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)
            .with_context(|| format!("invalid loan database URL: {database_url}"))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .with_context(|| format!("failed to connect to loan database {database_url}"))?;

        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_schema(pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loans (
                loan_id TEXT PRIMARY KEY,
                bank_name TEXT NOT NULL,
                account_number TEXT NOT NULL,
                loan_type TEXT NOT NULL,
                principal REAL NOT NULL,
                original_principal REAL NOT NULL,
                interest_rate REAL NOT NULL,
                spread REAL NOT NULL,
                base_cpi REAL NOT NULL,
                current_cpi REAL NOT NULL,
                origination_date TEXT NOT NULL,
                maturity_date TEXT NOT NULL,
                next_payment_date TEXT NOT NULL,
                monthly_payment REAL NOT NULL,
                payment_frequency_months INTEGER NOT NULL,
                status TEXT NOT NULL,
                last_update TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'ILS'
            )
            "#,
        )
        .execute(pool)
        .await
        .context("failed to create loans schema")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_loans_status ON loans(status)")
            .execute(pool)
            .await
            .context("failed to create loans status index")?;

        Ok(())
    }

    async fn seed_from_legacy_if_empty(&self, legacy_path: Option<&Path>) -> anyhow::Result<()> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM loans")
            .fetch_one(&self.pool)
            .await
            .context("failed to count loans")?;

        if count > 0 {
            return Ok(());
        }

        let Some(path) = legacy_path.filter(|path| path.exists()) else {
            return Ok(());
        };

        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read legacy loan store {}", path.display()))?;
        let file: LoanFile = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse legacy loan store {}", path.display()))?;

        for loan in file.loans {
            loan.validate()
                .map_err(|errors| anyhow!(errors.join("; ")))?;
            self.upsert(&loan).await?;
        }

        Ok(())
    }

    pub async fn list(&self) -> Vec<LoanRecord> {
        let rows = sqlx::query(
            r#"
            SELECT loan_id, bank_name, account_number, loan_type, principal, original_principal,
                   interest_rate, spread, base_cpi, current_cpi, origination_date, maturity_date,
                   next_payment_date, monthly_payment, payment_frequency_months, status, last_update,
                   currency
            FROM loans
            ORDER BY loan_id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .filter_map(|row| loan_from_row(&row).ok())
            .collect()
    }

    pub async fn get(&self, loan_id: &str) -> Option<LoanRecord> {
        let row = sqlx::query(
            r#"
            SELECT loan_id, bank_name, account_number, loan_type, principal, original_principal,
                   interest_rate, spread, base_cpi, current_cpi, origination_date, maturity_date,
                   next_payment_date, monthly_payment, payment_frequency_months, status, last_update,
                   currency
            FROM loans
            WHERE loan_id = ?
            "#,
        )
        .bind(loan_id)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        loan_from_row(&row).ok()
    }

    pub async fn create(&self, loan: LoanRecord) -> Result<(), String> {
        if let Err(errors) = loan.validate() {
            return Err(errors.join("; "));
        }

        if self.get(&loan.loan_id).await.is_some() {
            return Err(format!("Loan with ID {} already exists", loan.loan_id));
        }

        self.upsert(&loan).await.map_err(|err| err.to_string())
    }

    pub async fn update(&self, loan_id: &str, loan: LoanRecord) -> Result<(), String> {
        if loan_id != loan.loan_id {
            return Err("Loan ID in path must match request body".into());
        }
        if let Err(errors) = loan.validate() {
            return Err(errors.join("; "));
        }
        if self.get(loan_id).await.is_none() {
            return Err(format!("Loan with ID {loan_id} not found"));
        }

        self.upsert(&loan).await.map_err(|err| err.to_string())
    }

    pub async fn delete(&self, loan_id: &str) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM loans WHERE loan_id = ?")
            .bind(loan_id)
            .execute(&self.pool)
            .await
            .map_err(|err| err.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    /// Validate and upsert each loan; invalid rows are reported in `errors` without aborting the batch.
    pub async fn import_bulk(&self, loans: Vec<LoanRecord>) -> LoansBulkImportResponse {
        let mut applied = 0usize;
        let mut errors = Vec::new();
        for (index, loan) in loans.into_iter().enumerate() {
            if let Err(msgs) = loan.validate() {
                errors.push(BulkImportRowError {
                    index,
                    loan_id: if loan.loan_id.trim().is_empty() {
                        None
                    } else {
                        Some(loan.loan_id.clone())
                    },
                    message: msgs.join("; "),
                });
                continue;
            }
            match self.upsert(&loan).await {
                Ok(()) => applied += 1,
                Err(e) => errors.push(BulkImportRowError {
                    index,
                    loan_id: Some(loan.loan_id.clone()),
                    message: e.to_string(),
                }),
            }
        }
        LoansBulkImportResponse { applied, errors }
    }

    async fn upsert(&self, loan: &LoanRecord) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO loans (
                loan_id, bank_name, account_number, loan_type, principal, original_principal,
                interest_rate, spread, base_cpi, current_cpi, origination_date, maturity_date,
                next_payment_date, monthly_payment, payment_frequency_months, status, last_update,
                currency
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(loan_id) DO UPDATE SET
                bank_name = excluded.bank_name,
                account_number = excluded.account_number,
                loan_type = excluded.loan_type,
                principal = excluded.principal,
                original_principal = excluded.original_principal,
                interest_rate = excluded.interest_rate,
                spread = excluded.spread,
                base_cpi = excluded.base_cpi,
                current_cpi = excluded.current_cpi,
                origination_date = excluded.origination_date,
                maturity_date = excluded.maturity_date,
                next_payment_date = excluded.next_payment_date,
                monthly_payment = excluded.monthly_payment,
                payment_frequency_months = excluded.payment_frequency_months,
                status = excluded.status,
                last_update = excluded.last_update,
                currency = excluded.currency
            "#,
        )
        .bind(&loan.loan_id)
        .bind(&loan.bank_name)
        .bind(&loan.account_number)
        .bind(loan_type_to_str(&loan.loan_type))
        .bind(loan.principal)
        .bind(loan.original_principal)
        .bind(loan.interest_rate)
        .bind(loan.spread)
        .bind(loan.base_cpi)
        .bind(loan.current_cpi)
        .bind(&loan.origination_date)
        .bind(&loan.maturity_date)
        .bind(&loan.next_payment_date)
        .bind(loan.monthly_payment)
        .bind(loan.payment_frequency_months)
        .bind(loan_status_to_str(&loan.status))
        .bind(&loan.last_update)
        .bind(&loan.currency)
        .execute(&self.pool)
        .await
        .context("failed to upsert loan")?;

        Ok(())
    }
}

fn detect_default_database_url() -> anyhow::Result<String> {
    if let Ok(url) = std::env::var("LOANS_BACKEND_DB_URL") {
        return Ok(url);
    }

    if let Ok(path) = std::env::var("LOANS_BACKEND_PATH") {
        if path.starts_with("sqlite:") || path.ends_with(".db") {
            return Ok(if path.starts_with("sqlite:") {
                path
            } else {
                sqlite_database_url(Path::new(&path))
            });
        }
    }

    let candidates = [
        PathBuf::from("agents/backend/data/ledger.db"),
        PathBuf::from("data/ledger.db"),
    ];

    if let Some(existing) = candidates.iter().find(|path| path.exists()) {
        return Ok(sqlite_database_url(existing));
    }

    let preferred = Path::new("agents/backend/data");
    if preferred.exists() {
        Ok(sqlite_database_url(&preferred.join("ledger.db")))
    } else {
        Ok(sqlite_database_url(Path::new("data/ledger.db")))
    }
}

fn sqlite_database_url(path: &Path) -> String {
    format!("sqlite:{}", path.display())
}

fn legacy_import_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("LOANS_IMPORT_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(path) = std::env::var("LOANS_BACKEND_PATH") {
        let path = PathBuf::from(path);
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") && path.exists() {
            return Some(path);
        }
    }

    let path = PathBuf::from("config/loans.json");
    path.exists().then_some(path)
}

fn loan_type_to_str(value: &LoanType) -> &'static str {
    match value {
        LoanType::ShirBased => "SHIR_BASED",
        LoanType::CpiLinked => "CPI_LINKED",
    }
}

fn loan_status_to_str(value: &LoanStatus) -> &'static str {
    match value {
        LoanStatus::Active => "ACTIVE",
        LoanStatus::PaidOff => "PAID_OFF",
        LoanStatus::Defaulted => "DEFAULTED",
    }
}

fn loan_type_from_str(value: &str) -> anyhow::Result<LoanType> {
    match value {
        "SHIR_BASED" => Ok(LoanType::ShirBased),
        "CPI_LINKED" => Ok(LoanType::CpiLinked),
        _ => Err(anyhow!("unknown loan type {value}")),
    }
}

fn loan_status_from_str(value: &str) -> anyhow::Result<LoanStatus> {
    match value {
        "ACTIVE" => Ok(LoanStatus::Active),
        "PAID_OFF" => Ok(LoanStatus::PaidOff),
        "DEFAULTED" => Ok(LoanStatus::Defaulted),
        _ => Err(anyhow!("unknown loan status {value}")),
    }
}

fn loan_type_to_proto(t: &LoanType) -> i32 {
    match t {
        LoanType::ShirBased => 1, // LOAN_TYPE_SHIR_BASED
        LoanType::CpiLinked => 2, // LOAN_TYPE_CPI_LINKED
    }
}

fn loan_status_to_proto(s: &LoanStatus) -> i32 {
    match s {
        LoanStatus::Active => 1,    // LOAN_STATUS_ACTIVE
        LoanStatus::PaidOff => 2,   // LOAN_STATUS_PAID_OFF
        LoanStatus::Defaulted => 3, // LOAN_STATUS_DEFAULTED
    }
}

/// Convert a `LoanRecord` to the protobuf `Loan` message for binary/proto responses.
pub fn loan_record_to_proto(r: &LoanRecord) -> ProtoLoan {
    ProtoLoan {
        loan_id: r.loan_id.clone(),
        bank_name: r.bank_name.clone(),
        account_number: r.account_number.clone(),
        loan_type: loan_type_to_proto(&r.loan_type),
        principal: r.principal,
        original_principal: r.original_principal,
        interest_rate: r.interest_rate,
        spread: r.spread,
        base_cpi: r.base_cpi,
        current_cpi: r.current_cpi,
        origination_date: r.origination_date.clone(),
        maturity_date: r.maturity_date.clone(),
        next_payment_date: r.next_payment_date.clone(),
        monthly_payment: r.monthly_payment,
        payment_frequency_months: r.payment_frequency_months,
        status: loan_status_to_proto(&r.status),
        last_update: r.last_update.clone(),
    }
}

/// Build proto `LoansResponse` from loan records for NATS `api.loans.list.proto`.
pub fn loans_response_proto(records: &[LoanRecord]) -> LoansResponse {
    LoansResponse {
        loans: records.iter().map(loan_record_to_proto).collect(),
    }
}

fn loan_from_row(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<LoanRecord> {
    Ok(LoanRecord {
        loan_id: row.try_get("loan_id")?,
        bank_name: row.try_get("bank_name")?,
        account_number: row.try_get("account_number")?,
        loan_type: loan_type_from_str(row.try_get::<&str, _>("loan_type")?)?,
        principal: row.try_get("principal")?,
        original_principal: row.try_get("original_principal")?,
        interest_rate: row.try_get("interest_rate")?,
        spread: row.try_get("spread")?,
        base_cpi: row.try_get("base_cpi")?,
        current_cpi: row.try_get("current_cpi")?,
        origination_date: row.try_get("origination_date")?,
        maturity_date: row.try_get("maturity_date")?,
        next_payment_date: row.try_get("next_payment_date")?,
        monthly_payment: row.try_get("monthly_payment")?,
        payment_frequency_months: row.try_get("payment_frequency_months")?,
        status: loan_status_from_str(row.try_get::<&str, _>("status")?)?,
        last_update: row.try_get("last_update")?,
        currency: row
            .try_get("currency")
            .unwrap_or_else(|_| "ILS".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        loans_response_proto, parse_loans_import_file, parse_loans_import_file_json, LoanRecord,
        LoanRepository, LoanStatus, LoanType,
    };

    fn sample_loan() -> LoanRecord {
        LoanRecord {
            loan_id: "loan-1".into(),
            bank_name: "Discount".into(),
            account_number: "123456789".into(),
            loan_type: LoanType::ShirBased,
            principal: 1000.0,
            original_principal: 1200.0,
            interest_rate: 4.0,
            spread: 0.5,
            base_cpi: 0.0,
            current_cpi: 0.0,
            origination_date: "2025-01-01T00:00:00Z".into(),
            maturity_date: "2030-01-01T00:00:00Z".into(),
            next_payment_date: "2025-02-01T00:00:00Z".into(),
            monthly_payment: 100.0,
            payment_frequency_months: 1,
            status: LoanStatus::Active,
            last_update: "2025-01-15T00:00:00Z".into(),
            currency: "ILS".into(),
        }
    }

    #[test]
    fn parse_loans_import_accepts_loans_wrapper_and_legacy_file() {
        let wrap = r#"{"loans":[{"loan_id":"a","bank_name":"B","account_number":"1","loan_type":"SHIR_BASED","principal":1.0,"original_principal":1.0,"interest_rate":0.0,"spread":0.0,"base_cpi":0.0,"current_cpi":0.0,"origination_date":"2025-01-01T00:00:00Z","maturity_date":"2030-01-01T00:00:00Z","next_payment_date":"2025-02-01T00:00:00Z","monthly_payment":1.0,"payment_frequency_months":1,"status":"ACTIVE","last_update":"2025-01-15T00:00:00Z"}]}"#;
        let loans = parse_loans_import_file_json(wrap).expect("wrapper");
        assert_eq!(loans.len(), 1);
        assert_eq!(loans[0].loan_id, "a");

        let legacy = r#"{"version":"1","last_updated":"2025-01-01T00:00:00Z","loans":[{"loan_id":"b","bank_name":"B","account_number":"1","loan_type":"SHIR_BASED","principal":1.0,"original_principal":1.0,"interest_rate":0.0,"spread":0.0,"base_cpi":0.0,"current_cpi":0.0,"origination_date":"2025-01-01T00:00:00Z","maturity_date":"2030-01-01T00:00:00Z","next_payment_date":"2025-02-01T00:00:00Z","monthly_payment":1.0,"payment_frequency_months":1,"status":"ACTIVE","last_update":"2025-01-15T00:00:00Z"}]}"#;
        let loans = parse_loans_import_file_json(legacy).expect("legacy");
        assert_eq!(loans.len(), 1);
        assert_eq!(loans[0].loan_id, "b");
    }

    #[test]
    fn parse_loans_import_accepts_csv_with_header_row() {
        let csv = concat!(
            "loan_id,bank_name,account_number,loan_type,principal,original_principal,interest_rate,spread,base_cpi,current_cpi,origination_date,maturity_date,next_payment_date,monthly_payment,payment_frequency_months,status,last_update,currency\n",
            "csv-1,Discount,123,SHIR_BASED,1000.0,1200.0,4.0,0.5,0.0,0.0,2025-01-01T00:00:00Z,2030-01-01T00:00:00Z,2025-02-01T00:00:00Z,100.0,1,ACTIVE,2025-01-15T00:00:00Z,ILS\n"
        );

        let loans = parse_loans_import_file_json(csv).expect("csv parse");
        assert_eq!(loans.len(), 1);
        assert_eq!(loans[0].loan_id, "csv-1");
        assert_eq!(loans[0].bank_name, "Discount");
        assert_eq!(loans[0].loan_type, LoanType::ShirBased);
        assert_eq!(loans[0].status, LoanStatus::Active);
        assert_eq!(loans[0].currency, "ILS");
    }

    #[test]
    fn parse_loans_import_csv_collects_row_errors_and_keeps_valid_rows() {
        let csv = concat!(
            "loan_id,bank_name,account_number,loan_type,principal,original_principal,interest_rate,spread,base_cpi,current_cpi,origination_date,maturity_date,next_payment_date,monthly_payment,payment_frequency_months,status,last_update,currency\n",
            "bad-1,Discount,123,SHIR_BASED,NOT_A_NUMBER,1200.0,4.0,0.5,0.0,0.0,2025-01-01T00:00:00Z,2030-01-01T00:00:00Z,2025-02-01T00:00:00Z,100.0,1,ACTIVE,2025-01-15T00:00:00Z,ILS\n",
            "ok-1,Discount,123,SHIR_BASED,1000.0,1200.0,4.0,0.5,0.0,0.0,2025-01-01T00:00:00Z,2030-01-01T00:00:00Z,2025-02-01T00:00:00Z,100.0,1,ACTIVE,2025-01-15T00:00:00Z,ILS\n"
        );

        let parsed = parse_loans_import_file(csv).expect("parse with errors");
        assert_eq!(parsed.loans.len(), 1);
        assert_eq!(parsed.loans[0].loan_id, "ok-1");
        assert_eq!(parsed.parse_errors.len(), 1);
        assert_eq!(parsed.parse_errors[0].index, 1);
        assert!(!parsed.row_map.is_empty());
        assert_eq!(parsed.row_map.len(), 1);
        assert_eq!(parsed.row_map[0], 2);
    }

    #[tokio::test]
    async fn import_bulk_skips_invalid_and_applies_valid() {
        let path =
            std::env::temp_dir().join(format!("aether_api_loans_bulk_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let repo = LoanRepository::load(path.clone()).await.expect("repo");
        let good = sample_loan();
        let mut good2 = sample_loan();
        good2.loan_id = "loan-2".into();
        let bad = LoanRecord {
            loan_id: String::new(),
            ..sample_loan()
        };
        let res = repo.import_bulk(vec![bad, good, good2]).await;
        assert_eq!(res.applied, 2);
        assert_eq!(res.errors.len(), 1);
        assert_eq!(res.errors[0].index, 0);
        let list = repo.list().await;
        assert_eq!(list.len(), 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn loan_record_deserializes_canonical_contract_shape() {
        let raw = r#"{
            "loan_id":"loan-1",
            "bank_name":"Discount",
            "account_number":"123456789",
            "loan_type":"SHIR_BASED",
            "principal":1000.0,
            "original_principal":1200.0,
            "interest_rate":4.0,
            "spread":0.5,
            "base_cpi":0.0,
            "current_cpi":0.0,
            "origination_date":"2025-01-01T00:00:00Z",
            "maturity_date":"2030-01-01T00:00:00Z",
            "next_payment_date":"2025-02-01T00:00:00Z",
            "monthly_payment":100.0,
            "payment_frequency_months":1,
            "status":"ACTIVE",
            "last_update":"2025-01-15T00:00:00Z"
        }"#;

        let loan: LoanRecord = serde_json::from_str(raw).expect("deserialize canonical loan");

        assert_eq!(loan.loan_type, LoanType::ShirBased);
        assert_eq!(loan.status, LoanStatus::Active);
        assert_eq!(loan.loan_id, "loan-1");
    }

    #[test]
    fn loans_response_proto_encodes_sample_loan() {
        let records = vec![sample_loan()];
        let resp = loans_response_proto(&records);
        assert_eq!(resp.loans.len(), 1);
        let loan = &resp.loans[0];
        assert_eq!(loan.loan_id, "loan-1");
        assert_eq!(loan.bank_name, "Discount");
        assert_eq!(loan.status, 1); // LOAN_STATUS_ACTIVE
        assert_eq!(loan.loan_type, 1); // LOAN_TYPE_SHIR_BASED
    }

    #[test]
    fn loan_record_accepts_legacy_shir_alias() {
        let raw = r#"{
            "loan_id":"loan-legacy",
            "bank_name":"Discount",
            "account_number":"123456789",
            "loan_type":"SHIR",
            "principal":1000.0,
            "original_principal":1000.0,
            "interest_rate":4.0,
            "spread":0.5,
            "base_cpi":0.0,
            "current_cpi":0.0,
            "origination_date":"2025-01-01T00:00:00Z",
            "maturity_date":"2030-01-01T00:00:00Z",
            "next_payment_date":"2025-02-01T00:00:00Z",
            "monthly_payment":100.0,
            "payment_frequency_months":1,
            "status":"ACTIVE",
            "last_update":"2025-01-15T00:00:00Z"
        }"#;

        let loan: LoanRecord = serde_json::from_str(raw).expect("deserialize legacy SHIR loan");

        assert_eq!(loan.loan_type, LoanType::ShirBased);
    }

    #[test]
    fn loan_record_rejects_unknown_fields() {
        let raw = r#"{
            "loan_id":"loan-1",
            "bank_name":"Discount",
            "account_number":"123456789",
            "loan_type":"SHIR_BASED",
            "principal":1000.0,
            "original_principal":1200.0,
            "interest_rate":4.0,
            "spread":0.5,
            "base_cpi":0.0,
            "current_cpi":0.0,
            "origination_date":"2025-01-01T00:00:00Z",
            "maturity_date":"2030-01-01T00:00:00Z",
            "next_payment_date":"2025-02-01T00:00:00Z",
            "monthly_payment":100.0,
            "payment_frequency_months":1,
            "status":"ACTIVE",
            "last_update":"2025-01-15T00:00:00Z",
            "unexpected":"value"
        }"#;

        let err = serde_json::from_str::<LoanRecord>(raw).expect_err("unknown field should fail");

        assert!(err.to_string().contains("unexpected"));
    }

    #[test]
    fn cpi_linked_loans_require_positive_cpi_values() {
        let mut loan = sample_loan();
        loan.loan_type = LoanType::CpiLinked;
        loan.base_cpi = 0.0;
        loan.current_cpi = 0.0;

        let errors = loan.validate().expect_err("missing CPI values should fail");

        assert!(errors
            .iter()
            .any(|err| err.contains("Base CPI must be > 0")));
        assert!(errors
            .iter()
            .any(|err| err.contains("Current CPI must be > 0")));
    }

    #[test]
    fn loan_validation_rejects_inverted_dates() {
        let mut loan = sample_loan();
        loan.origination_date = "2030-01-01T00:00:00Z".into();
        loan.maturity_date = "2025-01-01T00:00:00Z".into();

        let errors = loan.validate().expect_err("inverted dates should fail");

        assert!(errors
            .iter()
            .any(|err| err.contains("Origination date must be before maturity date")));
    }

    #[tokio::test]
    async fn loan_repository_insert_and_retrieve() {
        use sqlx::sqlite::SqlitePoolOptions;

        let db_url = "sqlite::memory:";

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(db_url)
            .await
            .expect("create test pool");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loans (
                loan_id TEXT PRIMARY KEY,
                bank_name TEXT NOT NULL,
                account_number TEXT NOT NULL,
                loan_type TEXT NOT NULL,
                principal REAL NOT NULL,
                original_principal REAL NOT NULL,
                interest_rate REAL NOT NULL,
                spread REAL NOT NULL,
                base_cpi REAL NOT NULL,
                current_cpi REAL NOT NULL,
                origination_date TEXT NOT NULL,
                maturity_date TEXT NOT NULL,
                next_payment_date TEXT NOT NULL,
                monthly_payment REAL NOT NULL,
                payment_frequency_months INTEGER NOT NULL,
                status TEXT NOT NULL,
                last_update TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'ILS'
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create schema");

        let repo = super::LoanRepository { pool };

        let loan = LoanRecord {
            loan_id: "test-loan-1".into(),
            bank_name: "Test Bank".into(),
            account_number: "111222333".into(),
            loan_type: LoanType::ShirBased,
            principal: 5000.0,
            original_principal: 5000.0,
            interest_rate: 4.5,
            spread: 0.25,
            base_cpi: 0.0,
            current_cpi: 0.0,
            origination_date: "2025-01-01T00:00:00Z".into(),
            maturity_date: "2030-01-01T00:00:00Z".into(),
            next_payment_date: "2025-02-01T00:00:00Z".into(),
            monthly_payment: 150.0,
            payment_frequency_months: 1,
            status: LoanStatus::Active,
            last_update: "2025-01-15T00:00:00Z".into(),
            currency: "ILS".into(),
        };

        repo.create(loan.clone()).await.expect("insert loan");

        let retrieved = repo.get("test-loan-1").await;
        assert!(retrieved.is_some(), "loan should be retrievable");

        let found = retrieved.unwrap();
        assert_eq!(found.loan_id, loan.loan_id);
        assert_eq!(found.bank_name, loan.bank_name);
        assert_eq!(found.principal, loan.principal);
        assert_eq!(found.loan_type, loan.loan_type);
        assert_eq!(found.status, loan.status);
    }

    #[tokio::test]
    async fn loan_repository_list_returns_all_loans() {
        use sqlx::sqlite::SqlitePoolOptions;

        let db_url = "sqlite::memory:";

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(db_url)
            .await
            .expect("create test pool");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loans (
                loan_id TEXT PRIMARY KEY,
                bank_name TEXT NOT NULL,
                account_number TEXT NOT NULL,
                loan_type TEXT NOT NULL,
                principal REAL NOT NULL,
                original_principal REAL NOT NULL,
                interest_rate REAL NOT NULL,
                spread REAL NOT NULL,
                base_cpi REAL NOT NULL,
                current_cpi REAL NOT NULL,
                origination_date TEXT NOT NULL,
                maturity_date TEXT NOT NULL,
                next_payment_date TEXT NOT NULL,
                monthly_payment REAL NOT NULL,
                payment_frequency_months INTEGER NOT NULL,
                status TEXT NOT NULL,
                last_update TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'ILS'
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create schema");

        let repo = super::LoanRepository { pool };

        for i in 1..=3 {
            let loan = LoanRecord {
                loan_id: format!("loan-{}", i),
                bank_name: "Test Bank".into(),
                account_number: format!("acc-{}", i),
                loan_type: LoanType::ShirBased,
                principal: 1000.0 * i as f64,
                original_principal: 1000.0 * i as f64,
                interest_rate: 4.0,
                spread: 0.5,
                base_cpi: 0.0,
                current_cpi: 0.0,
                origination_date: "2025-01-01T00:00:00Z".into(),
                maturity_date: "2030-01-01T00:00:00Z".into(),
                next_payment_date: "2025-02-01T00:00:00Z".into(),
                monthly_payment: 100.0 * i as f64,
                payment_frequency_months: 1,
                status: LoanStatus::Active,
                last_update: "2025-01-15T00:00:00Z".into(),
                currency: "ILS".into(),
            };
            repo.create(loan).await.expect("insert loan");
        }

        let loans = repo.list().await;
        assert_eq!(loans.len(), 3, "should have 3 loans");
    }

    #[tokio::test]
    async fn loan_repository_update_existing_loan() {
        use sqlx::sqlite::SqlitePoolOptions;

        let db_url = "sqlite::memory:";

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(db_url)
            .await
            .expect("create test pool");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loans (
                loan_id TEXT PRIMARY KEY,
                bank_name TEXT NOT NULL,
                account_number TEXT NOT NULL,
                loan_type TEXT NOT NULL,
                principal REAL NOT NULL,
                original_principal REAL NOT NULL,
                interest_rate REAL NOT NULL,
                spread REAL NOT NULL,
                base_cpi REAL NOT NULL,
                current_cpi REAL NOT NULL,
                origination_date TEXT NOT NULL,
                maturity_date TEXT NOT NULL,
                next_payment_date TEXT NOT NULL,
                monthly_payment REAL NOT NULL,
                payment_frequency_months INTEGER NOT NULL,
                status TEXT NOT NULL,
                last_update TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'ILS'
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create schema");

        let repo = super::LoanRepository { pool };

        let loan = LoanRecord {
            loan_id: "update-test".into(),
            bank_name: "Original Bank".into(),
            account_number: "123".into(),
            loan_type: LoanType::ShirBased,
            principal: 1000.0,
            original_principal: 1000.0,
            interest_rate: 4.0,
            spread: 0.5,
            base_cpi: 0.0,
            current_cpi: 0.0,
            origination_date: "2025-01-01T00:00:00Z".into(),
            maturity_date: "2030-01-01T00:00:00Z".into(),
            next_payment_date: "2025-02-01T00:00:00Z".into(),
            monthly_payment: 100.0,
            payment_frequency_months: 1,
            status: LoanStatus::Active,
            last_update: "2025-01-15T00:00:00Z".into(),
            currency: "ILS".into(),
        };

        repo.create(loan.clone()).await.expect("insert loan");

        let mut updated = loan.clone();
        updated.bank_name = "New Bank".into();
        updated.principal = 800.0;
        updated.status = LoanStatus::PaidOff;

        repo.update("update-test", updated.clone())
            .await
            .expect("update loan");

        let retrieved = repo.get("update-test").await.unwrap();
        assert_eq!(retrieved.bank_name, "New Bank");
        assert_eq!(retrieved.principal, 800.0);
        assert_eq!(retrieved.status, LoanStatus::PaidOff);
    }

    #[tokio::test]
    async fn loan_repository_json_import_round_trip() {
        use sqlx::sqlite::SqlitePoolOptions;

        let db_url = "sqlite::memory:";

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(db_url)
            .await
            .expect("create test pool");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loans (
                loan_id TEXT PRIMARY KEY,
                bank_name TEXT NOT NULL,
                account_number TEXT NOT NULL,
                loan_type TEXT NOT NULL,
                principal REAL NOT NULL,
                original_principal REAL NOT NULL,
                interest_rate REAL NOT NULL,
                spread REAL NOT NULL,
                base_cpi REAL NOT NULL,
                current_cpi REAL NOT NULL,
                origination_date TEXT NOT NULL,
                maturity_date TEXT NOT NULL,
                next_payment_date TEXT NOT NULL,
                monthly_payment REAL NOT NULL,
                payment_frequency_months INTEGER NOT NULL,
                status TEXT NOT NULL,
                last_update TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'ILS'
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create schema");

        let repo = super::LoanRepository { pool };

        let json_content = r#"{
            "version": "1.0",
            "last_updated": "2025-02-15T00:00:00Z",
            "loans": [
                {
                    "loan_id": "json-import-1",
                    "bank_name": "JSON Bank",
                    "account_number": "999888777",
                    "loan_type": "SHIR_BASED",
                    "principal": 2500.0,
                    "original_principal": 3000.0,
                    "interest_rate": 3.75,
                    "spread": 0.3,
                    "base_cpi": 0.0,
                    "current_cpi": 0.0,
                    "origination_date": "2025-02-01T00:00:00Z",
                    "maturity_date": "2031-02-01T00:00:00Z",
                    "next_payment_date": "2025-03-01T00:00:00Z",
                    "monthly_payment": 75.0,
                    "payment_frequency_months": 1,
                    "status": "ACTIVE",
                    "last_update": "2025-02-15T00:00:00Z"
                },
                {
                    "loan_id": "json-import-2",
                    "bank_name": "CPI Bank",
                    "account_number": "111222333",
                    "loan_type": "CPI_LINKED",
                    "principal": 10000.0,
                    "original_principal": 10000.0,
                    "interest_rate": 2.5,
                    "spread": 0.1,
                    "base_cpi": 250.0,
                    "current_cpi": 255.0,
                    "origination_date": "2025-01-15T00:00:00Z",
                    "maturity_date": "2035-01-15T00:00:00Z",
                    "next_payment_date": "2025-02-15T00:00:00Z",
                    "monthly_payment": 200.0,
                    "payment_frequency_months": 1,
                    "status": "ACTIVE",
                    "last_update": "2025-01-15T00:00:00Z"
                }
            ]
        }"#;

        let file: super::LoanFile = serde_json::from_str(json_content).expect("parse JSON");

        for loan in file.loans {
            repo.upsert(&loan).await.expect("insert loan from JSON");
        }

        let loans = repo.list().await;
        assert_eq!(loans.len(), 2, "should have 2 loans from JSON");

        let first = repo.get("json-import-1").await.unwrap();
        assert_eq!(first.bank_name, "JSON Bank");
        assert_eq!(first.loan_type, LoanType::ShirBased);
        assert_eq!(first.principal, 2500.0);

        let second = repo.get("json-import-2").await.unwrap();
        assert_eq!(second.bank_name, "CPI Bank");
        assert_eq!(second.loan_type, LoanType::CpiLinked);
        assert_eq!(second.base_cpi, 250.0);
        assert_eq!(second.current_cpi, 255.0);
    }

    #[test]
    fn effective_rate_for_shir_based_uses_current_shir() {
        let loan = LoanRecord {
            loan_id: "test-shir".into(),
            bank_name: "Test".into(),
            account_number: "123".into(),
            loan_type: LoanType::ShirBased,
            principal: 1000.0,
            original_principal: 1000.0,
            interest_rate: 4.0,
            spread: 0.5,
            base_cpi: 0.0,
            current_cpi: 0.0,
            origination_date: "2025-01-01T00:00:00Z".into(),
            maturity_date: "2030-01-01T00:00:00Z".into(),
            next_payment_date: "2025-02-01T00:00:00Z".into(),
            monthly_payment: 100.0,
            payment_frequency_months: 1,
            status: LoanStatus::Active,
            last_update: "2025-01-15T00:00:00Z".into(),
            currency: "ILS".into(),
        };

        let rate_with_shir = loan.effective_rate(Some(0.035));
        assert!((rate_with_shir - 0.04).abs() < 0.001);

        let rate_with_default = loan.effective_rate(None);
        assert!((rate_with_default - 0.0445).abs() < 0.001);
    }

    #[test]
    fn effective_rate_for_cpi_linked_uses_cpi_ratio() {
        let loan = LoanRecord {
            loan_id: "test-cpi".into(),
            bank_name: "Test".into(),
            account_number: "123".into(),
            loan_type: LoanType::CpiLinked,
            principal: 1000.0,
            original_principal: 1000.0,
            interest_rate: 2.0,
            spread: 0.3,
            base_cpi: 250.0,
            current_cpi: 255.0,
            origination_date: "2025-01-01T00:00:00Z".into(),
            maturity_date: "2030-01-01T00:00:00Z".into(),
            next_payment_date: "2025-02-01T00:00:00Z".into(),
            monthly_payment: 100.0,
            payment_frequency_months: 1,
            status: LoanStatus::Active,
            last_update: "2025-01-15T00:00:00Z".into(),
            currency: "ILS".into(),
        };

        let rate = loan.effective_rate(None);
        let expected = (255.0 / 250.0 - 1.0) + 0.3 / 100.0;
        assert!((rate - expected).abs() < 0.001);
    }
}
