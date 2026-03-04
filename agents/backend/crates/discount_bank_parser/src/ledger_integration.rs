//! Ledger integration for Discount Bank transactions
//!
//! Converts parsed Discount Bank records to ledger Transaction objects

use crate::parser::ParsedFile;
use crate::records::{HeaderRecord, TransactionRecord};
use ledger::account::AccountPath;
use ledger::currency::Currency;
use ledger::error::Result as LedgerResult;
use ledger::money::Money;
use ledger::transaction::{Transaction, TransactionBuilder};
use rust_decimal::Decimal;

/// Convert Discount Bank transactions to ledger transactions
///
/// # Arguments
/// * `parsed_file` - Parsed Discount Bank file
/// * `exchange_rate` - Optional exchange rate for ILS → USD conversion (if None, uses ILS)
/// * `account_path_prefix` - Account path prefix (default: "Assets:Bank:Discount")
pub fn convert_to_transactions(
    parsed_file: &ParsedFile,
    exchange_rate: Option<Decimal>,
    account_path_prefix: Option<&str>,
) -> LedgerResult<Vec<Transaction>> {
    let prefix = account_path_prefix.unwrap_or("Assets:Bank:Discount");
    let currency_code = parsed_file.currency_code().unwrap_or("01"); // Default to ILS

    // Parse currency code to Currency enum
    let currency = match currency_code {
        "01" => Currency::ILS,
        "02" => Currency::USD,
        "03" => Currency::EUR,
        _ => Currency::ILS, // Default to ILS
    };

    // Determine target currency (convert to USD if exchange rate provided, otherwise use original)
    let _target_currency = if exchange_rate.is_some() && currency == Currency::ILS {
        Currency::USD
    } else {
        currency
    };

    // Build account path
    let account_number = parsed_file.account_number().unwrap_or("unknown");
    let bank_account = AccountPath::from_string(&format!("{}:{}", prefix, account_number))?;
    let equity_account = AccountPath::from_string("Equity:Capital")?;

    let mut transactions = Vec::new();

    // Group transactions by date (from headers)
    let mut current_header: Option<&HeaderRecord> = None;

    for header in &parsed_file.headers {
        current_header = Some(header);
    }

    // Process each transaction
    for txn_record in &parsed_file.transactions {
        // Determine if this is a deposit (credit) or withdrawal (debit)
        // Discount Bank: "-" sign = debit (money out), space = credit (money in)
        let is_debit = txn_record.debit_credit_sign == '-';
        let amount = if is_debit {
            -txn_record.amount // Negative for debit
        } else {
            txn_record.amount // Positive for credit
        };

        // Convert currency if needed
        let money = if let Some(rate) = exchange_rate {
            // Convert ILS to USD
            let usd_amount = amount * rate;
            Money::new(usd_amount, Currency::USD)
        } else {
            Money::new(amount, currency)
        };

        // Create description
        let description = if is_debit {
            format!("Discount Bank Withdrawal: {}", txn_record.reference)
        } else {
            format!("Discount Bank Deposit: {}", txn_record.reference)
        };

        // Build transaction
        let mut builder = TransactionBuilder::new(description)
            .with_date(txn_record.value_date)
            .with_metadata("source", "discount_bank")
            .with_metadata("reference", &txn_record.reference)
            .with_metadata("account_number", account_number);

        if let Some(header) = current_header {
            builder = builder
                .with_metadata("branch_number", &header.branch_number)
                .with_metadata("section_number", &header.section_number);
        }

        // Create double-entry transaction
        // For deposits (credit): Debit bank account, Credit equity
        // For withdrawals (debit): Debit equity, Credit bank account
        let transaction = if is_debit {
            // Withdrawal: Debit equity, Credit bank (money leaving bank)
            builder
                .debit(equity_account.clone(), money.abs())
                .credit(bank_account.clone(), money.abs())
        } else {
            // Deposit: Debit bank, Credit equity (money entering bank)
            builder
                .debit(bank_account.clone(), money.abs())
                .credit(equity_account.clone(), money.abs())
        }
        .build()?;

        transactions.push(transaction);
    }

    Ok(transactions)
}

/// Convert single transaction record to ledger transaction
pub fn convert_single_transaction(
    txn_record: &TransactionRecord,
    header: Option<&HeaderRecord>,
    account_path: &AccountPath,
    currency: Currency,
    exchange_rate: Option<Decimal>,
) -> LedgerResult<Transaction> {
    let is_debit = txn_record.debit_credit_sign == '-';
    let amount = if is_debit {
        -txn_record.amount
    } else {
        txn_record.amount
    };

    // Convert currency if needed
    let money = if let Some(rate) = exchange_rate {
        let usd_amount = amount * rate;
        Money::new(usd_amount, Currency::USD)
    } else {
        Money::new(amount, currency)
    };

    let description = format!("Discount Bank: {}", txn_record.reference);

    let mut builder = TransactionBuilder::new(description)
        .with_date(txn_record.value_date)
        .with_metadata("source", "discount_bank")
        .with_metadata("reference", &txn_record.reference);

    if let Some(h) = header {
        builder = builder
            .with_metadata("account_number", &h.account_number)
            .with_metadata("branch_number", &h.branch_number);
    }

    let equity_account = AccountPath::from_string("Equity:Capital")?;

    let transaction = if is_debit {
        builder
            .debit(equity_account, money.abs())
            .credit(account_path.clone(), money.abs())
    } else {
        builder
            .debit(account_path.clone(), money.abs())
            .credit(equity_account, money.abs())
    }
    .build()?;

    Ok(transaction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::DiscountBankParser;
    use ledger::account::accounts;

    #[tokio::test]
    async fn test_convert_to_transactions() {
        // Create a simple parsed file
        let content = b"0001234567890123456789012345678901234567890123456789012345\r\n01250118123456789012 1234567                   \r\n";

        let parsed = DiscountBankParser::parse_bytes(content).await.unwrap();
        let transactions = convert_to_transactions(&parsed, None, None).unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].postings.len(), 2);
    }

    #[test]
    fn test_account_path_creation() {
        let path = AccountPath::from_string("Assets:Bank:Discount:123456").unwrap();
        assert_eq!(path.to_string(), "Assets:Bank:Discount:123456");
    }
}
