//! CSV import for Discount Bank transactions.
//!
//! This is an optional path for workflows that export bank entries to CSV and
//! want to ingest/reconcile them via the same ledger conversion pipeline as the
//! fixed-width `DISCOUNT*.dat` parser.

use crate::errors::{ParseError, Result};
use crate::ledger_integration::convert_single_transaction;
use crate::records::{HeaderRecord, TransactionRecord};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use csv::Trim;
use ledger::account::AccountPath;
use ledger::currency::Currency;
use ledger::error::{LedgerError, Result as LedgerResult};
use ledger::transaction::Transaction;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// One Discount Bank transaction row as exported to CSV.
///
/// Expected headers (snake_case):
/// - `value_date` (YYYY-MM-DD)
/// - `amount` (decimal, positive number)
/// - `is_debit` (bool; `true` means withdrawal / money out)
/// - `reference` (string)
/// - `account_number` (string)
/// - `branch_number` (optional string)
/// - `section_number` (optional string)
/// - `currency_code` (optional string; "01" ILS, "02" USD, "03" EUR)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscountBankCsvTransactionRow {
    pub value_date: String,
    pub amount: Decimal,
    pub is_debit: bool,
    pub reference: String,
    pub account_number: String,

    #[serde(default)]
    pub branch_number: String,
    #[serde(default)]
    pub section_number: String,
    #[serde(default)]
    pub currency_code: String,
}

pub fn convert_csv_to_transactions(
    text: &str,
    exchange_rate: Option<Decimal>,
    account_path_prefix: Option<&str>,
) -> LedgerResult<Vec<Transaction>> {
    let rows = parse_discount_bank_transactions_csv(text)
        .map_err(|e| LedgerError::Persistence(anyhow::anyhow!(e.to_string())))?;

    let prefix = account_path_prefix.unwrap_or("Assets:Bank:Discount");
    let mut out = Vec::new();

    for (idx, row) in rows.into_iter().enumerate() {
        let line = idx + 1;

        let value_date = parse_ymd_date(&row.value_date, line)
            .map_err(|e| LedgerError::Persistence(anyhow::anyhow!(e.to_string())))?;
        let amount = if row.is_debit { -row.amount } else { row.amount };

        let currency = currency_from_code(row.currency_code.as_str());
        let header = header_from_row(&row, value_date);

        let txn = TransactionRecord {
            value_date,
            amount,
            debit_credit_sign: if row.is_debit { '-' } else { ' ' },
            reference: row.reference,
        };

        let bank_account = AccountPath::from_string(&format!("{prefix}:{}", row.account_number))?;
        let ledger_txn = convert_single_transaction(&txn, Some(&header), &bank_account, currency, exchange_rate)?;
        out.push(ledger_txn);
    }

    Ok(out)
}

fn parse_discount_bank_transactions_csv(text: &str) -> Result<Vec<DiscountBankCsvTransactionRow>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(text.as_bytes());

    let mut out = Vec::new();
    for (index, row) in reader.deserialize::<DiscountBankCsvTransactionRow>().enumerate() {
        let line = index + 1;
        let row = row.map_err(|e| ParseError::ParseError {
            line,
            message: format!("CSV row decode failed: {e}"),
        })?;

        if row.account_number.trim().is_empty() {
            return Err(ParseError::ParseError {
                line,
                message: "account_number is required".to_string(),
            });
        }
        if row.value_date.trim().is_empty() {
            return Err(ParseError::ParseError {
                line,
                message: "value_date is required".to_string(),
            });
        }

        out.push(row);
    }

    if out.is_empty() {
        return Err(ParseError::ParseError {
            line: 0,
            message: "CSV contained no data rows".to_string(),
        });
    }

    Ok(out)
}

fn parse_ymd_date(text: &str, line: usize) -> Result<chrono::DateTime<Utc>> {
    let date = NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").map_err(|e| {
        ParseError::InvalidDate(format!("invalid YYYY-MM-DD date {text:?}: {e}"), line)
    })?;
    Ok(Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0).single().ok_or_else(
        || ParseError::InvalidDate(format!("invalid date {text:?}"), line),
    )?)
}

fn currency_from_code(code: &str) -> Currency {
    match code.trim() {
        "01" | "" => Currency::ILS,
        "02" => Currency::USD,
        "03" => Currency::EUR,
        _ => Currency::ILS,
    }
}

fn header_from_row(row: &DiscountBankCsvTransactionRow, transaction_date: chrono::DateTime<Utc>) -> HeaderRecord {
    HeaderRecord {
        bank_number: 0,
        branch_number: row.branch_number.clone(),
        section_number: row.section_number.clone(),
        currency_code: if row.currency_code.trim().is_empty() {
            "01".to_string()
        } else {
            row.currency_code.clone()
        },
        account_number: row.account_number.clone(),
        opening_balance: Decimal::ZERO,
        opening_sign: ' ',
        closing_balance: Decimal::ZERO,
        closing_sign: ' ',
        transaction_date,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rows_and_converts_to_ledger_transactions() {
        let csv = r#"value_date,amount,is_debit,reference,account_number,branch_number,section_number,currency_code
2026-03-30,123.45,false,SALARY,123456,234,5670,01
2026-03-31,10.00,true,ATM,123456,234,5670,01
"#;

        let txns = convert_csv_to_transactions(csv, None, None).unwrap();
        assert_eq!(txns.len(), 2);
        assert_eq!(txns[0].postings.len(), 2);
        assert_eq!(txns[1].postings.len(), 2);
    }

    #[test]
    fn rejects_empty_csv() {
        let err = parse_discount_bank_transactions_csv("value_date,amount,is_debit,reference,account_number\n")
            .unwrap_err();
        assert!(err.to_string().contains("no data rows"));
    }
}

