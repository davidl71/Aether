//! Ledger integration helpers for trading operations
//!
//! Provides helper functions for recording common trading transactions
//! like position changes, box spreads, and cash flows.

use crate::account::accounts;
use crate::currency::Currency;
use crate::engine::LedgerEngine;
use crate::error::Result;
use crate::money::Money;
use crate::transaction::TransactionBuilder;
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{error, warn};

/// Record a position change transaction (buy or sell)
///
/// This records a double-entry transaction for buying or selling a position.
/// For buys: Debit position account, Credit cash account
/// For sells: Debit cash account, Credit position account
pub async fn record_position_change(
    ledger: Arc<LedgerEngine>,
    symbol: &str,
    quantity: i32,
    price: f64,
    currency: Currency,
    trade_id: Option<&str>,
) -> Result<()> {
    let notional = (quantity.abs() as f64) * price;
    let notional_decimal = Decimal::try_from(notional).map_err(|_| {
        crate::error::LedgerError::InvalidDecimal(format!("Invalid notional: {}", notional))
    })?;
    let amount = Money::new(notional_decimal, currency);

    let description = if quantity > 0 {
        format!("Buy {} {}", quantity, symbol)
    } else {
        format!("Sell {} {}", quantity.abs(), symbol)
    };

    let mut builder = TransactionBuilder::new(description).with_date(Utc::now());

    if let Some(trade_id) = trade_id {
        builder = builder.with_metadata("trade_id", trade_id);
    }
    builder = builder.with_metadata("symbol", symbol);
    builder = builder.with_metadata("quantity", quantity.to_string());

    let position_account = accounts::ibkr_position(symbol);
    let cash_account = accounts::ibkr_cash();

    if quantity > 0 {
        // Buying: Debit position, Credit cash
        builder = builder
            .debit(position_account, amount.clone())
            .credit(cash_account, amount);
    } else {
        // Selling: Debit cash, Credit position (with cost basis)
        builder = builder
            .debit(cash_account, amount.clone())
            .credit(position_account, amount);
    }

    let transaction = builder.build()?;
    ledger.record_transaction(transaction).await
}

/// Record a box spread transaction
///
/// Records a double-entry transaction for executing a box spread:
/// - Debit: Box spread position account (asset)
/// - Credit: Cash account (payment)
#[allow(clippy::too_many_arguments)]
pub async fn record_box_spread(
    ledger: Arc<LedgerEngine>,
    symbol: &str,
    strike1: i32,
    strike2: i32,
    expiry: &str,
    net_debit: f64,
    trade_id: Option<&str>,
    currency: Currency,
) -> Result<()> {
    let net_debit_decimal = Decimal::try_from(net_debit).map_err(|_| {
        crate::error::LedgerError::InvalidDecimal(format!("Invalid net_debit: {}", net_debit))
    })?;
    let amount = Money::new(net_debit_decimal, currency);

    let description = format!("Box Spread: {} {}/{} {}", symbol, strike1, strike2, expiry);

    let mut builder = TransactionBuilder::new(description)
        .with_date(Utc::now())
        .with_metadata("strategy", "box_spread")
        .with_metadata("symbol", symbol)
        .with_metadata("strike1", strike1.to_string())
        .with_metadata("strike2", strike2.to_string())
        .with_metadata("expiry", expiry)
        .with_metadata("net_debit", net_debit.to_string());

    if let Some(trade_id) = trade_id {
        builder = builder.with_metadata("trade_id", trade_id);
    }

    let box_spread_account = accounts::ibkr_box_spread(symbol, strike1, strike2, expiry);
    let cash_account = accounts::ibkr_cash();

    // Debit: Box spread position (asset)
    // Credit: Cash (payment)
    let transaction = builder
        .debit(box_spread_account, amount.clone())
        .credit(cash_account, amount)
        .build()?;

    ledger.record_transaction(transaction).await
}

