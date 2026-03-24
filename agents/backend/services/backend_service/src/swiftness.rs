use std::time::Duration;

use anyhow::Context;
use api::{Alert, PositionSnapshot};
use reqwest::Client;
use serde::Deserialize;
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::shared_state::SharedSnapshot;

const SWIFTNESS_API_URL: &str = "http://127.0.0.1:8081";
const UPDATE_INTERVAL_SECS: u64 = 60; // Update every minute

pub fn swiftness_enabled() -> bool {
    matches!(
        std::env::var("ENABLE_SWIFTNESS")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

#[derive(Debug, Clone, Deserialize)]
struct SwiftnessPosition {
    id: String,
    symbol: String,
    quantity: i32,
    cost_basis: f64,
    mark: f64,
    unrealized_pnl: f64,
}

/// Spawn background task to periodically fetch Swiftness positions and merge into SystemSnapshot
pub fn spawn_swiftness_position_fetcher(state: SharedSnapshot) {
    tokio::spawn(async move {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("failed to create HTTP client");

        let mut update_interval = interval(Duration::from_secs(UPDATE_INTERVAL_SECS));
        update_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Initial fetch
        if let Err(e) = fetch_and_merge_positions(&client, &state).await {
            warn!(%e, "initial Swiftness position fetch failed");
        }

        loop {
            update_interval.tick().await;

            if let Err(e) = fetch_and_merge_positions(&client, &state).await {
                warn!(%e, "Swiftness position fetch failed, will retry on next interval");
            }
        }
    });
}

async fn fetch_and_merge_positions(client: &Client, state: &SharedSnapshot) -> anyhow::Result<()> {
    let url = format!(
        "{}/positions?check_validity=true&max_age_days=90",
        SWIFTNESS_API_URL
    );

    debug!(%url, "fetching Swiftness positions");

    let response = client
        .get(&url)
        .send()
        .await
        .context("failed to send HTTP request to Swiftness API")?;

    if !response.status().is_success() {
        anyhow::bail!("Swiftness API returned error status: {}", response.status());
    }

    let positions: Vec<SwiftnessPosition> = response
        .json()
        .await
        .context("failed to parse Swiftness positions JSON")?;

    debug!(count = positions.len(), "fetched Swiftness positions");

    // Merge positions into SystemSnapshot
    {
        let mut snapshot = state.write().await;
        snapshot.touch();

        // Remove existing Swiftness positions (identified by symbol prefix "SWIFTNESS-")
        snapshot
            .positions
            .retain(|p| !p.symbol.starts_with("SWIFTNESS-"));

        // Add new Swiftness positions
        let new_count = positions.len();
        let account_id = snapshot.account_id.clone();
        for pos in positions {
            snapshot.positions.push(PositionSnapshot {
                id: pos.id,
                symbol: pos.symbol,
                quantity: pos.quantity,
                cost_basis: pos.cost_basis,
                mark: pos.mark,
                unrealized_pnl: pos.unrealized_pnl,
                account_id: Some(account_id.clone()),
                source: Some("swiftness".into()),
            });
        }

        if new_count > 0 {
            info!(
                count = new_count,
                "merged {} Swiftness positions into SystemSnapshot", new_count
            );
            snapshot.alerts.push(Alert::info(format!(
                "Swiftness positions updated: {} positions",
                new_count
            )));
            // Trim alerts to keep list manageable
            while snapshot.alerts.len() > 32 {
                snapshot.alerts.remove(0);
            }
        }
    }

    Ok(())
}

/// Check if Swiftness API is available
#[allow(dead_code)]
pub async fn check_swiftness_api_health() -> bool {
    let client = match Client::builder().timeout(Duration::from_secs(2)).build() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let url = format!("{}/health", SWIFTNESS_API_URL);
    client
        .get(&url)
        .send()
        .await
        .ok()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
