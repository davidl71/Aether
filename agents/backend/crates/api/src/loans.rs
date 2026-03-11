use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoanType {
    ShirBased,
    CpiLinked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoanStatus {
    Active,
    PaidOff,
    Defaulted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

        let origination = parse_loan_datetime(&self.origination_date, "origination_date", &mut errors);
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
struct LoanFile {
    version: String,
    last_updated: String,
    loans: Vec<LoanRecord>,
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
        repo.seed_from_legacy_if_empty(legacy_import_path.as_deref()).await?;
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
        repo.seed_from_legacy_if_empty(legacy_import_path.as_deref()).await?;
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
                last_update TEXT NOT NULL
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
            loan.validate().map_err(|errors| anyhow!(errors.join("; ")))?;
            self.upsert(&loan).await?;
        }

        Ok(())
    }

    pub async fn list(&self) -> Vec<LoanRecord> {
        let rows = sqlx::query(
            r#"
            SELECT loan_id, bank_name, account_number, loan_type, principal, original_principal,
                   interest_rate, spread, base_cpi, current_cpi, origination_date, maturity_date,
                   next_payment_date, monthly_payment, payment_frequency_months, status, last_update
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
                   next_payment_date, monthly_payment, payment_frequency_months, status, last_update
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

    async fn upsert(&self, loan: &LoanRecord) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO loans (
                loan_id, bank_name, account_number, loan_type, principal, original_principal,
                interest_rate, spread, base_cpi, current_cpi, origination_date, maturity_date,
                next_payment_date, monthly_payment, payment_frequency_months, status, last_update
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                last_update = excluded.last_update
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

    let candidates = [PathBuf::from("agents/backend/data/ledger.db"), PathBuf::from("data/ledger.db")];

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
    })
}