/// Record box spread expiration/closure
///
/// Records the expiration of a box spread, returning cash:
/// - Debit: Cash account (receipt)
/// - Credit: Box spread position account (removal)
#[allow(clippy::too_many_arguments)]
pub async fn record_box_spread_expiration(
    ledger: Arc<LedgerEngine>,
    symbol: &str,
    strike1: i32,
    strike2: i32,
    expiry: &str,
    payout: f64,
    trade_id: Option<&str>,
    currency: Currency,
) -> Result<()> {
    let payout_decimal = Decimal::try_from(payout).map_err(|_| {
        crate::error::LedgerError::InvalidDecimal(format!("Invalid payout: {}", payout))
    })?;
    let amount = Money::new(payout_decimal, currency);

    let description = format!(
        "Box Spread Expiration: {} {}/{} {}",
        symbol, strike1, strike2, expiry
    );

    let mut builder = TransactionBuilder::new(description)
        .with_date(Utc::now())
        .with_metadata("strategy", "box_spread")
        .with_metadata("event", "expiration")
        .with_metadata("symbol", symbol)
        .with_metadata("strike1", strike1.to_string())
        .with_metadata("strike2", strike2.to_string())
        .with_metadata("expiry", expiry)
        .with_metadata("payout", payout.to_string());

    if let Some(trade_id) = trade_id {
        builder = builder.with_metadata("trade_id", trade_id);
    }

    let box_spread_account = accounts::ibkr_box_spread(symbol, strike1, strike2, expiry);
    let cash_account = accounts::ibkr_cash();

    // Debit: Cash (receipt of payout)
    // Credit: Box spread position (removal)
    let transaction = builder
        .debit(cash_account, amount.clone())
        .credit(box_spread_account, amount)
        .build()?;

    ledger.record_transaction(transaction).await
}

/// Record a cash flow transaction (deposit or withdrawal)
///
/// Records a deposit or withdrawal:
/// - Deposit: Debit cash, Credit equity capital
/// - Withdrawal: Debit equity capital, Credit cash
pub async fn record_cash_flow(
    ledger: Arc<LedgerEngine>,
    amount: f64,
    currency: Currency,
    description: &str,
    is_deposit: bool,
) -> Result<()> {
    let amount_decimal = Decimal::try_from(amount).map_err(|_| {
        crate::error::LedgerError::InvalidDecimal(format!("Invalid amount: {}", amount))
    })?;
    let money = Money::new(amount_decimal, currency);

    let cash_account = if currency == Currency::ILS {
        accounts::ibkr_cash_ils()
    } else {
        accounts::ibkr_cash()
    };
    let capital_account = accounts::equity_capital();

    let transaction = if is_deposit {
        // Deposit: Debit cash, Credit equity
        TransactionBuilder::new(format!("Deposit: {}", description))
            .with_date(Utc::now())
            .with_metadata("event", "deposit")
            .debit(cash_account, money.clone())
            .credit(capital_account, money)
    } else {
        // Withdrawal: Debit equity, Credit cash
        TransactionBuilder::new(format!("Withdrawal: {}", description))
            .with_date(Utc::now())
            .with_metadata("event", "withdrawal")
            .debit(capital_account, money.clone())
            .credit(cash_account, money)
    }
    .build()?;

    ledger.record_transaction(transaction).await
}

/// Record a position close with realized PnL
///
/// Records the closing of a position with realized profit/loss:
/// - Debit: Cash (proceeds from sale)
/// - Credit: Position (cost basis)
/// - Credit/Debit: Equity:RealizedPnL (profit or loss)
pub async fn record_position_close(
    ledger: Arc<LedgerEngine>,
    symbol: &str,
    quantity: i32,
    cost_basis: f64,
    sale_price: f64,
    currency: Currency,
    trade_id: Option<&str>,
) -> Result<()> {
    let proceeds = (quantity.abs() as f64) * sale_price;
    let cost = (quantity.abs() as f64) * cost_basis;
    let pnl = proceeds - cost;

    let proceeds_decimal = Decimal::try_from(proceeds).map_err(|_| {
        crate::error::LedgerError::InvalidDecimal(format!("Invalid proceeds: {}", proceeds))
    })?;
    let cost_decimal = Decimal::try_from(cost).map_err(|_| {
        crate::error::LedgerError::InvalidDecimal(format!("Invalid cost: {}", cost))
    })?;
    let pnl_decimal = Decimal::try_from(pnl)
        .map_err(|_| crate::error::LedgerError::InvalidDecimal(format!("Invalid pnl: {}", pnl)))?;

    let proceeds_money = Money::new(proceeds_decimal, currency);
    let cost_money = Money::new(cost_decimal, currency);
    let pnl_money = Money::new(pnl_decimal.abs(), currency);

    let description = format!(
        "Close Position: {} {} @ {}",
        quantity.abs(),
        symbol,
        sale_price
    );

    let mut builder = TransactionBuilder::new(description)
        .with_date(Utc::now())
        .with_metadata("symbol", symbol)
        .with_metadata("quantity", quantity.abs().to_string())
        .with_metadata("realized_pnl", pnl.to_string());

    if let Some(trade_id) = trade_id {
        builder = builder.with_metadata("trade_id", trade_id);
    }

    let position_account = accounts::ibkr_position(symbol);
    let cash_account = accounts::ibkr_cash();
    let pnl_account = accounts::equity_realized_pnl();

    // Debit: Cash (proceeds from sale)
    // Credit: Position (cost basis)
    // Credit/Debit: Equity:RealizedPnL (if profit: debit, if loss: credit)
    let transaction = if pnl >= 0.0 {
        // Profit: Debit PnL account (increases equity)
        builder
            .debit(cash_account, proceeds_money)
            .credit(position_account, cost_money.clone())
            .debit(pnl_account, pnl_money)
            .build()?
    } else {
        // Loss: Credit PnL account (decreases equity)
        builder
            .debit(cash_account, proceeds_money)
            .credit(position_account, cost_money.clone())
            .credit(pnl_account, pnl_money)
            .build()?
    };

    ledger.record_transaction(transaction).await
}

