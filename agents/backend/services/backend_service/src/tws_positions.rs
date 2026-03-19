//! TWS/IB Gateway position fetcher – merge real positions into SystemSnapshot.
//!
//! When market data provider is `tws` and `IB_PORTAL_URL` is **not** set, periodically
//! fetches positions from the TWS socket API and merges them into the shared snapshot.
//! **Exclusive with Client Portal:** use either IB Client Portal (IB_PORTAL_URL) or TWS
//! for positions, not both (only one can be logged in at a time).
//!
//! On connection failure uses exponential backoff (2s → 60s cap) before next attempt; one log per attempt.

use std::time::Duration;

use api::{Alert, PositionSnapshot, SharedSnapshot};
use backoff::backoff::Backoff;
use backoff::exponential::ExponentialBackoffBuilder;
use ibapi::prelude::*;
use ibapi::Client;
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::tws_env::parse_tws_client_id;

/// Same port order as tws_market_data (paper first).
const PORTS_AUTODETECT: &[u16] = &[7497, 4002, 7496, 4001];

const UPDATE_INTERVAL_SECS: u64 = 60;

/// Spawn background task to fetch TWS positions and merge into SystemSnapshot.
/// Uses a separate client_id (TWS_CLIENT_ID + 1) to avoid conflicting with market data.
pub fn spawn_tws_position_fetcher(state: SharedSnapshot) {
    let backoff: backoff::ExponentialBackoff = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(2))
        .with_multiplier(2.0)
        .with_max_interval(Duration::from_secs(60))
        .with_randomization_factor(0.0)
        .with_max_elapsed_time(None)
        .build();

    tokio::spawn(async move {
        let mut backoff = backoff;
        let mut update_interval = interval(Duration::from_secs(UPDATE_INTERVAL_SECS));
        update_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        if let Err(e) = fetch_and_merge_positions(&state).await {
            warn!(error = %e, "initial TWS position fetch failed");
        } else {
            backoff.reset();
        }

        loop {
            update_interval.tick().await;
            match fetch_and_merge_positions(&state).await {
                Ok(()) => {
                    backoff.reset();
                }
                Err(e) => {
                    let delay = backoff.next_backoff().unwrap_or(Duration::from_secs(60));
                    warn!(
                        error = %e,
                        delay_secs = delay.as_secs(),
                        "TWS positions: reconnecting…"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    });
}

async fn fetch_and_merge_positions(state: &SharedSnapshot) -> anyhow::Result<()> {
    let host = std::env::var("TWS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let ports: Vec<u16> = match std::env::var("TWS_PORT") {
        Ok(s) => match s.parse::<u16>() {
            Ok(p) => vec![p],
            Err(_) => PORTS_AUTODETECT.to_vec(),
        },
        Err(_) => PORTS_AUTODETECT.to_vec(),
    };
    let base_id: i32 = parse_tws_client_id(0);
    let client_id = base_id.saturating_add(1);

    let mut positions = Vec::new();
    let mut last_err = None;
    for &port in &ports {
        let address = format!("{}:{}", host, port);
        match Client::connect(&address, client_id).await {
            Ok(client) => {
                let mut sub = client
                    .positions()
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                while let Some(result) = sub.next().await {
                    match result.map_err(|e| anyhow::anyhow!("{}", e))? {
                        PositionUpdate::Position(p) => {
                            let symbol = p.contract.symbol.to_string();
                            let conid = p.contract.contract_id;
                            let position_type = Some(format!("{}", p.contract.security_type));
                            positions.push(PositionSnapshot {
                                id: format!("tws-{}", conid),
                                symbol: symbol.clone(),
                                quantity: p.position as i32,
                                cost_basis: p.average_cost,
                                mark: p.average_cost,
                                unrealized_pnl: 0.0,
                                account_id: Some(p.account.clone()),
                                source: Some("TWS".into()),
                                position_type: position_type.clone(),
                                strategy: None,
                                apr_pct: None,
                                combo_net_bid: None,
                                combo_net_ask: None,
                                combo_quote_source: None,
                            });
                        }
                        PositionUpdate::PositionEnd => break,
                    }
                }
                break;
            }
            Err(e) => last_err = Some((address, e)),
        }
    }

    if positions.is_empty() && last_err.is_some() {
        let (addr, e) = last_err.unwrap();
        anyhow::bail!("TWS connection failed (tried all ports): {}: {}", addr, e);
    }

    if positions.is_empty() {
        info!(
            "TWS positions: 0 positions (check TWS/Gateway is running and account has positions)"
        );
    } else {
        debug!(count = positions.len(), "fetched TWS positions");
    }

    {
        let mut snapshot = state.write().await;
        snapshot.touch();

        snapshot.positions.retain(|p| !p.id.starts_with("tws-"));

        let new_count = positions.len();
        snapshot.positions.append(&mut positions);

        if new_count > 0 {
            info!(
                count = new_count,
                "merged {} TWS positions into SystemSnapshot", new_count
            );
            snapshot.alerts.push(Alert::info(format!(
                "TWS positions updated: {} positions",
                new_count
            )));
            while snapshot.alerts.len() > 32 {
                snapshot.alerts.remove(0);
            }
        }
    }

    Ok(())
}
