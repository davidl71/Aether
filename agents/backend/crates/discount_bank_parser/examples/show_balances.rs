//! Show account balances from Discount Bank file

use discount_bank_parser::DiscountBankParser;
use std::env;
use std::path::Path;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_discount_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = Path::new(&args[1]);
    let parsed = DiscountBankParser::parse_file(file_path).await?;

    println!("\n=== Account Balances ===\n");

    if parsed.headers.is_empty() {
        println!("No header records found in file.");
        return Ok(());
    }

    // Show all headers with balances
    for (i, header) in parsed.headers.iter().enumerate() {
        println!(
            "Header {} (Date: {}):",
            i + 1,
            header.transaction_date.format("%Y-%m-%d")
        );
        println!(
            "  Account: {}-{}-{}",
            header.branch_number, header.section_number, header.account_number
        );
        println!("  Currency: {}", header.currency_code);

        let opening = if header.opening_sign == '-' {
            -header.opening_balance
        } else {
            header.opening_balance
        };

        let closing = if header.closing_sign == '-' {
            -header.closing_balance
        } else {
            header.closing_balance
        };

        println!(
            "  Opening Balance: {} {}",
            if opening.is_sign_negative() { "-" } else { "" },
            opening.abs()
        );
        println!(
            "  Closing Balance: {} {}",
            if closing.is_sign_negative() { "-" } else { "" },
            closing.abs()
        );
        println!();
    }

    // Show final balance (from last header)
    if let Some(last_header) = parsed.headers.last() {
        let final_balance = if last_header.closing_sign == '-' {
            -last_header.closing_balance
        } else {
            last_header.closing_balance
        };

        println!("=== Final Account Balance ===");
        println!(
            "Account: {}-{}-{}",
            last_header.branch_number, last_header.section_number, last_header.account_number
        );
        println!("Date: {}", last_header.transaction_date.format("%Y-%m-%d"));
        println!(
            "Balance: {} {}",
            if final_balance.is_sign_negative() {
                "-"
            } else {
                ""
            },
            final_balance.abs()
        );
        println!("Currency: {}", last_header.currency_code);
    }

    // Calculate net from transactions
    use rust_decimal::Decimal;
    let mut net_amount = Decimal::ZERO;
    for txn in &parsed.transactions {
        let amount = if txn.debit_credit_sign == '-' {
            -txn.amount
        } else {
            txn.amount
        };
        net_amount += amount;
    }

    println!("\n=== Transaction Summary ===");
    println!("Total Transactions: {}", parsed.transactions.len());
    println!("Net from Transactions: {}", net_amount);

    if let Some(summary) = &parsed.summary {
        println!("Expected Transaction Count: {}", summary.transaction_count);
    }

    Ok(())
}
