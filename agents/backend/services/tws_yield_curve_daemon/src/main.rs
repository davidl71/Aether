//! Standalone daemon: fetches box spread yield curve from TWS (option chain + quotes) and
//! writes to NATS KV key `yield_curve.{symbol}` as protobuf `YieldCurve` so backend/TUI can read it.
//!
//! Run with TWS/IB Gateway up. Env: NATS_URL, NATS_KV_BUCKET, TWS_HOST, TWS_PORT, SYMBOLS, INTERVAL_SECS.
//! On TWS fetch failure uses exponential backoff (2s → 60s cap) before next cycle; one log per attempt.

use api::yield_curve_proto::{encode_yield_curve_to_bytes, yield_curve_from_opportunities};
use bytes::Bytes;
use nats_adapter::async_nats::Client;
use nats_adapter::NatsClient;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use backoff::backoff::Backoff;
use backoff::exponential::ExponentialBackoffBuilder;

const NATS_URL_ENV: &str = "NATS_URL";
const NATS_URL_DEFAULT: &str = "nats://localhost:4222";
const KV_BUCKET_ENV: &str = "NATS_KV_BUCKET";
const KV_BUCKET_DEFAULT: &str = "LIVE_STATE";
const KEY_PREFIX: &str = "yield_curve";
const SYMBOLS_ENV: &str = "SYMBOLS";
const SYMBOLS_DEFAULT: &str = "SPX";
const INTERVAL_SECS_ENV: &str = "INTERVAL_SECS";
const INTERVAL_SECS_DEFAULT: u64 = 60;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let nats_url = std::env::var(NATS_URL_ENV).unwrap_or_else(|_| NATS_URL_DEFAULT.to_string());
    let bucket = std::env::var(KV_BUCKET_ENV).unwrap_or_else(|_| KV_BUCKET_DEFAULT.to_string());
    let symbols: Vec<String> = std::env::var(SYMBOLS_ENV)
        .unwrap_or_else(|_| SYMBOLS_DEFAULT.to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let interval_secs: u64 = std::env::var(INTERVAL_SECS_ENV)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(INTERVAL_SECS_DEFAULT);

    if symbols.is_empty() {
        warn!("No symbols configured (set SYMBOLS=e.g. SPX,SPXW); exiting");
        return Ok(());
    }

    let nc = NatsClient::connect(&nats_url)
        .await
        .map_err(|e| format!("NATS connect: {}", e))?;
    let health_interval_secs: u64 = std::env::var("HEALTH_PUBLISH_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(15);
    let mut health_extra = HashMap::new();
    health_extra.insert("pid".to_string(), std::process::id().to_string());
    health_extra.insert("service".to_string(), "tws_yield_curve_daemon".to_string());
    health_extra.insert("bucket".to_string(), bucket.clone());
    health_extra.insert("symbols".to_string(), symbols.join(","));
    nats_adapter::spawn_health_publisher(
        Arc::new(nc.clone()),
        "tws_yield_curve_daemon".to_string(),
        health_interval_secs,
        health_extra,
    );
    let client: Client = nc.client().clone();

    let js = nats_adapter::async_nats::jetstream::new(client.clone());
    let kv = match js.get_key_value(bucket.as_str()).await {
        Ok(k) => k,
        Err(_) => js
            .create_key_value(nats_adapter::async_nats::jetstream::kv::Config {
                bucket: bucket.clone(),
                history: 3,
                max_age: Duration::from_secs(86400),
                max_value_size: 65_536,
                max_bytes: 10 * 1024 * 1024,
                ..Default::default()
            })
            .await
            .map_err(|e| format!("KV create bucket {:?}: {}", bucket, e))?,
    };

    info!(
        %nats_url,
        %bucket,
        symbols = ?symbols,
        interval_secs = interval_secs,
        "tws_yield_curve_daemon started"
    );

    async fn write_cycle(
        kv: &nats_adapter::async_nats::jetstream::kv::Store,
        symbols: &[String],
    ) -> bool {
        let mut had_tws_error = false;
        for symbol in symbols {
            match tws_yield_curve::fetch_yield_curve_from_tws(symbol).await {
                Ok(opportunities) if !opportunities.is_empty() => {
                    let key = format!("{}.{}", KEY_PREFIX, symbol);
                    let strike_width = opportunities
                        .first()
                        .and_then(|o| o.get("spread"))
                        .and_then(|s| s.get("strike_width"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(4.0);
                    let Some(yc) =
                        yield_curve_from_opportunities(&opportunities, symbol, strike_width)
                    else {
                        warn!(
                            %symbol,
                            opp_count = opportunities.len(),
                            "yield_curve_from_opportunities produced no proto curve; skipping KV write"
                        );
                        continue;
                    };
                    let payload = encode_yield_curve_to_bytes(&yc);
                    if let Err(e) = kv.put(key.as_str(), Bytes::from(payload)).await {
                        warn!(%key, error = %e, "KV put failed");
                    } else {
                        debug!(%key, points = opportunities.len(), "wrote");
                    }
                }
                Ok(_) => debug!(%symbol, "no TWS points"),
                Err(e) => {
                    had_tws_error = true;
                    warn!(%symbol, error = %e, "TWS fetch failed");
                }
            }
        }
        had_tws_error
    }

    let backoff: backoff::ExponentialBackoff = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(2))
        .with_multiplier(2.0)
        .with_max_interval(Duration::from_secs(60))
        .with_randomization_factor(0.0)
        .with_max_elapsed_time(None)
        .build();

    let mut backoff = backoff;
    let mut had_error = write_cycle(&kv, &symbols).await;
    if had_error {
        let delay = backoff.next_backoff().unwrap_or(Duration::from_secs(60));
        warn!(
            delay_secs = delay.as_secs(),
            "TWS yield curve: reconnecting…"
        );
        tokio::time::sleep(delay).await;
    } else {
        backoff.reset();
    }

    let mut tick = tokio::time::interval(Duration::from_secs(interval_secs));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        had_error = write_cycle(&kv, &symbols).await;
        if had_error {
            let delay = backoff.next_backoff().unwrap_or(Duration::from_secs(60));
            warn!(
                delay_secs = delay.as_secs(),
                "TWS yield curve: reconnecting…"
            );
            tokio::time::sleep(delay).await;
        } else {
            backoff.reset();
        }
    }
}
