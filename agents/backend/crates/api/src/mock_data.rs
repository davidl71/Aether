//! Centralized mock data generation for all backends.
//! Used by backend_service for snapshot seeding and by NATS handlers when real data is unavailable
//! (no Discount Bank file, no loan DB, no FRED API key, no FMP API key).
//!
//! **Mock data sources:** Snapshot (positions, orders, historic, symbols), Loans, Discount Bank,
//! Finance rates (SOFR, Treasury), FMP (income statement, balance sheet, cash flow, quote).
//! Market data stream uses `market_data::MockMarketDataSource` when provider is `mock`.

use chrono::{TimeDelta, Utc};
use market_data::{BalanceSheet, CashFlowStatement, FmpQuote, IncomeStatement};

use crate::discount_bank::{
    BankAccountDto, BankAccountsListDto, DiscountBankBalanceDto, DiscountBankTransactionDto,
    DiscountBankTransactionsListDto,
};
use crate::finance_rates::{
    BenchmarkRateResponse, SofrBenchmarksResponse, SofrOvernightResponse,
    TreasuryBenchmarksResponse,
};
use crate::loans::{LoanRecord, LoanStatus, LoanType};
use crate::state::{
    Alert, CandleSnapshot, HistoricPosition, OrderSnapshot, PositionSnapshot, SymbolSnapshot,
    SystemSnapshot,
};

/// Default symbols used when none provided (European-style indices).
pub const DEFAULT_MOCK_SYMBOLS: &[&str] = &["SPX", "XSP", "NDX"];

/// Seeds a snapshot with mock positions, orders, historic positions, symbol quotes, and alerts.
/// Only fills empty vecs; does not clear existing data. Symbols default to DEFAULT_MOCK_SYMBOLS.
pub fn seed_snapshot(snapshot: &mut SystemSnapshot, symbols: &[String]) {
    let symbols_slice: &[String] = if symbols.is_empty() {
        &DEFAULT_MOCK_SYMBOLS
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>()
    } else {
        symbols
    };

    if snapshot.positions.is_empty() {
        snapshot
            .positions
            .extend(mock_positions(&snapshot.account_id));
    }
    if snapshot.orders.is_empty() {
        snapshot.orders.extend(mock_orders());
    }
    if snapshot.historic.is_empty() {
        snapshot.historic.extend(mock_historic());
    }
    if snapshot.symbols.is_empty() {
        snapshot
            .symbols
            .extend(mock_symbol_snapshots(symbols_slice));
    }
    snapshot.metrics.portal_ok = true;
    snapshot.metrics.tws_ok = true;
    snapshot.metrics.questdb_ok = true;
    snapshot.metrics.nats_ok = true;

    snapshot
        .alerts
        .push(Alert::info("Mock runtime initialised"));
    snapshot
        .alerts
        .push(Alert::info("Waiting for market data updates"));
    while snapshot.alerts.len() > 32 {
        snapshot.alerts.remove(0);
    }
}

/// Mock box-spread scenarios per calendar day, DTE +4 around the money, for TUI scenario explorer.
/// One or two scenarios per symbol with expiration = today + 4 days, strike_center near symbol last.
///
/// NOTE: Stubbed out because `ScenarioDto` does not exist in `runtime_state.rs`.
/// The proto-generated `BoxSpreadScenario` type exists in `nats_adapter::proto::v1` but the domain
/// `ScenarioDto` was never defined. See T-1773933296882755000.
#[allow(dead_code)]
pub fn mock_scenarios(_symbols: &[SymbolSnapshot]) -> Vec<String> {
    // TODO: Re-implement once ScenarioDto is properly defined (T-1773933296882755000)
    Vec::new()
}

