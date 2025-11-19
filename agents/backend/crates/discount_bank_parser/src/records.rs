//! Record type definitions for Discount Bank reconciliation format

use crate::errors::{ParseError, Result};
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Header record (56 characters, code "00")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderRecord {
    pub bank_number: u8,
    pub branch_number: String,
    pub section_number: String,
    pub currency_code: String,
    pub account_number: String,
    pub opening_balance: Decimal,
    pub opening_sign: char,
    pub closing_balance: Decimal,
    pub closing_sign: char,
    pub transaction_date: DateTime<Utc>,
}

/// Transaction record (47 characters, code "01")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub value_date: DateTime<Utc>,
    pub amount: Decimal,
    pub debit_credit_sign: char,
    pub reference: String,
}

/// Summary record (107 characters, code "04")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryRecord {
    pub bank_number: String,
    pub branch_number: String,
    pub account_number: String,
    pub transaction_count: u64,
}

impl HeaderRecord {
    /// Parse header record from line
    pub fn from_line(line: &str, line_number: usize) -> Result<Self> {
        // Accept 54-56 characters (some files may have different line endings or missing filler)
        if line.len() < 54 {
            return Err(ParseError::InvalidRecordLength {
                expected: 54,
                actual: line.len(),
                line: line_number,
            });
        }

        // Record code (1-2): "00"
        let record_code = &line[0..2];
        if record_code != "00" {
            return Err(ParseError::InvalidRecordCode {
                expected: "00".to_string(),
                actual: record_code.to_string(),
                line: line_number,
            });
        }

        // Bank number (3): "0" or "1"
        let bank_number = line[2..3]
            .parse::<u8>()
            .map_err(|_| ParseError::ParseError {
                line: line_number,
                message: format!("Invalid bank number: {}", &line[2..3]),
            })?;

        // Branch number (4-6): 3 digits
        let branch_number = line[3..6].trim().to_string();

        // Section number (7-10): 4 digits
        let section_number = line[6..10].trim().to_string();

        // Currency code (11-12): 2 characters
        let currency_code = line[10..12].trim().to_string();

        // Account number (13-18): 6 digits
        let account_number = line[12..18].trim().to_string();

        // Opening balance (19-32): 14 digits, last 2 = cents
        let opening_balance_str = line[18..32].trim();
        let opening_balance = parse_amount(opening_balance_str, line_number)?;

        // Opening sign (33): "-" or space
        let opening_sign = line.chars().nth(32).unwrap_or(' ');

        // Closing balance (34-47): 14 digits, last 2 = cents
        let closing_balance_str = line[33..47].trim();
        let closing_balance = parse_amount(closing_balance_str, line_number)?;

        // Closing sign (48): "-" or space
        let closing_sign = line.chars().nth(47).unwrap_or(' ');

        // Transaction date (49-54): YYMMDD (handle lines that are 54-55 chars)
        let date_str = if line.len() >= 54 {
            &line[48..54]
        } else if line.len() >= 50 {
            &line[48..line.len()]
        } else {
            return Err(ParseError::InvalidDate(
                "Date field missing".to_string(),
                line_number,
            ));
        };
        let transaction_date = parse_date(date_str, line_number)?;

        Ok(Self {
            bank_number,
            branch_number,
            section_number,
            currency_code,
            account_number,
            opening_balance,
            opening_sign,
            closing_balance,
            closing_sign,
            transaction_date,
        })
    }
}

impl TransactionRecord {
    /// Parse transaction record from line
    pub fn from_line(line: &str, line_number: usize) -> Result<Self> {
        // Accept 47+ characters (some files may have additional fields)
        if line.len() < 28 {
            return Err(ParseError::InvalidRecordLength {
                expected: 28,
                actual: line.len(),
                line: line_number,
            });
        }

        // Record code (1-2): "01"
        let record_code = &line[0..2];
        if record_code != "01" {
            return Err(ParseError::InvalidRecordCode {
                expected: "01".to_string(),
                actual: record_code.to_string(),
                line: line_number,
            });
        }

        // Value date (3-8): YYMMDD
        let date_str = &line[2..8];
        let value_date = parse_date(date_str, line_number)?;

        // Amount: Handle both formats
        // Standard format: 9-20 (12 digits)
        // Short format: 9-18 (10 digits) - actual file format
        let amount_str = if line.len() >= 20 {
            // Standard format: 12 digits
            line[8..20].trim()
        } else if line.len() >= 18 {
            // Short format: 10 digits (actual file format)
            line[8..18].trim()
        } else {
            return Err(ParseError::InvalidAmount(
                format!("Amount field too short (line length: {})", line.len()),
                line_number,
            ));
        };
        let amount = parse_amount(amount_str, line_number)?;

        // Debit/credit sign: Position varies by format
        let sign_pos = if line.len() >= 20 { 20 } else { 18 };
        let debit_credit_sign = line.chars().nth(sign_pos).unwrap_or(' ');

        // Reference: 7 characters after sign
        let ref_start = sign_pos + 1;
        let ref_end = std::cmp::min(ref_start + 7, line.len());
        let reference = if ref_end > ref_start {
            line[ref_start..ref_end].trim().to_string()
        } else {
            String::new()
        };

        Ok(Self {
            value_date,
            amount,
            debit_credit_sign,
            reference,
        })
    }
}

