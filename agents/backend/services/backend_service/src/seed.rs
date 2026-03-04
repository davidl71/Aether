use api::{Alert, HistoricPosition, OrderSnapshot, PositionSnapshot, SystemSnapshot};
use chrono::{Duration, Utc};

pub fn seed_static_data(snapshot: &mut SystemSnapshot) {
  if snapshot.positions.is_empty() {
    snapshot.positions.push(PositionSnapshot {
      id: "POS-1".into(),
      symbol: "XSP".into(),
      quantity: 2,
      cost_basis: 98.75,
      mark: 101.10,
      unrealized_pnl: 4.7,
    });
  }

  if snapshot.orders.is_empty() {
    snapshot.orders.push(OrderSnapshot {
      id: "ORD-1".into(),
      symbol: "XSP".into(),
      side: "BUY".into(),
      quantity: 2,
      status: "FILLED".into(),
      submitted_at: Utc::now() - Duration::minutes(30),
    });
  }

  if snapshot.historic.is_empty() {
    snapshot.historic.push(HistoricPosition {
      id: "POS-0".into(),
      symbol: "SPY".into(),
      quantity: 2,
      realized_pnl: 6.2,
      closed_at: Utc::now() - Duration::hours(5),
    });
  }

  snapshot.alerts.push(Alert::info("Mock runtime initialised"));
  snapshot.alerts.push(Alert::info("Waiting for market data updates"));
  while snapshot.alerts.len() > 32 {
    snapshot.alerts.remove(0);
  }
}