fn mock_positions(account_id: &str) -> Vec<PositionSnapshot> {
    vec![
        PositionSnapshot {
            id: "POS-1".into(),
            symbol: "XSP".into(),
            quantity: 2,
            cost_basis: 98.75,
            mark: 101.10,
            unrealized_pnl: 4.7,
            account_id: Some(account_id.to_string()),
            source: None,
        },
        PositionSnapshot {
            id: "POS-2".into(),
            symbol: "SPX".into(),
            quantity: 1,
            cost_basis: 5850.0,
            mark: 5862.50,
            unrealized_pnl: 12.50,
            account_id: Some(account_id.to_string()),
            source: None,
        },
    ]
}

fn mock_orders() -> Vec<OrderSnapshot> {
    let now = Utc::now();
    vec![
        OrderSnapshot {
            id: "ORD-1".into(),
            symbol: "XSP".into(),
            side: "BUY".into(),
            quantity: 2,
            status: "FILLED".into(),
            submitted_at: now - TimeDelta::minutes(30),
        },
        OrderSnapshot {
            id: "ORD-2".into(),
            symbol: "NDX".into(),
            side: "SELL".into(),
            quantity: 1,
            status: "SUBMITTED".into(),
            submitted_at: now - TimeDelta::minutes(5),
        },
    ]
}

fn mock_historic() -> Vec<HistoricPosition> {
    let now = Utc::now();
    vec![
        HistoricPosition {
            id: "POS-0".into(),
            symbol: "SPY".into(),
            quantity: 2,
            realized_pnl: 6.2,
            closed_at: now - TimeDelta::hours(5),
        },
        HistoricPosition {
            id: "POS-3".into(),
            symbol: "QQQ".into(),
            quantity: 1,
            realized_pnl: -2.1,
            closed_at: now - TimeDelta::hours(24),
        },
    ]
}

fn mock_symbol_snapshots(symbols: &[String]) -> Vec<SymbolSnapshot> {
    let now = Utc::now();
    let baselines: std::collections::HashMap<&str, f64> = [
        ("SPX", 5860.0),
        ("XSP", 101.0),
        ("NDX", 20850.0),
        ("SPY", 509.0),
        ("QQQ", 445.0),
    ]
    .into_iter()
    .collect();

    symbols
        .iter()
        .map(|s| {
            let last = baselines.get(s.as_str()).copied().unwrap_or(100.0);
            let spread = 0.05;
            let bid = last - spread / 2.0;
            let ask = last + spread / 2.0;
            SymbolSnapshot {
                symbol: s.clone(),
                last,
                bid,
                ask,
                spread,
                roi: 0.5,
                maker_count: 1,
                taker_count: 0,
                volume: 1000,
                candle: CandleSnapshot {
                    open: last - 0.2,
                    high: last + 0.3,
                    low: last - 0.4,
                    close: last,
                    volume: 5000,
                    entry: last - 0.1,
                    updated: now,
                },
            }
        })
        .collect()
}

// ---- Loans ----

/// Returns a fixed list of mock loans for NATS api.loans.list when no repository is configured.
pub fn mock_loans_list() -> Vec<LoanRecord> {
    vec![
        mock_loan_shir("loan-mock-1", "Discount", "123456789", 100_000.0, 4.0),
        mock_loan_cpi("loan-mock-2", "Leumi", "987654321", 50_000.0, 3.5),
    ]
}

/// Single mock SHIR-based loan.
pub fn mock_loan_shir(
    loan_id: &str,
    bank_name: &str,
    account_number: &str,
    principal: f64,
    interest_rate: f64,
) -> LoanRecord {
    LoanRecord {
        loan_id: loan_id.into(),
        bank_name: bank_name.into(),
        account_number: account_number.into(),
        loan_type: LoanType::ShirBased,
        principal,
        original_principal: principal,
        interest_rate,
        spread: 0.5,
        base_cpi: 0.0,
        current_cpi: 0.0,
        origination_date: "2025-01-01T00:00:00Z".into(),
        maturity_date: "2030-01-01T00:00:00Z".into(),
        next_payment_date: "2025-02-01T00:00:00Z".into(),
        monthly_payment: principal * 0.008,
        payment_frequency_months: 1,
        status: LoanStatus::Active,
        last_update: Utc::now().to_rfc3339(),
    }
}