impl SummaryRecord {
    /// Parse summary record from line
    pub fn from_line(line: &str, line_number: usize) -> Result<Self> {
        if line.len() < 107 {
            return Err(ParseError::InvalidRecordLength {
                expected: 107,
                actual: line.len(),
                line: line_number,
            });
        }

        // Record code (1-2): "04"
        let record_code = &line[0..2];
        if record_code != "04" {
            return Err(ParseError::InvalidRecordCode {
                expected: "04".to_string(),
                actual: record_code.to_string(),
                line: line_number,
            });
        }

        // Bank number (3-6): 4 digits
        let bank_number = line[2..6].trim().to_string();

        // Branch number (7-10): 4 digits
        let branch_number = line[6..10].trim().to_string();

        // Account number (11-20): 10 digits
        let account_number = line[10..20].trim().to_string();

        // Transaction counter (21-31): 11 digits
        let counter_str = line[20..31].trim();
        let transaction_count = counter_str
            .parse::<u64>()
            .map_err(|_| ParseError::ParseError {
                line: line_number,
                message: format!("Invalid transaction count: {}", counter_str),
            })?;

        Ok(Self {
            bank_number,
            branch_number,
            account_number,
            transaction_count,
        })
    }
}

/// Parse amount from integer string (last 2 digits = cents)
fn parse_amount(amount_str: &str, line_number: usize) -> Result<Decimal> {
    let amount_int = amount_str
        .parse::<i64>()
        .map_err(|e| ParseError::InvalidAmount(e.to_string(), line_number))?;

    // Convert to decimal: divide by 100 (last 2 digits = cents)
    let amount_decimal = Decimal::from(amount_int) / Decimal::from(100);
    Ok(amount_decimal)
}

/// Parse date from YYMMDD format
fn parse_date(date_str: &str, line_number: usize) -> Result<DateTime<Utc>> {
    if date_str.len() != 6 {
        return Err(ParseError::InvalidDate(
            format!("Invalid date length: {}", date_str),
            line_number,
        ));
    }

    let year_str = &date_str[0..2];
    let month_str = &date_str[2..4];
    let day_str = &date_str[4..6];

    let year_2digit: u32 = year_str
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidDate(format!("Invalid year: {}", year_str), line_number))?;
    let month: u32 = month_str
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidDate(format!("Invalid month: {}", month_str), line_number))?;
    let day: u32 = day_str
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidDate(format!("Invalid day: {}", day_str), line_number))?;

    // Interpret 2-digit year as 2000s (e.g., "25" = 2025)
    let year = 2000 + year_2digit as i32;

    // Validate month and day
    if month < 1 || month > 12 {
        return Err(ParseError::InvalidDate(
            format!("Invalid month: {}", month),
            line_number,
        ));
    }
    if day < 1 || day > 31 {
        return Err(ParseError::InvalidDate(
            format!("Invalid day: {}", day),
            line_number,
        ));
    }

    Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
        .single()
        .ok_or_else(|| {
            ParseError::InvalidDate(
                format!("Invalid date: {}/{}/{}", year, month, day),
                line_number,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header_record() {
        // Example header record (56 chars)
        let line = "0001234567890123456789012345678901234567890123456789012345";
        let record = HeaderRecord::from_line(line, 1).unwrap();
        assert_eq!(record.bank_number, 0);
        assert_eq!(record.branch_number, "123");
        assert_eq!(record.account_number, "456789");
    }

    #[test]
    fn test_parse_transaction_record() {
        // Example transaction record (47 chars)
        let line = "01250118123456789012 1234567                   ";
        let record = TransactionRecord::from_line(line, 1).unwrap();
        assert_eq!(record.reference, "1234567");
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("251118", 1).unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 18);
    }

    #[test]
    fn test_parse_amount() {
        let amount = parse_amount("123456", 1).unwrap();
        assert_eq!(amount, Decimal::from_str("1234.56").unwrap());
    }
}