/// Record transaction with error logging (non-blocking)
///
/// Records a transaction but logs errors instead of propagating them.
/// This allows position tracking to continue even if ledger recording fails.
pub async fn record_transaction_safe(
    ledger: Arc<LedgerEngine>,
    transaction: crate::transaction::Transaction,
) {
    match ledger.record_transaction(transaction).await {
        Ok(()) => {}
        Err(err) => {
            error!(error = %err, "Failed to record ledger transaction (non-blocking)");
        }
    }
}

/// Record position change with error logging (non-blocking)
pub async fn record_position_change_safe(
    ledger: Arc<LedgerEngine>,
    symbol: &str,
    quantity: i32,
    price: f64,
    currency: Currency,
    trade_id: Option<&str>,
) {
    match record_position_change(ledger, symbol, quantity, price, currency, trade_id).await {
        Ok(()) => {}
        Err(err) => {
            warn!(error = %err, symbol, quantity, price, "Failed to record position change in ledger (non-blocking)");
        }
    }
}

/// Record box spread with error logging (non-blocking)
#[allow(clippy::too_many_arguments)]
pub async fn record_box_spread_safe(
    ledger: Arc<LedgerEngine>,
    symbol: &str,
    strike1: i32,
    strike2: i32,
    expiry: &str,
    net_debit: f64,
    trade_id: Option<&str>,
    currency: Currency,
) {
    match record_box_spread(
        ledger, symbol, strike1, strike2, expiry, net_debit, trade_id, currency,
    )
    .await
    {
        Ok(()) => {}
        Err(err) => {
            warn!(error = %err, symbol, strike1, strike2, "Failed to record box spread in ledger (non-blocking)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{LedgerEngine, PersistenceLayer, TransactionFilter};
    use crate::transaction::Transaction;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use uuid::Uuid;

    // Mock persistence layer for testing
    struct MockPersistence {
        transactions: Arc<RwLock<Vec<Transaction>>>,
    }

    #[async_trait]
    impl PersistenceLayer for MockPersistence {
        async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
            self.transactions.write().await.push(transaction.clone());
            Ok(())
        }

        async fn load_transaction(&self, id: &Uuid) -> Result<Option<Transaction>> {
            let transactions = self.transactions.read().await;
            Ok(transactions.iter().find(|t| t.id == *id).cloned())
        }

        async fn load_transactions(&self, filter: &TransactionFilter) -> Result<Vec<Transaction>> {
            let transactions = self.transactions.read().await;
            let mut result = transactions.clone();

            // Apply filters
            if let Some(ref account) = filter.account {
                result.retain(|t| t.postings.iter().any(|p| p.account == *account));
            }

            if let Some(ref desc) = filter.description {
                result.retain(|t| t.description.contains(desc));
            }

            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_record_position_change_buy() {
        let persistence = Arc::new(MockPersistence {
            transactions: Arc::new(RwLock::new(Vec::new())),
        });
        let engine = Arc::new(LedgerEngine::new(persistence.clone()));

        record_position_change(
            engine.clone(),
            "SPY",
            100,
            450.0,
            Currency::USD,
            Some("ORD-12345"),
        )
        .await
        .unwrap();

        let transactions = persistence.transactions.read().await;
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].description, "Buy 100 SPY");
        assert_eq!(transactions[0].postings.len(), 2);
    }

    #[tokio::test]
    async fn test_record_box_spread() {
        let persistence = Arc::new(MockPersistence {
            transactions: Arc::new(RwLock::new(Vec::new())),
        });
        let engine = Arc::new(LedgerEngine::new(persistence.clone()));

        record_box_spread(
            engine.clone(),
            "SPY",
            450,
            460,
            "20251219",
            1000.0,
            Some("BOX-12345"),
            Currency::USD,
        )
        .await
        .unwrap();

        let transactions = persistence.transactions.read().await;
        assert_eq!(transactions.len(), 1);
        assert!(transactions[0].description.contains("Box Spread"));
        assert_eq!(
            transactions[0].metadata.get("strategy"),
            Some(&"box_spread".to_string())
        );
    }
}
