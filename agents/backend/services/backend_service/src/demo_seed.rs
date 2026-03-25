use chrono::{TimeDelta, Utc};

use api::{
    Alert, CandleSnapshot, HistoricPosition, OrderSnapshot, PositionSnapshot, SymbolSnapshot,
    SystemSnapshot,
};

pub const DEFAULT_DEMO_SYMBOLS: &[&str] = &["SPX", "XSP", "NDX"];

/// Seeds only demo positions, orders, and historic trades into an empty snapshot.
/// This is service-local bootstrap data for CI/QA and local smoke testing.
pub fn seed_demo_positions(snapshot: &mut SystemSnapshot) {
    if snapshot.positions.is_empty() {
        snapshot
            .positions
            .extend(demo_positions(&snapshot.account_id));
    }
    if snapshot.orders.is_empty() {
        snapshot.orders.extend(demo_orders());
    }
    if snapshot.historic.is_empty() {
        snapshot.historic.extend(demo_historic());
    }
    snapshot
        .alerts
        .push(Alert::info("Demo positions seeded (mock_positions=true)"));
    while snapshot.alerts.len() > 32 {
        snapshot.alerts.remove(0);
    }
}

/// Seeds only demo symbol quotes into an empty snapshot.
/// This intentionally does not mutate unrelated service-health fields.
pub fn seed_demo_market_data(snapshot: &mut SystemSnapshot, symbols: &[String]) {
    let default_symbols: Vec<String> = DEFAULT_DEMO_SYMBOLS.iter().map(|s| s.to_string()).collect();
    let symbols_slice = if symbols.is_empty() {
        &default_symbols
    } else {
        symbols
    };
    if snapshot.symbols.is_empty() {
        snapshot
            .symbols
            .extend(demo_symbol_snapshots(symbols_slice));
    }
    snapshot.alerts.push(Alert::info(
        "Demo market data seeded (mock_market_data=true)",
    ));
    while snapshot.alerts.len() > 32 {
        snapshot.alerts.remove(0);
    }
}

fn demo_positions(account_id: &str) -> Vec<PositionSnapshot> {
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

fn demo_orders() -> Vec<OrderSnapshot> {
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

fn demo_historic() -> Vec<HistoricPosition> {
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

fn demo_symbol_snapshots(symbols: &[String]) -> Vec<SymbolSnapshot> {
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
