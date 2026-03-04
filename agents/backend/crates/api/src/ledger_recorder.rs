use std::sync::Arc;

use tracing::{debug, warn};

pub fn record_position_close(
  ledger: Arc<ledger::LedgerEngine>,
  symbol: &str,
  quantity: i32,
  cost_basis: f64,
  mark: f64,
  order_id: &str,
) {
  let symbol = symbol.to_string();
  let order_id = order_id.to_string();
  tokio::spawn(async move {
    if let Err(err) = ledger::record_position_close(
      ledger,
      &symbol,
      quantity,
      cost_basis,
      mark,
      ledger::Currency::USD,
      Some(&order_id),
    )
    .await
    {
      warn!(error = %err, symbol = %symbol, "Failed to record position close in ledger");
    }
  });
}

pub fn record_position_change(
  ledger: Arc<ledger::LedgerEngine>,
  symbol: &str,
  quantity: i32,
  price: f64,
  order_id: &str,
) {
  let symbol = symbol.to_string();
  let order_id = order_id.to_string();
  tokio::spawn(async move {
    ledger::record_position_change_safe(
      ledger,
      &symbol,
      quantity,
      price,
      ledger::Currency::USD,
      Some(&order_id),
    )
    .await;
  });
}

pub fn record_box_spread(
  ledger: Arc<ledger::LedgerEngine>,
  symbol: &str,
  strike1: i32,
  strike2: i32,
  expiry: &str,
  net_debit: f64,
  trade_id: Option<&str>,
) {
  debug!(
    %symbol, strike1, strike2, %expiry, net_debit,
    "Box spread transaction queued for ledger recording"
  );
  let symbol = symbol.to_string();
  let expiry = expiry.to_string();
  let trade_id = trade_id.map(|s| s.to_string());
  let ledger_clone = ledger;
  tokio::spawn(async move {
    ledger::record_box_spread_safe(
      ledger_clone,
      &symbol,
      strike1,
      strike2,
      &expiry,
      net_debit,
      trade_id.as_deref(),
      ledger::Currency::USD,
    )
    .await;
  });
}

pub fn record_cash_flow(
  ledger: Arc<ledger::LedgerEngine>,
  amount: f64,
  currency: ledger::Currency,
  description: &str,
  is_deposit: bool,
) {
  debug!(
    amount, currency = ?currency, %description, is_deposit,
    "Cash flow transaction queued for ledger recording"
  );
  let description = description.to_string();
  tokio::spawn(async move {
    if let Err(err) = ledger::record_cash_flow(ledger, amount, currency, &description, is_deposit).await {
      warn!(error = %err, description = %description, "Failed to record cash flow in ledger");
    }
  });
}
