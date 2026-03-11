use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

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
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map(|date| {
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
    path: PathBuf,
    inner: Arc<RwLock<BTreeMap<String, LoanRecord>>>,
}

impl LoanRepository {
    pub fn load_default() -> anyhow::Result<Self> {
        let path = detect_default_path()?;
        if !path.exists() {
            if let Some(legacy_path) = legacy_config_path().filter(|legacy| legacy.exists()) {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create {}", parent.display()))?;
                }
                fs::copy(&legacy_path, &path).with_context(|| {
                    format!(
                        "failed to seed backend loan store from {} to {}",
                        legacy_path.display(),
                        path.display()
                    )
                })?;
            }
        }
        Self::load(path)
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let map = if path.exists() {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read loan store {}", path.display()))?;
            let file: LoanFile = serde_json::from_str(&text)
                .with_context(|| format!("failed to parse loan store {}", path.display()))?;
            let mut map = BTreeMap::new();
            for loan in file.loans {
                loan.validate().map_err(|errors| anyhow!(errors.join("; ")))?;
                map.insert(loan.loan_id.clone(), loan);
            }
            map
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            path,
            inner: Arc::new(RwLock::new(map)),
        })
    }

    pub async fn list(&self) -> Vec<LoanRecord> {
        self.inner.read().await.values().cloned().collect()
    }

    pub async fn get(&self, loan_id: &str) -> Option<LoanRecord> {
        self.inner.read().await.get(loan_id).cloned()
    }

    pub async fn create(&self, loan: LoanRecord) -> Result<(), String> {
        if let Err(errors) = loan.validate() {
            return Err(errors.join("; "));
        }

        let mut guard = self.inner.write().await;
        if guard.contains_key(&loan.loan_id) {
            return Err(format!("Loan with ID {} already exists", loan.loan_id));
        }
        guard.insert(loan.loan_id.clone(), loan);
        drop(guard);
        self.persist().await.map_err(|err| err.to_string())
    }

    pub async fn update(&self, loan_id: &str, loan: LoanRecord) -> Result<(), String> {
        if loan_id != loan.loan_id {
            return Err("Loan ID in path must match request body".into());
        }
        if let Err(errors) = loan.validate() {
            return Err(errors.join("; "));
        }

        let mut guard = self.inner.write().await;
        if !guard.contains_key(loan_id) {
            return Err(format!("Loan with ID {loan_id} not found"));
        }
        guard.insert(loan_id.to_string(), loan);
        drop(guard);
        self.persist().await.map_err(|err| err.to_string())
    }

    pub async fn delete(&self, loan_id: &str) -> Result<bool, String> {
        let mut guard = self.inner.write().await;
        let removed = guard.remove(loan_id).is_some();
        drop(guard);
        if removed {
            self.persist().await.map_err(|err| err.to_string())?;
        }
        Ok(removed)
    }

    async fn persist(&self) -> anyhow::Result<()> {
        let loans: Vec<LoanRecord> = self.inner.read().await.values().cloned().collect();
        let file = LoanFile {
            version: "1.0".into(),
            last_updated: Utc::now().to_rfc3339(),
            loans,
        };

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let tmp_path = self.path.with_extension("json.tmp");
        let payload = serde_json::to_string_pretty(&file)?;
        fs::write(&tmp_path, payload)
            .with_context(|| format!("failed to write {}", tmp_path.display()))?;
        fs::rename(&tmp_path, &self.path)
            .with_context(|| format!("failed to move {} to {}", tmp_path.display(), self.path.display()))?;
        Ok(())
    }
}

fn detect_default_path() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("LOANS_BACKEND_PATH") {
        return Ok(PathBuf::from(path));
    }

    let candidates = [PathBuf::from("agents/backend/data/loans.json"), PathBuf::from("data/loans.json")];

    if let Some(existing) = candidates.iter().find(|path| path.exists()) {
        return Ok(existing.clone());
    }

    let preferred = Path::new("agents/backend/data");
    if preferred.exists() {
        Ok(preferred.join("loans.json"))
    } else {
        Ok(PathBuf::from("data/loans.json"))
    }
}

fn legacy_config_path() -> Option<PathBuf> {
    let path = PathBuf::from("config/loans.json");
    path.exists().then_some(path)
}
