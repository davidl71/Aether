//! Test program for Discount Bank parser
//!
//! Usage: cargo run --example test_parser -- <path_to_discount_file>

use discount_bank_parser::{convert_to_transactions, DiscountBankParser};
use std::env;
use std::path::Path;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Get file path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_discount_file>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  cargo run --example test_parser -- ~/DISCOUNT.DAT");
        std::process::exit(1);
    }

    let file_path = Path::new(&args[1]);

    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        std::process::exit(1);
    }

    info!("Parsing file: {}", file_path.display());

    // Parse the file
    let parsed = DiscountBankParser::parse_file(file_path).await?;

    info!("Parsed file successfully!");
    info!("  Headers: {}", parsed.headers.len());
    info!("  Transactions: {}", parsed.transactions.len());
    info!("  Summary: {:?}", parsed.summary);

    // Validate
    if let Err(e) = parsed.validate() {
        eprintln!("Warning: Validation failed: {}", e);
    } else {
        info!("Validation passed!");
    }

    // Show first header (if any)
    if let Some(header) = parsed.headers.first() {
        info!("\nFirst Header:");
        info!("  Bank Number: {}", header.bank_number);
        info!("  Branch: {}", header.branch_number);
        info!("  Account: {}", header.account_number);
        info!("  Currency: {}", header.currency_code);
        info!("  Opening Balance: {}", header.opening_balance);
        info!("  Closing Balance: {}", header.closing_balance);
        info!("  Date: {}", header.transaction_date);
    }

    // Show first few transactions
    info!("\nFirst 5 Transactions:");
    for (i, txn) in parsed.transactions.iter().take(5).enumerate() {
        info!(
            "  {}: Date={}, Amount={}, Sign={}, Ref={}",
            i + 1,
            txn.value_date.format("%Y-%m-%d"),
            txn.amount,
            txn.debit_credit_sign,
            txn.reference
        );
    }

    // Convert to ledger transactions
    info!("\nConverting to ledger transactions...");
    let ledger_txns = convert_to_transactions(&parsed, None, None)?;

    info!(
        "Converted {} transactions to ledger format",
        ledger_txns.len()
    );

    // Show first ledger transaction
    if let Some(txn) = ledger_txns.first() {
        info!("\nFirst Ledger Transaction:");
        info!("  ID: {}", txn.id);
        info!("  Date: {}", txn.date.format("%Y-%m-%d"));
        info!("  Description: {}", txn.description);
        info!("  Cleared: {}", txn.cleared);
        info!("  Postings: {}", txn.postings.len());
        for (i, posting) in txn.postings.iter().enumerate() {
            info!("    {}: {} {}", i + 1, posting.account, posting.amount);
        }
        info!("  Metadata: {:?}", txn.metadata);
    }

    // Validate all transactions balance
    info!("\nValidating ledger transactions...");
    let mut errors = 0;
    for (i, txn) in ledger_txns.iter().enumerate() {
        if let Err(e) = txn.validate_balance() {
            eprintln!("  Transaction {} failed validation: {}", i + 1, e);
            errors += 1;
        }
    }

    if errors == 0 {
        info!(
            "All {} transactions validated successfully!",
            ledger_txns.len()
        );
    } else {
        eprintln!("Warning: {} transactions failed validation", errors);
    }

    Ok(())
}