/// Single mock CPI-linked loan.
pub fn mock_loan_cpi(
    loan_id: &str,
    bank_name: &str,
    account_number: &str,
    principal: f64,
    interest_rate: f64,
) -> LoanRecord {
    LoanRecord {
        loan_id: loan_id.into(),
        bank_name: bank_name.into(),
        account_number: account_number.into(),
        loan_type: LoanType::CpiLinked,
        principal,
        original_principal: principal,
        interest_rate,
        spread: 0.25,
        base_cpi: 100.0,
        current_cpi: 102.5,
        origination_date: "2024-06-01T00:00:00Z".into(),
        maturity_date: "2029-06-01T00:00:00Z".into(),
        next_payment_date: "2025-02-01T00:00:00Z".into(),
        monthly_payment: principal * 0.007,
        payment_frequency_months: 1,
        status: LoanStatus::Active,
        last_update: Utc::now().to_rfc3339(),
    }
}

// ---- Discount Bank ----

/// Mock balance for NATS api.discount_bank.balance when no file is present.
pub fn mock_discount_bank_balance() -> DiscountBankBalanceDto {
    DiscountBankBalanceDto {
        account: mock_bank_account(),
        balance: 125_000.0,
        currency: "ILS".into(),
        balance_date: Utc::now().format("%Y-%m-%d").to_string(),
        credit_rate: 0.03,
        debit_rate: 0.103,
    }
}

/// Mock transactions list for NATS api.discount_bank.transactions.
pub fn mock_discount_bank_transactions(limit: usize) -> DiscountBankTransactionsListDto {
    let account = mock_bank_account();
    let transactions = vec![
        DiscountBankTransactionDto {
            value_date: Utc::now().format("%Y-%m-%d").to_string(),
            amount: 500.0,
            is_debit: false,
            reference: "MOCK-DEP-001".into(),
            account_id: account.id.clone(),
        },
        DiscountBankTransactionDto {
            value_date: (Utc::now() - TimeDelta::days(1))
                .format("%Y-%m-%d")
                .to_string(),
            amount: 200.0,
            is_debit: true,
            reference: "MOCK-WD-001".into(),
            account_id: account.id.clone(),
        },
    ];
    let total_count = transactions.len();
    let transactions: Vec<_> = transactions.into_iter().take(limit).collect();
    DiscountBankTransactionsListDto {
        account,
        total_count,
        transactions,
    }
}

/// Mock bank accounts for NATS api.discount_bank.bank_accounts.
pub fn mock_discount_bank_accounts() -> BankAccountsListDto {
    BankAccountsListDto {
        accounts: vec![mock_bank_account()],
        total_count: 1,
    }
}

fn mock_bank_account() -> BankAccountDto {
    BankAccountDto {
        id: "mock-account-1".into(),
        institution: "Discount Bank".into(),
        account_number: "123456".into(),
        branch_number: "001".into(),
        section_number: "00".into(),
        currency: "ILS".into(),
    }
}

// ---- Finance rates (SOFR / Treasury) ----

