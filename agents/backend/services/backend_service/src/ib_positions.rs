//! IB Client Portal position fetcher – merge real positions into SystemSnapshot.
//!
//! When `IB_PORTAL_URL` is set, periodically fetches positions from the IB Client Portal API
//! and merges them into the shared snapshot so the TUI and other consumers see real IB positions.
//! Requires the IB Client Portal Gateway to be running and logged in (e.g. https://localhost:5001).
//! Client Portal and TWS socket API are exclusive – only one can be logged in at a time.

use std::time::Duration;

use api::{Alert, PositionSnapshot, SharedSnapshot};
use tokio::time::interval;
use tracing::{debug, info, warn};

const UPDATE_INTERVAL_SECS: u64 = 60;

/// Enable IB position fetch when IB_PORTAL_URL is set and not empty.
pub fn ib_positions_enabled() -> bool {
    std::env::var("IB_PORTAL_URL")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

/// Spawn background task to periodically fetch IB positions (Client Portal) and merge into SystemSnapshot.
pub fn spawn_ib_position_fetcher(state: SharedSnapshot) {
    tokio::spawn(async move {
        let mut update_interval = interval(Duration::from_secs(UPDATE_INTERVAL_SECS));
        update_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        if let Err(e) = fetch_and_merge_positions(&state).await {
            warn!(error = %e, "initial IB position fetch failed");
        }

        loop {
            update_interval.tick().await;
            if let Err(e) = fetch_and_merge_positions(&state).await {
                warn!(error = %e, "IB position fetch failed, will retry on next interval");
            }
        }
    });
}

async fn fetch_and_merge_positions(state: &SharedSnapshot) -> anyhow::Result<()> {
    let positions = api::fetch_ib_positions_all().await.map_err(|e| anyhow::anyhow!("{}", e))?;

    if positions.is_empty() {
        info!(
            "IB positions: 0 positions (check Client Portal is running, logged in, and accounts have positions)"
        );
    } else {
        debug!(count = positions.len(), "fetched IB positions from Client Portal");
    }

    {
        let mut snapshot = state.write().await;
        snapshot.touch();

        snapshot
            .positions
            .retain(|p| !p.id.starts_with("ib-"));

        let account_id = snapshot.account_id.clone();
        let new_count = positions.len();
        for pos in positions {
            let id = pos
                .conid
                .map(|c| format!("ib-{}", c))
                .unwrap_or_else(|| format!("ib-{}", pos.symbol));
            snapshot.positions.push(PositionSnapshot {
                id,
                symbol: pos.symbol,
                quantity: pos.quantity as i32,
                cost_basis: pos.avg_price,
                mark: pos.current_price.unwrap_or(pos.avg_price),
                unrealized_pnl: pos.unrealized_pl.unwrap_or(0.0),
                account_id: pos.account_id.or(Some(account_id.clone())),
                source: Some("IB".into()),
            });
        }

        if new_count > 0 {
            info!(
                count = new_count,
                "merged {} IB positions into SystemSnapshot", new_count
            );
            snapshot.alerts.push(Alert::info(format!(
                "IB positions updated: {} positions",
                new_count
            )));
            while snapshot.alerts.len() > 32 {
                snapshot.alerts.remove(0);
            }
        }
    }

    Ok(())
}
