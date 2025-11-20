# Ledger Core Library Design

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document
**Related:**
- `docs/RESEARCH_FINANCIAL_LEDGER_PLATFORMS.md`
- `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`

## Executive Summary

This document defines the architecture for a lightweight ledger core library integrated into the IB box spread portfolio management system. The library provides double-entry accounting optimized for trading operations with minimal latency (< 1ms transaction recording), multi-currency support, and Ledger CLI-compatible transaction export for reconciliation.

**Key Design Principles:**
1. **Double-entry accounting:** Every transaction has balanced debits and credits
2. **Trading-optimized:** Low-latency transaction recording for real-time trading
3. **Ledger CLI compatible:** Export to `.ledger` format for reconciliation
4. **Rust-native:** Integrated with existing Rust backend architecture
5. **Multi-currency:** Support for USD, ILS, and currency conversion

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Data Models](#data-models)
3. [API Design](#api-design)
4. [Transaction Format](#transaction-format)
5. [Account Structure](#account-structure)
6. [Integration Points](#integration-points)
7. [Performance Requirements](#performance-requirements)
8. [Implementation Plan](#implementation-plan)

---

## Architecture Overview

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    IB Box Spread System                      │
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │  Box Spread  │    │   Position   │    │   Account    │  │
│  │   Strategy   │───▶│   Tracking   │───▶│   Summary    │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                    │                    │          │
│         └────────────────────┼────────────────────┘          │
│                              ▼                                │
│                    ┌──────────────────┐                      │
│                    │  Ledger Core     │                      │
│                    │     Library      │                      │
│                    └──────────────────┘                      │
│                              │                                │
│                    ┌─────────┴─────────┐                     │
│                    ▼                   ▼                     │
│          ┌─────────────┐     ┌─────────────┐                │
│          │ PostgreSQL  │     │   Ledger    │                │
│          │  Database   │     │  Export     │                │
│          └─────────────┘     │  (.ledger)  │                │
│                              └─────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

### Component Structure

**Rust Crate:** `agents/backend/crates/ledger/`

```
ledger/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   ├── transaction.rs      # Transaction data model
│   ├── account.rs          # Account data model
│   ├── posting.rs          # Posting data model (debit/credit)
│   ├── currency.rs         # Currency handling and conversion
│   ├── engine.rs           # Transaction recording engine
│   ├── balance.rs          # Balance calculation
│   ├── persistence.rs      # Database persistence
│   ├── export.rs           # Ledger CLI export
│   └── query.rs            # Transaction querying
└── tests/
    ├── transaction_tests.rs
    ├── balance_tests.rs
    └── integration_tests.rs
```

---

## Data Models

### Transaction

A transaction represents a complete financial operation with multiple postings (debits and credits) that must balance.

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: Uuid,

    /// Transaction date and time
    pub date: DateTime<Utc>,

    /// Transaction description (e.g., "Buy SPY", "Box Spread: SPY 450/460")
    pub description: String,

    /// Transaction cleared status (true = cleared, false = pending)
    pub cleared: bool,

    /// Multiple postings (debits and credits) - must balance
    pub postings: Vec<Posting>,

    /// Transaction metadata (trade_id, strategy, etc.)
    pub metadata: HashMap<String, String>,
}

impl Transaction {
    /// Validate that transaction balances (sum of debits = sum of credits)
    pub fn validate_balance(&self) -> Result<(), LedgerError> {
        let mut total_debits = Money::zero();
        let mut total_credits = Money::zero();

        for posting in &self.postings {
            if posting.amount.amount.is_sign_positive() {
                total_debits = total_debits + posting.amount;
            } else {
                total_credits = total_credits - posting.amount; // Negate to make positive
            }
        }

        // Allow small floating-point differences
        let difference = (total_debits - total_credits).abs();
        if difference.amount > 0.01 {
            return Err(LedgerError::UnbalancedTransaction {
                debits: total_debits,
                credits: total_credits,
                difference,
            });
        }

        Ok(())
    }
}
```

### Posting

A posting represents one side (debit or credit) of a transaction.

```rust
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct Posting {
    /// Account path (e.g., "Assets:IBKR:SPY", "Assets:IBKR:Cash")
    pub account: AccountPath,

    /// Amount and currency (positive = debit, negative = credit)
    pub amount: Money,

    /// Cost basis for investment tracking (e.g., "100 SPY @ $450.00")
    pub cost: Option<Cost>,

    /// Posting metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cost {
    /// Quantity (e.g., 100 shares)
    pub quantity: Decimal,

    /// Unit price (e.g., $450.00 per share)
    pub price: Money,
}
```

### Account Path

Account paths follow Ledger CLI format with hierarchical structure.

```rust
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountPath {
    segments: Vec<String>,
}

impl AccountPath {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    pub fn from_string(path: &str) -> Result<Self, LedgerError> {
        if path.is_empty() {
            return Err(LedgerError::InvalidAccountPath(path.to_string()));
        }

        let segments: Vec<String> = path
            .split(':')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if segments.is_empty() {
            return Err(LedgerError::InvalidAccountPath(path.to_string()));
        }

        Ok(Self { segments })
    }

    pub fn to_string(&self) -> String {
        self.segments.join(":")
    }

    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() > 1 {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        } else {
            None
        }
    }
}

impl FromStr for AccountPath {
    type Err = LedgerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AccountPath::from_string(s)
    }
}
```

### Money

Represents an amount in a specific currency.

```rust
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    /// Amount (always positive, sign handled by posting direction)
    pub amount: Decimal,

    /// Currency code (USD, ILS, etc.)
    pub currency: Currency,
}

impl Money {
    pub fn zero() -> Self {
        Self {
            amount: Decimal::ZERO,
            currency: Currency::USD,
        }
    }

    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency.clone(),
        }
    }
}

impl std::ops::Add for Money {
    type Output = Result<Money, LedgerError>;

    fn add(self, other: Money) -> Result<Money, LedgerError> {
        if self.currency != other.currency {
            return Err(LedgerError::CurrencyMismatch {
                expected: self.currency,
                actual: other.currency,
            });
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }
}

impl std::ops::Sub for Money {
    type Output = Result<Money, LedgerError>;

    fn sub(self, other: Money) -> Result<Money, LedgerError> {
        if self.currency != other.currency {
            return Err(LedgerError::CurrencyMismatch {
                expected: self.currency,
                actual: other.currency,
            });
        }
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}
```

### Currency

Currency enumeration with conversion support.

```rust
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    USD,
    ILS,
    EUR,
    GBP,
}

impl Currency {
    pub fn code(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::ILS => "ILS",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "USD" => Some(Currency::USD),
            "ILS" => Some(Currency::ILS),
            "EUR" => Some(Currency::EUR),
            "GBP" => Some(Currency::GBP),
            _ => None,
        }
    }
}
```

### Account

Account represents a ledger account with balance tracking.

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Account {
    /// Account path (e.g., "Assets:IBKR:SPY")
    pub path: AccountPath,

    /// Current balance (calculated from transactions)
    pub balance: Money,

    /// Account creation time
    pub created_at: DateTime<Utc>,

    /// Account metadata
    pub metadata: HashMap<String, String>,
}

impl Account {
    pub fn new(path: AccountPath, currency: Currency) -> Self {
        Self {
            path,
            balance: Money::zero().with_currency(currency),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
```

---

## API Design

### Ledger Engine

Main API for recording transactions and querying balances.

```rust
use async_trait::async_trait;

pub struct LedgerEngine {
    persistence: Box<dyn PersistenceLayer>,
    balance_cache: Arc<RwLock<HashMap<AccountPath, Money>>>,
}

#[async_trait]
pub trait PersistenceLayer: Send + Sync {
    async fn save_transaction(&self, transaction: &Transaction) -> Result<(), LedgerError>;
    async fn load_transaction(&self, id: &Uuid) -> Result<Option<Transaction>, LedgerError>;
    async fn load_transactions(&self, filter: &TransactionFilter) -> Result<Vec<Transaction>, LedgerError>;
}

impl LedgerEngine {
    /// Create a new ledger engine
    pub fn new(persistence: Box<dyn PersistenceLayer>) -> Self {
        Self {
            persistence,
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a transaction (async for database operations)
    pub async fn record_transaction(&self, transaction: Transaction) -> Result<(), LedgerError> {
        // Validate transaction balances
        transaction.validate_balance()?;

        // Persist transaction
        self.persistence.save_transaction(&transaction).await?;

        // Update balance cache
        self.update_balance_cache(&transaction).await?;

        Ok(())
    }

    /// Get account balance (from cache if available)
    pub async fn get_balance(&self, account: &AccountPath) -> Result<Money, LedgerError> {
        // Check cache first
        if let Some(balance) = self.balance_cache.read().await.get(account) {
            return Ok(balance.clone());
        }

        // Calculate from transactions if not in cache
        let balance = self.calculate_balance(account).await?;

        // Update cache
        self.balance_cache.write().await.insert(account.clone(), balance.clone());

        Ok(balance)
    }

    /// Query transactions with filters
    pub async fn query_transactions(&self, filter: TransactionFilter) -> Result<Vec<Transaction>, LedgerError> {
        self.persistence.load_transactions(&filter).await
    }

    /// Calculate balance from all transactions
    async fn calculate_balance(&self, account: &AccountPath) -> Result<Money, LedgerError> {
        let filter = TransactionFilter {
            account: Some(account.clone()),
            ..Default::default()
        };

        let transactions = self.persistence.load_transactions(&filter).await?;

        let mut balance = Money::zero();
        let mut currency: Option<Currency> = None;

        for transaction in transactions {
            for posting in &transaction.postings {
                if posting.account == *account {
                    if currency.is_none() {
                        currency = Some(posting.amount.currency);
                    }

                    balance = (balance + posting.amount)?;
                }
            }
        }

        Ok(balance)
    }

    /// Update balance cache after transaction
    async fn update_balance_cache(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        let mut cache = self.balance_cache.write().await;

        for posting in &transaction.postings {
            let current_balance = cache
                .get(&posting.account)
                .cloned()
                .unwrap_or_else(|| Money::zero().with_currency(posting.amount.currency));

            let new_balance = (current_balance + posting.amount)?;
            cache.insert(posting.account.clone(), new_balance);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    pub account: Option<AccountPath>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

### Transaction Builder

Helper for constructing transactions with validation.

```rust
pub struct TransactionBuilder {
    date: DateTime<Utc>,
    description: String,
    cleared: bool,
    postings: Vec<Posting>,
    metadata: HashMap<String, String>,
}

impl TransactionBuilder {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            date: Utc::now(),
            description: description.into(),
            cleared: true,
            postings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_date(mut self, date: DateTime<Utc>) -> Self {
        self.date = date;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn add_posting(mut self, posting: Posting) -> Self {
        self.postings.push(posting);
        self
    }

    pub fn debit(mut self, account: impl Into<AccountPath>, amount: Money) -> Self {
        self.postings.push(Posting {
            account: account.into(),
            amount, // Positive amount = debit
            cost: None,
            metadata: HashMap::new(),
        });
        self
    }

    pub fn credit(mut self, account: impl Into<AccountPath>, amount: Money) -> Self {
        let mut money = amount;
        money.amount = -money.amount; // Negative amount = credit
        self.postings.push(Posting {
            account: account.into(),
            amount: money,
            cost: None,
            metadata: HashMap::new(),
        });
        self
    }

    pub fn build(self) -> Result<Transaction, LedgerError> {
        let transaction = Transaction {
            id: Uuid::new_v4(),
            date: self.date,
            description: self.description,
            cleared: self.cleared,
            postings: self.postings,
            metadata: self.metadata,
        };

        transaction.validate_balance()?;
        Ok(transaction)
    }
}
```

---

## Transaction Format

### Ledger CLI Export Format

Transactions exported to `.ledger` files follow Ledger CLI format:

```
2025/11/18 * Buy SPY
    Assets:IBKR:SPY           100 SPY @ $450.00
    Assets:IBKR:Cash       -$45,000.00
    ; trade_id: ORD-12345
    ; strategy: box_spread

2025/11/18 * Box Spread: SPY 450/460 Dec 2025
    Assets:IBKR:BoxSpread:SPY:450:460:20251219    $1000.00
    Assets:IBKR:Cash                              -$1000.00
    ; trade_id: BOX-12345
    ; strategy: box_spread
    ; net_debit: 1000.00

2025/11/18 * Box Spread Expiration: SPY 450/460
    Assets:IBKR:Cash            $1000.00
    Assets:IBKR:BoxSpread:SPY:450:460:20251219    -$1000.00
    ; trade_id: BOX-12345-EXP
    ; strategy: box_spread
```

### Export Implementation

```rust
pub struct LedgerExporter;

impl LedgerExporter {
    pub fn export_transactions(transactions: &[Transaction]) -> String {
        let mut output = String::new();

        // Sort transactions by date
        let mut sorted = transactions.to_vec();
        sorted.sort_by(|a, b| a.date.cmp(&b.date));

        for transaction in sorted {
            output.push_str(&format!(
                "{} {} {}\n",
                Self::format_date(&transaction.date),
                if transaction.cleared { "*" } else { "!" },
                transaction.description
            ));

            for posting in &transaction.postings {
                output.push_str(&format!(
                    "    {:40} {}\n",
                    posting.account.to_string(),
                    Self::format_amount(&posting, &transaction.postings)
                ));
            }

            // Add metadata as comments
            for (key, value) in &transaction.metadata {
                output.push_str(&format!("    ; {}: {}\n", key, value));
            }

            output.push('\n');
        }

        output
    }

    fn format_date(date: &DateTime<Utc>) -> String {
        format!("{}/{}/{}", date.year(), date.month(), date.day())
    }

    fn format_amount(posting: &Posting, all_postings: &[Posting]) -> String {
        let mut result = String::new();

        // Add cost basis if present
        if let Some(cost) = &posting.cost {
            result.push_str(&format!("{} {} @ ", cost.quantity, posting.account.path()));
        }

        // Format amount with currency
        let amount = posting.amount.abs();
        result.push_str(&format!("${:.2}", amount.amount));

        result
    }
}
```

---

## Account Structure

### Chart of Accounts

Accounts follow Ledger CLI hierarchy:

```
Assets
  IBKR
    Cash                    # USD cash balance
    Cash:ILS                # ILS cash balance
    SPY                     # SPY stock positions
    TLT                     # TLT bond positions
    BoxSpread               # Box spread positions
      SPY:450:460:20251219  # Specific box spread
    Options                 # Individual option positions
      SPY:20251219:C:450    # SPY Dec 19 450 Call

Equity
  Capital                   # Initial capital
  RealizedPnL              # Realized gains/losses
  UnrealizedPnL            # Unrealized gains/losses

Expenses
  Commissions              # Trading commissions
  Interest                 # Margin interest

Income
  Dividends                # Dividend income
  Interest                 # Interest income
```

### Account Path Constants

```rust
pub mod accounts {
    use super::AccountPath;

    pub fn ibkr_cash() -> AccountPath {
        AccountPath::from_string("Assets:IBKR:Cash").unwrap()
    }

    pub fn ibkr_cash_ils() -> AccountPath {
        AccountPath::from_string("Assets:IBKR:Cash:ILS").unwrap()
    }

    pub fn ibkr_position(symbol: &str) -> AccountPath {
        AccountPath::from_string(&format!("Assets:IBKR:{}", symbol)).unwrap()
    }

    pub fn ibkr_box_spread(symbol: &str, strike1: i32, strike2: i32, expiry: &str) -> AccountPath {
        AccountPath::from_string(&format!(
            "Assets:IBKR:BoxSpread:{}:{}:{}:{}",
            symbol, strike1, strike2, expiry
        )).unwrap()
    }

    pub fn equity_capital() -> AccountPath {
        AccountPath::from_string("Equity:Capital").unwrap()
    }

    pub fn equity_realized_pnl() -> AccountPath {
        AccountPath::from_string("Equity:RealizedPnL").unwrap()
    }
}
```

---

## Integration Points

### 1. Box Spread Execution

Record box spread transaction when executed:

```rust
// In order_manager.cpp or Rust backend
async fn record_box_spread_transaction(
    ledger: &LedgerEngine,
    spread: &BoxSpreadLeg,
    net_debit: f64,
    trade_id: &str,
) -> Result<(), LedgerError> {
    let transaction = TransactionBuilder::new(
        format!("Box Spread: {} {}/{} {}",
            spread.long_call.symbol,
            spread.long_call.strike,
            spread.short_call.strike,
            spread.long_call.expiry
        )
    )
    .with_metadata("trade_id", trade_id)
    .with_metadata("strategy", "box_spread")
    .with_metadata("net_debit", &net_debit.to_string())
    // Debit: Box spread position (asset)
    .debit(
        accounts::ibkr_box_spread(
            &spread.long_call.symbol,
            spread.long_call.strike as i32,
            spread.short_call.strike as i32,
            &spread.long_call.expiry.to_string(),
        ),
        Money::new(Decimal::from_f64(net_debit).unwrap(), Currency::USD),
    )
    // Credit: Cash (payment)
    .credit(
        accounts::ibkr_cash(),
        Money::new(Decimal::from_f64(net_debit).unwrap(), Currency::USD),
    )
    .build()?;

    ledger.record_transaction(transaction).await
}
```

### 2. Position Updates

Record position changes when positions update:

```rust
// In state.rs or integration layer
async fn record_position_change(
    ledger: &LedgerEngine,
    symbol: &str,
    old_quantity: i32,
    new_quantity: i32,
    price: f64,
    trade_id: &str,
) -> Result<(), LedgerError> {
    let quantity_change = new_quantity - old_quantity;
    let notional = (quantity_change as f64) * price;

    let transaction = TransactionBuilder::new(
        format!("{} {} {}",
            if quantity_change > 0 { "Buy" } else { "Sell" },
            quantity_change.abs(),
            symbol
        )
    )
    .with_metadata("trade_id", trade_id)
    // Debit: Position (if buying)
    // Credit: Position (if selling)
    // Credit: Cash (if buying)
    // Debit: Cash (if selling)
    .build()?;

    ledger.record_transaction(transaction).await
}
```

### 3. Cash Flows

Record deposits and withdrawals:

```rust
async fn record_cash_flow(
    ledger: &LedgerEngine,
    amount: f64,
    currency: Currency,
    description: &str,
) -> Result<(), LedgerError> {
    let transaction = TransactionBuilder::new(description)
    .debit(
        if currency == Currency::ILS {
            accounts::ibkr_cash_ils()
        } else {
            accounts::ibkr_cash()
        },
        Money::new(Decimal::from_f64(amount).unwrap(), currency),
    )
    .credit(
        accounts::equity_capital(),
        Money::new(Decimal::from_f64(amount).unwrap(), currency),
    )
    .build()?;

    ledger.record_transaction(transaction).await
}
```

---

## Performance Requirements

### Transaction Recording

- **Target:** < 1ms per transaction
- **Method:** In-memory balance cache + async database writes
- **Optimization:** Batch database writes for high-frequency scenarios

### Balance Queries

- **Target:** < 0.1ms from cache
- **Method:** In-memory `HashMap<AccountPath, Money>` cache
- **Fallback:** Calculate from database if cache miss

### Concurrent Access

- **Thread-safe:** Use `Arc<RwLock<>>` for balance cache
- **Database:** PostgreSQL handles concurrent writes
- **Transaction ordering:** Use database timestamps for ordering

---

## Implementation Plan

### Phase 1: Core Data Models (T-82)

1. Implement `Transaction`, `Posting`, `AccountPath`, `Money`, `Currency`
2. Implement `TransactionBuilder` helper
3. Unit tests for all data models

### Phase 2: Transaction Engine (T-82)

1. Implement `LedgerEngine` with balance cache
2. Implement transaction validation
3. Implement balance calculation
4. Unit tests for engine

### Phase 3: Persistence (T-83)

1. Implement PostgreSQL persistence layer
2. Implement database schema (migrations)
3. Implement transaction querying
4. Integration tests

### Phase 4: Export (T-83)

1. Implement Ledger CLI export format
2. Test export format compatibility
3. Integration with reconciliation workflow

### Phase 5: Integration (T-84)

1. Integrate with box spread execution
2. Integrate with position tracking
3. Integrate with account summary
4. Integration tests

---

## References

1. Ledger CLI Documentation: https://ledger-cli.org/
2. Ledger CLI GitHub: https://github.com/ledger/ledger
3. Research Document: `docs/RESEARCH_FINANCIAL_LEDGER_PLATFORMS.md`
4. Investment Strategy: `docs/INVESTMENT_STRATEGY_FRAMEWORK.md`

---

**Next Steps:**
1. Implement core data models (T-82)
2. Implement transaction engine (T-82)
3. Implement persistence layer (T-83)
4. Integrate with trading system (T-84)