/// Mock SOFR benchmarks for NATS api.finance_rates.sofr when FRED is unavailable.
pub fn mock_sofr_benchmarks() -> SofrBenchmarksResponse {
    SofrBenchmarksResponse {
        overnight: SofrOvernightResponse {
            rate: Some(4.57),
            timestamp: Some(Utc::now().to_rfc3339()),
        },
        term_rates: vec![
            BenchmarkRateResponse {
                tenor: "1M".into(),
                rate: 4.62,
                days_to_expiry: Some(30),
                timestamp: Utc::now().to_rfc3339(),
                source: "mock".into(),
            },
            BenchmarkRateResponse {
                tenor: "3M".into(),
                rate: 4.75,
                days_to_expiry: Some(90),
                timestamp: Utc::now().to_rfc3339(),
                source: "mock".into(),
            },
        ],
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Mock Treasury benchmarks for NATS api.finance_rates.treasury when FRED is unavailable.
pub fn mock_treasury_benchmarks() -> TreasuryBenchmarksResponse {
    TreasuryBenchmarksResponse {
        rates: vec![
            BenchmarkRateResponse {
                tenor: "4-Week".into(),
                rate: 4.52,
                days_to_expiry: Some(28),
                timestamp: Utc::now().to_rfc3339(),
                source: "mock".into(),
            },
            BenchmarkRateResponse {
                tenor: "13-Week".into(),
                rate: 4.68,
                days_to_expiry: Some(91),
                timestamp: Utc::now().to_rfc3339(),
                source: "mock".into(),
            },
        ],
        timestamp: Utc::now().to_rfc3339(),
    }
}

// ---- FMP fundamentals (when FMP_API_KEY unset) ----

/// Mock income statement for NATS api.fmp.income_statement when FMP is unavailable.
pub fn mock_fmp_income_statement(symbol: &str, limit: u32) -> Vec<IncomeStatement> {
    let sym = if symbol.is_empty() { "MOCK" } else { symbol };
    (0..limit.min(4))
        .map(|i| IncomeStatement {
            symbol: sym.to_string(),
            date: (Utc::now() - TimeDelta::days(365 * i as i64))
                .format("%Y-%m-%d")
                .to_string(),
            revenue: Some(100_000_000.0 * (1.0 - 0.05 * i as f64)),
            gross_profit: Some(60_000_000.0),
            operating_income: Some(25_000_000.0),
            net_income: Some(18_000_000.0),
            eps: Some(2.5),
            eps_diluted: Some(2.45),
        })
        .collect()
}

/// Mock balance sheet for NATS api.fmp.balance_sheet when FMP is unavailable.
pub fn mock_fmp_balance_sheet(symbol: &str, limit: u32) -> Vec<BalanceSheet> {
    let sym = if symbol.is_empty() { "MOCK" } else { symbol };
    (0..limit.min(4))
        .map(|i| BalanceSheet {
            symbol: sym.to_string(),
            date: (Utc::now() - TimeDelta::days(365 * i as i64))
                .format("%Y-%m-%d")
                .to_string(),
            total_assets: Some(500_000_000.0),
            total_liabilities: Some(200_000_000.0),
            total_stockholders_equity: Some(300_000_000.0),
            cash_and_cash_equivalents: Some(50_000_000.0),
            total_debt: Some(80_000_000.0),
        })
        .collect()
}

/// Mock cash flow for NATS api.fmp.cash_flow when FMP is unavailable.
pub fn mock_fmp_cash_flow(symbol: &str, limit: u32) -> Vec<CashFlowStatement> {
    let sym = if symbol.is_empty() { "MOCK" } else { symbol };
    (0..limit.min(4))
        .map(|i| CashFlowStatement {
            symbol: sym.to_string(),
            date: (Utc::now() - TimeDelta::days(365 * i as i64))
                .format("%Y-%m-%d")
                .to_string(),
            operating_cash_flow: Some(30_000_000.0),
            capital_expenditure: Some(-5_000_000.0),
            free_cash_flow: Some(25_000_000.0),
            dividends_paid: Some(-3_000_000.0),
        })
        .collect()
}

/// Mock quote for NATS api.fmp.quote when FMP is unavailable.
pub fn mock_fmp_quote(symbol: &str) -> FmpQuote {
    let sym = if symbol.is_empty() { "MOCK" } else { symbol };
    FmpQuote {
        symbol: sym.to_string(),
        price: Some(100.0),
        open: Some(99.5),
        day_high: Some(101.0),
        day_low: Some(99.0),
        volume: Some(1_000_000),
        previous_close: Some(99.0),
    }
}
