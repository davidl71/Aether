use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use api::SharedSnapshot;
use futures::StreamExt;
use nats_adapter::{
    async_nats,
    proto::v1::{MarketDataEvent, NatsEnvelope},
};
use prost::Message;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex, time::interval};
use tracing::{debug, info, warn};

/// Prefix for market-data tick subjects; batching applies only to subjects matching this.
const MARKET_DATA_TICK_PREFIX: &str = "market-data.tick";

#[derive(Clone)]
struct CollectionConfig {
    nats_url: String,
    subjects: Vec<String>,
    kv_bucket: Option<String>,
    questdb_ilp_addr: Option<String>,
    use_jetstream: bool,
    /// When > 0, buffer market-data.tick.* messages and flush every this many ms (reduces write rate to KV/QuestDB).
    tick_batch_ms: u64,
}

#[derive(Clone)]
struct CollectionRuntime {
    kv: Option<async_nats::jetstream::kv::Store>,
    questdb: Option<Arc<Mutex<TcpStream>>>,
}

pub fn spawn_collection_aggregator(snapshot: SharedSnapshot, nats_url: Option<String>) {
    tokio::spawn(async move {
        let Some(config) = CollectionConfig::from_env(nats_url) else {
            warn!("collection aggregation disabled: NATS_URL not configured");
            return;
        };

        if config.use_jetstream {
            warn!("NATS_USE_JETSTREAM requested, but Rust collection currently uses core NATS subscriptions only");
        }

        loop {
            match async_nats::connect(&config.nats_url).await {
                Ok(client) => {
                    {
                        let mut state = snapshot.write().await;
                        state.metrics.nats_ok = true;
                    }
                    info!(url = %config.nats_url, "rust collection aggregation connected to NATS");

                    match CollectionRuntime::new(client.clone(), &config, snapshot.clone()).await {
                        Ok(runtime) => {
                            let mut tasks = Vec::new();
                            let tick_batch_ms = config.tick_batch_ms;
                            for subject in &config.subjects {
                                let client = client.clone();
                                let runtime = runtime.clone();
                                let subject = subject.clone();
                                let snapshot = snapshot.clone();
                                let use_batch = tick_batch_ms > 0
                                    && (subject.starts_with(MARKET_DATA_TICK_PREFIX)
                                        || subject == MARKET_DATA_TICK_PREFIX);
                                tasks.push(tokio::spawn(async move {
                                    match client.subscribe(subject.clone()).await {
                                        Ok(mut subscriber) => {
                                            info!(
                                                %subject,
                                                batch_ms = use_batch.then_some(tick_batch_ms),
                                                "rust collection aggregation subscribed"
                                            );
                                            if use_batch {
                                                run_batched_tick_loop(
                                                    subscriber,
                                                    snapshot,
                                                    runtime,
                                                    subject.clone(),
                                                    tick_batch_ms,
                                                )
                                                .await;
                                            } else {
                                                while let Some(message) = subscriber.next().await {
                                                    handle_message(
                                                        snapshot.clone(),
                                                        runtime.clone(),
                                                        subject.clone(),
                                                        message.subject.to_string(),
                                                        message.payload.as_ref(),
                                                    )
                                                    .await;
                                                }
                                            }
                                            warn!(%subject, "collection subscription ended");
                                        }
                                        Err(err) => {
                                            warn!(%subject, %err, "failed to subscribe for collection aggregation")
                                        }
                                    }
                                }));
                            }

                            for task in tasks {
                                if let Err(err) = task.await {
                                    warn!(%err, "collection subscription task failed");
                                }
                            }
                        }
                        Err(err) => {
                            warn!(%err, "failed to initialize rust collection sinks");
                        }
                    }

                    {
                        let mut state = snapshot.write().await;
                        state.metrics.nats_ok = false;
                    }
                }
                Err(err) => {
                    {
                        let mut state = snapshot.write().await;
                        state.metrics.nats_ok = false;
                    }
                    warn!(%err, url = %config.nats_url, "failed to connect rust collection aggregation to NATS");
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

impl CollectionConfig {
    fn from_env(nats_url: Option<String>) -> Option<Self> {
        let nats_url = nats_url
            .filter(|value| !value.trim().is_empty())
            .or_else(|| std::env::var("NATS_URL").ok())?;
        let subjects = std::env::var("NATS_SUBJECTS")
            .unwrap_or_else(|_| "market-data.tick.>,strategy.signal.>,strategy.decision.>".into())
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        let kv_bucket = std::env::var("NATS_KV_BUCKET")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| Some("LIVE_STATE".to_string()));
        let questdb_ilp_addr = std::env::var("QUESTDB_ILP_ADDR")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let use_jetstream = matches!(
            std::env::var("NATS_USE_JETSTREAM")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
        let tick_batch_ms = std::env::var("NATS_TICK_BATCH_MS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0);

        Some(Self {
            nats_url,
            subjects,
            kv_bucket,
            questdb_ilp_addr,
            use_jetstream,
            tick_batch_ms,
        })
    }
}

impl CollectionRuntime {
    async fn new(
        client: async_nats::Client,
        config: &CollectionConfig,
        snapshot: SharedSnapshot,
    ) -> anyhow::Result<Self> {
        let jetstream = async_nats::jetstream::new(client);
        let kv = if let Some(bucket) = &config.kv_bucket {
            let store = match jetstream.get_key_value(bucket).await {
                Ok(store) => store,
                Err(_) => {
                    jetstream
                        .create_key_value(async_nats::jetstream::kv::Config {
                            bucket: bucket.clone(),
                            history: 5,
                            max_age: Duration::from_secs(86400), // 24h TTL
                            max_value_size: 65_536,             // 64 KiB
                            max_bytes: 10 * 1024 * 1024,        // 10 MiB bucket cap
                            ..Default::default()
                        })
                        .await?
                }
            };
            if let Err(e) = store.status().await {
                warn!(%bucket, error = %e, "LIVE_STATE bucket status check failed (writes may still succeed)");
            } else {
                info!(%bucket, "LIVE_STATE bucket reachable");
            }
            Some(store)
        } else {
            None
        };

        let questdb = if let Some(addr) = &config.questdb_ilp_addr {
            let stream = TcpStream::connect(addr).await?;
            {
                let mut state = snapshot.write().await;
                state.metrics.questdb_ok = true;
            }
            info!(addr = %addr, "rust collection aggregation connected to QuestDB");
            Some(Arc::new(Mutex::new(stream)))
        } else {
            None
        };

        Ok(Self { kv, questdb })
    }
}

/// Runs the receive loop for a market-data.tick.* subscription with batching: buffers the latest
/// payload per subject and flushes every `batch_ms` to reduce write rate to KV/QuestDB.
async fn run_batched_tick_loop(
    mut subscriber: async_nats::Subscriber,
    snapshot: SharedSnapshot,
    runtime: CollectionRuntime,
    subscribed_subject: String,
    batch_ms: u64,
) {
    let buffer: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
    let buffer_clone = Arc::clone(&buffer);
    let mut ticker = interval(Duration::from_millis(batch_ms));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            Some(message) = subscriber.next() => {
                let actual = message.subject.to_string();
                let payload = message.payload.as_ref().to_vec();
                let mut buf = buffer_clone.lock().await;
                buf.insert(actual, payload);
            }
            _ = ticker.tick() => {
                let to_flush = {
                    let mut buf = buffer_clone.lock().await;
                    std::mem::take(&mut *buf)
                };
                for (actual_subject, payload) in to_flush {
                    handle_message(
                        snapshot.clone(),
                        runtime.clone(),
                        subscribed_subject.clone(),
                        actual_subject.clone(),
                        &payload,
                    )
                    .await;
                }
            }
        }
    }
}

async fn handle_message(
    snapshot: SharedSnapshot,
    runtime: CollectionRuntime,
    subscribed_subject: String,
    actual_subject: String,
    payload: &[u8],
) {
    let envelope = match NatsEnvelope::decode(payload) {
        Ok(envelope) => envelope,
        Err(err) => {
            debug!(%err, subject = %actual_subject, subscribed_subject = %subscribed_subject, "failed to decode NATS envelope");
            return;
        }
    };

    let message_type = if envelope.message_type.is_empty() {
        "unknown".to_string()
    } else {
        envelope.message_type.clone()
    };

    if let Some(kv) = &runtime.kv {
        let key = kv_key(&actual_subject, &message_type);
        if let Err(err) = kv.put(key.clone(), payload.to_vec().into()).await {
            warn!(%err, %key, subject = %actual_subject, "failed to write LIVE_STATE key");
        }
    }

    if let Some(questdb) = &runtime.questdb {
        match market_data_ilp_line(&envelope, &actual_subject) {
            Ok(Some(line)) => {
                let mut writer = questdb.lock().await;
                if let Err(err) = writer.write_all(line.as_bytes()).await {
                    {
                        let mut state = snapshot.write().await;
                        state.metrics.questdb_ok = false;
                    }
                    warn!(%err, subject = %actual_subject, "failed to write QuestDB ILP line");
                }
            }
            Ok(None) => {}
            Err(err) => warn!(%err, subject = %actual_subject, "failed to build QuestDB ILP line"),
        }
    }
}

fn kv_key(subject: &str, message_type: &str) -> String {
    let symbol = subject
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("default");
    format!("{message_type}.{symbol}")
}

fn market_data_ilp_line(envelope: &NatsEnvelope, subject: &str) -> anyhow::Result<Option<String>> {
    if envelope.message_type != "MarketDataEvent" {
        return Ok(None);
    }

    let event = MarketDataEvent::decode(envelope.payload.as_slice())?;
    let symbol = if event.symbol.is_empty() {
        subject.rsplit('.').next().unwrap_or_default().to_string()
    } else {
        event.symbol.clone()
    };
    if symbol.is_empty() {
        anyhow::bail!("missing market data symbol");
    }

    let timestamp = envelope
        .timestamp
        .as_ref()
        .and_then(|value| (*value).try_into().ok())
        .unwrap_or_else(std::time::SystemTime::now);
    let escaped_symbol = symbol.replace(' ', "\\ ");
    Ok(Some(format!(
        "market_data,symbol={} bid={:.6},ask={:.6},last={:.6},volume={}i {}\n",
        escaped_symbol,
        event.bid,
        event.ask,
        event.last,
        event.volume,
        timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nats_adapter::proto::v1::StrategySignal;

    #[test]
    fn kv_key_uses_message_type_and_symbol() {
        assert_eq!(
            kv_key("market-data.tick.SPY", "MarketDataEvent"),
            "MarketDataEvent.SPY"
        );
    }

    #[test]
    fn market_data_ilp_line_uses_envelope_payload() {
        let envelope = NatsEnvelope {
            id: "evt-1".into(),
            timestamp: Some(prost_types::Timestamp::from(
                std::time::SystemTime::UNIX_EPOCH,
            )),
            source: "cpp-engine".into(),
            message_type: "MarketDataEvent".into(),
            payload: MarketDataEvent {
                symbol: "SPY".into(),
                bid: 100.25,
                ask: 100.75,
                last: 100.50,
                volume: 42,
                timestamp: None,
            }
            .encode_to_vec(),
        };

        let line = market_data_ilp_line(&envelope, "market-data.tick.SPY")
            .unwrap()
            .expect("line");
        assert!(line.contains("market_data,symbol=SPY"));
        assert!(line.contains("bid=100.250000,ask=100.750000,last=100.500000,volume=42i"));
    }

    #[test]
    fn market_data_ilp_line_skips_non_market_data() {
        let envelope = NatsEnvelope {
            id: "evt-2".into(),
            timestamp: None,
            source: "backend".into(),
            message_type: "StrategySignal".into(),
            payload: StrategySignal::default().encode_to_vec(),
        };
        assert!(market_data_ilp_line(&envelope, "strategy.signal.SPY")
            .unwrap()
            .is_none());
    }

    #[test]
    fn market_data_ilp_line_falls_back_to_subject_symbol() {
        let envelope = NatsEnvelope {
            id: "evt-3".into(),
            timestamp: None,
            source: "backend".into(),
            message_type: "MarketDataEvent".into(),
            payload: MarketDataEvent {
                symbol: String::new(),
                bid: 1.0,
                ask: 2.0,
                last: 1.5,
                volume: 3,
                timestamp: None,
            }
            .encode_to_vec(),
        };
        let line = market_data_ilp_line(&envelope, "market-data.tick.XSP")
            .unwrap()
            .expect("line");
        assert!(line.contains("symbol=XSP"));
    }
}
