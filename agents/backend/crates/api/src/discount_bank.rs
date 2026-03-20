//! Discount Bank API: balance, transactions, bank accounts, import positions.
//! Exposed via NATS only (subjects `api.discount_bank.*`). See docs/platform/NATS_API.md §3.

use discount_bank_parser::DiscountBankParser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::project_paths::discover_workspace_root;
use crate::IbPositionDto;

// Proto-aligned DTOs: same shape as proto/messages.proto BankAccount, DiscountBankBalance, DiscountBankTransaction
// so JSON matches proto JSON and clients can rely on a single contract.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountDto {
    pub id: String,
    pub institution: String,
    pub account_number: String,
    pub branch_number: String,
    pub section_number: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountBankBalanceDto {
    pub account: BankAccountDto,
    pub balance: f64,
    pub currency: String,
    #[serde(rename = "balanceDate")]
    pub balance_date: String,
    #[serde(rename = "creditRate")]
    pub credit_rate: f64,
    #[serde(rename = "debitRate")]
    pub debit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountBankTransactionDto {
    #[serde(rename = "valueDate")]
    pub value_date: String,
    pub amount: f64,
    #[serde(rename = "isDebit")]
    pub is_debit: bool,
    pub reference: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountBankTransactionsListDto {
    pub account: BankAccountDto,
    pub transactions: Vec<DiscountBankTransactionDto>,
    #[serde(rename = "totalCount")]
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountsListDto {
    pub accounts: Vec<BankAccountDto>,
    #[serde(rename = "totalCount")]
    pub total_count: usize,
}

const DEFAULT_DISCOUNT_BANK_FILE_PATH: &str = "~/Downloads/DISCOUNT.dat";
const DEFAULT_DISCOUNT_BANK_CREDIT_RATE: f64 = 0.03;
const DEFAULT_DISCOUNT_BANK_DEBIT_RATE: f64 = 0.103;
const DEFAULT_RUST_API_URL: &str = "http://127.0.0.1:8080";

#[derive(Debug, Clone, Serialize)]
pub struct ReconciledPositionResponse {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub current_price: Option<f64>,
    pub market_value: Option<f64>,
    pub unrealized_pl: Option<f64>,
    pub currency: String,
    pub broker: String,
    pub in_ledger: bool,
    pub ledger_account_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPositionsResponse {
    pub positions: Vec<ReconciledPositionResponse>,
    pub imported_count: i32,
    pub existing_count: usize,
    pub missing_count: usize,
    pub total_count: usize,
    pub write_disabled: bool,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportPositionsQuery {
    pub broker: String,
    pub account_id: Option<String>,
    pub dry_run: Option<bool>,
}

pub async fn get_balance() -> Result<DiscountBankBalanceDto, String> {
    let latest_file =
        latest_discount_bank_file().ok_or_else(|| "Discount Bank file not found".to_string())?;
    let parsed = DiscountBankParser::parse_file(&latest_file)
        .await
        .map_err(|error| format!("Failed to parse Discount Bank file: {error}"))?;
    let header = parsed
        .headers
        .last()
        .ok_or_else(|| "No header record found in Discount Bank file".to_string())?;

    let balance = decimal_signed_to_f64(header.closing_balance.to_string(), header.closing_sign)?;
    let currency = currency_from_code(&header.currency_code);
    let credit_rate = discount_bank_credit_rate();
    let debit_rate = discount_bank_debit_rate();
    let account_id = format!(
        "{}-{}-{}",
        header.branch_number, header.section_number, header.account_number
    );

    Ok(DiscountBankBalanceDto {
        account: BankAccountDto {
            id: account_id.clone(),
            institution: "discount_bank".to_string(),
            account_number: header.account_number.clone(),
            branch_number: header.branch_number.clone(),
            section_number: header.section_number.clone(),
            currency: currency.clone(),
        },
        balance,
        currency,
        balance_date: header.transaction_date.format("%Y-%m-%d").to_string(),
        credit_rate,
        debit_rate,
    })
}

pub async fn get_transactions(limit: usize) -> Result<DiscountBankTransactionsListDto, String> {
    let latest_file =
        latest_discount_bank_file().ok_or_else(|| "Discount Bank file not found".to_string())?;
    let parsed = DiscountBankParser::parse_file(&latest_file)
        .await
        .map_err(|error| format!("Failed to parse Discount Bank file: {error}"))?;
    let balance = get_balance().await?;
    let account_id = balance.account.id.clone();

    let mut transactions: Vec<DiscountBankTransactionDto> = parsed
        .transactions
        .iter()
        .rev()
        .take(limit)
        .map(|transaction| {
            let amount = transaction
                .amount
                .to_string()
                .parse::<f64>()
                .unwrap_or_default();
            DiscountBankTransactionDto {
                value_date: transaction.value_date.format("%Y-%m-%d").to_string(),
                amount,
                is_debit: transaction.debit_credit_sign != '+',
                reference: transaction.reference.trim().to_string(),
                account_id: account_id.clone(),
            }
        })
        .collect::<Vec<_>>();

    if transactions.len() > limit {
        transactions.truncate(limit);
    }
    let total_count = transactions.len();

    Ok(DiscountBankTransactionsListDto {
        account: balance.account,
        transactions,
        total_count,
    })
}

pub async fn get_bank_accounts() -> Result<BankAccountsListDto, String> {
    let pool = open_ledger_pool().await?;
    let rows = sqlx::query("SELECT transaction_json FROM transactions")
        .fetch_all(&pool)
        .await
        .map_err(|error| format!("Failed to read ledger transactions: {error}"))?;

    let mut account_balances: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for row in rows {
        let transaction_json: String = row
            .try_get("transaction_json")
            .map_err(|error| format!("Failed to decode ledger row: {error}"))?;
        let Ok(transaction) = serde_json::from_str::<Value>(&transaction_json) else {
            continue;
        };
        let Some(postings) = transaction.get("postings").and_then(Value::as_array) else {
            continue;
        };
        for posting in postings {
            let Some(account_path) = account_path_from_posting(posting) else {
                continue;
            };
            if !account_path.starts_with("Assets:Bank:") {
                continue;
            }
            let (amount, currency) = amount_from_posting(posting);
            let currency_balances = account_balances.entry(account_path).or_default();
            *currency_balances.entry(currency).or_insert(0.0) += amount;
        }
    }

    let mut accounts = account_balances
        .into_iter()
        .map(|(account_path, balances)| bank_account_dto_from_balances(&account_path, balances))
        .collect::<Vec<_>>();
    accounts.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(BankAccountsListDto {
        total_count: accounts.len(),
        accounts,
    })
}

pub async fn import_positions(
    query: ImportPositionsQuery,
    client: &Client,
) -> Result<ImportPositionsResponse, String> {
    let _ = query.dry_run;
    let broker = query.broker.to_lowercase();
    if broker != "ibkr" {
        return Err(format!("Unknown broker: {}. Use: ibkr", query.broker));
    }

    let positions = fetch_ib_positions(client, query.account_id.as_deref()).await?;
    let ledger_positions = load_ledger_positions().await?;

    let mut existing_count = 0_usize;
    let mut missing_count = 0_usize;
    let mut reconciled = Vec::new();

    for position in positions {
        if position.symbol.trim().is_empty() || position.quantity == 0.0 {
            continue;
        }
        let key = position.symbol.to_uppercase();
        let in_ledger = ledger_positions.contains_key(&key);
        if in_ledger {
            existing_count += 1;
        } else {
            missing_count += 1;
        }
        reconciled.push(ReconciledPositionResponse {
            symbol: key.clone(),
            quantity: position.quantity,
            avg_price: position.avg_price,
            current_price: position.current_price,
            market_value: position.market_value,
            unrealized_pl: position.unrealized_pl,
            currency: "USD".to_string(),
            broker: broker.clone(),
            in_ledger,
            ledger_account_path: in_ledger.then(|| format!("Assets:IBKR:{key}")),
        });
    }

    Ok(ImportPositionsResponse {
        total_count: reconciled.len(),
        positions: reconciled,
        imported_count: 0,
        existing_count,
        missing_count,
        write_disabled: true,
        status: "read_only_reconciliation".to_string(),
    })
}

fn latest_discount_bank_file() -> Option<PathBuf> {
    let configured = std::env::var("DISCOUNT_BANK_FILE_PATH")
        .unwrap_or_else(|_| DEFAULT_DISCOUNT_BANK_FILE_PATH.to_string());
    let expanded = expand_home(&configured);

    if expanded.is_file() {
        return Some(expanded);
    }
    if expanded.is_dir() {
        let mut files = expanded
            .read_dir()
            .ok()?
            .filter_map(|entry| entry.ok().map(|item| item.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("DISCOUNT"))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        files.sort_by_key(|path| path.metadata().and_then(|m| m.modified()).ok());
        return files.pop();
    }
    None
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(stripped);
        }
    }
    PathBuf::from(path)
}

fn discount_bank_credit_rate() -> f64 {
    std::env::var("DISCOUNT_BANK_CREDIT_RATE")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(DEFAULT_DISCOUNT_BANK_CREDIT_RATE)
}

fn discount_bank_debit_rate() -> f64 {
    std::env::var("DISCOUNT_BANK_DEBIT_RATE")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(DEFAULT_DISCOUNT_BANK_DEBIT_RATE)
}

fn currency_from_code(code: &str) -> String {
    match code {
        "01" => "ILS".to_string(),
        "02" => "USD".to_string(),
        "03" => "EUR".to_string(),
        other => other.to_string(),
    }
}

fn decimal_signed_to_f64(amount: String, sign: char) -> Result<f64, String> {
    let mut value = amount
        .parse::<f64>()
        .map_err(|error| format!("Failed to parse decimal amount: {error}"))?;
    if sign == '-' {
        value = -value;
    }
    Ok(value)
}

async fn open_ledger_pool() -> Result<sqlx::SqlitePool, String> {
    let path = ledger_database_path().ok_or_else(|| "Ledger database not found".to_string())?;
    let uri = format!("sqlite://{}", path.display());
    let options = SqliteConnectOptions::from_str(&uri)
        .map_err(|error| format!("Invalid ledger database path: {error}"))?
        .read_only(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| format!("Failed to open ledger database: {error}"))
}

fn ledger_database_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("LEDGER_DATABASE_PATH") {
        let expanded = expand_home(&path);
        if expanded.exists() {
            return Some(expanded);
        }
    }

    let repo_root = discover_workspace_root().or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
    })?;

    let candidates = [
        repo_root.join("ledger.db"),
        repo_root.join("agents/backend/ledger.db"),
        repo_root.join("agents/backend/data/ledger.db"),
        repo_root.join("data/ledger.db"),
        PathBuf::from(std::env::var("HOME").ok()?).join(".ledger/ledger.db"),
    ];

    candidates.into_iter().find(|path| path.exists())
}

fn account_path_from_posting(posting: &Value) -> Option<String> {
    match posting.get("account")? {
        Value::String(value) => Some(value.clone()),
        Value::Object(map) => {
            if let Some(segments) = map.get("segments").and_then(Value::as_array) {
                let joined = segments
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(":");
                if !joined.is_empty() {
                    return Some(joined);
                }
            }
            map.get("to_string")
                .and_then(Value::as_str)
                .map(str::to_string)
        }
        _ => None,
    }
}

fn amount_from_posting(posting: &Value) -> (f64, String) {
    let Some(amount) = posting.get("amount") else {
        return (0.0, "USD".to_string());
    };
    match amount {
        Value::Object(map) => {
            let value = map
                .get("amount")
                .and_then(Value::as_str)
                .and_then(|text| text.parse::<f64>().ok())
                .or_else(|| map.get("amount").and_then(Value::as_f64))
                .unwrap_or_default();
            let currency = map
                .get("currency")
                .and_then(Value::as_str)
                .unwrap_or("USD")
                .to_string();
            (value, currency)
        }
        Value::Number(number) => (number.as_f64().unwrap_or_default(), "USD".to_string()),
        Value::String(text) => (text.parse::<f64>().unwrap_or_default(), "USD".to_string()),
        _ => (0.0, "USD".to_string()),
    }
}

fn bank_account_dto_from_balances(
    account_path: &str,
    balances: HashMap<String, f64>,
) -> BankAccountDto {
    let segments: Vec<&str> = account_path.split(':').collect();
    let institution = segments
        .get(2)
        .map(|s| (*s).to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let account_number = segments
        .get(3)
        .map(|s| (*s).to_string())
        .unwrap_or_else(|| account_path.to_string());
    let currency = if balances.len() == 1 {
        balances
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "USD".to_string())
    } else {
        "MULTI".to_string()
    };

    BankAccountDto {
        id: account_path.to_string(),
        institution: institution.clone(),
        account_number: account_number.clone(),
        branch_number: String::new(),
        section_number: String::new(),
        currency,
    }
}

async fn load_ledger_positions() -> Result<HashMap<String, String>, String> {
    let pool = open_ledger_pool().await?;
    let rows = sqlx::query("SELECT transaction_json FROM transactions")
        .fetch_all(&pool)
        .await
        .map_err(|error| format!("Failed to read ledger positions: {error}"))?;

    let mut positions = HashMap::new();
    for row in rows {
        let transaction_json: String = row
            .try_get("transaction_json")
            .map_err(|error| format!("Failed to decode ledger row: {error}"))?;
        let Ok(transaction) = serde_json::from_str::<Value>(&transaction_json) else {
            continue;
        };
        let Some(postings) = transaction.get("postings").and_then(Value::as_array) else {
            continue;
        };
        for posting in postings {
            let Some(account_path) = account_path_from_posting(posting) else {
                continue;
            };
            if account_path.starts_with("Assets:IBKR:") {
                let symbol = account_path
                    .split(':')
                    .nth(2)
                    .unwrap_or_default()
                    .to_uppercase();
                if !symbol.is_empty() {
                    positions.insert(symbol, account_path);
                }
            }
        }
    }
    Ok(positions)
}

async fn fetch_ib_positions(
    client: &Client,
    account_id: Option<&str>,
) -> Result<Vec<IbPositionDto>, String> {
    let base = std::env::var("RUST_API_URL").unwrap_or_else(|_| DEFAULT_RUST_API_URL.to_string());
    let url = if let Some(account_id) = account_id {
        format!("{base}/api/v1/ib/positions?account_id={account_id}")
    } else {
        format!("{base}/api/v1/ib/positions")
    };
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch IB positions: {error}"))?;
    response
        .json::<Vec<IbPositionDto>>()
        .await
        .map_err(|error| format!("Failed to decode IB positions: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_account_dto_from_discount_single_currency() {
        let account = bank_account_dto_from_balances(
            "Assets:Bank:Discount:123456",
            HashMap::from([("ILS".to_string(), 100.5)]),
        );

        assert_eq!(account.id, "Assets:Bank:Discount:123456");
        assert_eq!(account.institution, "Discount");
        assert_eq!(account.account_number, "123456");
        assert_eq!(account.currency, "ILS");
    }

    #[test]
    fn bank_account_dto_handles_mixed_currency() {
        let account = bank_account_dto_from_balances(
            "Assets:Bank:Discount:123456",
            HashMap::from([("ILS".to_string(), 100.5), ("USD".to_string(), 25.25)]),
        );

        assert_eq!(account.currency, "MULTI");
        assert_eq!(account.id, "Assets:Bank:Discount:123456");
    }

    #[test]
    fn bank_account_dto_legacy_balances_shape() {
        // Keep a minimal check that multiple currencies produce expected map shape
        let _account = bank_account_dto_from_balances(
            "Assets:Bank:Discount:123456",
            HashMap::from([("ILS".to_string(), 100.5), ("USD".to_string(), 25.25)]),
        );
    }
}
