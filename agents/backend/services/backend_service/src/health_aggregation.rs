use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use api::{BackendHealthState, NatsTransportHealthState, SharedHealthAggregate};
use chrono::Utc;
use futures::StreamExt;
use nats_adapter::{async_nats, topics};
use tracing::{info, warn};

pub fn spawn_health_aggregator(state: SharedHealthAggregate, nats_url: Option<String>) {
    tokio::spawn(async move {
        let Some(nats_url) = nats_url.filter(|value| !value.trim().is_empty()) else {
            warn!("health aggregation disabled: NATS_URL not configured");
            return;
        };

        // Full reconnect cycles (connect/subscribe/stream end → backoff). Monotonic for this task.
        let reconnect_cycles = AtomicU64::new(0);

        loop {
            match async_nats::connect(&nats_url).await {
                Ok(client) => {
                    let stats = client.statistics();
                    let stats_in_bytes = stats.in_bytes.load(Ordering::Relaxed);
                    let stats_out_bytes = stats.out_bytes.load(Ordering::Relaxed);
                    let stats_in_messages = stats.in_messages.load(Ordering::Relaxed);
                    let stats_out_messages = stats.out_messages.load(Ordering::Relaxed);
                    let stats_connects = stats.connects.load(Ordering::Relaxed);
                    {
                        let mut health = state.write().await;
                        health.nats_connected = true;
                        let mut t =
                            NatsTransportHealthState::connected(Some(nats_url.clone()), Utc::now())
                                .with_subject(topics::system::health())
                                .with_role("subscriber");
                        t.reconnect_count = reconnect_cycles.load(Ordering::Relaxed);
                        t.in_bytes = Some(stats_in_bytes);
                        t.out_bytes = Some(stats_out_bytes);
                        t.in_messages = Some(stats_in_messages);
                        t.out_messages = Some(stats_out_messages);
                        t.connects = Some(stats_connects);
                        health.transport = t;
                    }
                    info!(
                        "health aggregation subscribed to {}",
                        topics::system::health()
                    );

                    match client.subscribe(topics::system::health().to_string()).await {
                        Ok(mut subscriber) => {
                            while let Some(message) = subscriber.next().await {
                                if let Some(health) =
                                    api::backend_health_from_message(message.payload.as_ref())
                                {
                                    let backend = health.backend.clone();
                                    let mapped = BackendHealthState::from_proto(health);
                                    state.write().await.backends.insert(backend, mapped);
                                }
                                let mut health = state.write().await;
                                let mut transport = health
                                    .transport
                                    .observed(Utc::now())
                                    .with_subject(topics::system::health())
                                    .with_role("subscriber");
                                transport.reconnect_count =
                                    reconnect_cycles.load(Ordering::Relaxed);
                                transport.in_bytes = Some(stats_in_bytes);
                                transport.out_bytes = Some(stats_out_bytes);
                                transport.in_messages = Some(stats_in_messages);
                                transport.out_messages = Some(stats_out_messages);
                                transport.connects = Some(stats_connects);
                                health.transport = transport;
                            }
                        }
                        Err(err) => {
                            let mut health = state.write().await;
                            health.nats_connected = false;
                            let mut t = NatsTransportHealthState::disconnected(
                                Some(nats_url.clone()),
                                Utc::now(),
                                Some(err.to_string()),
                                Some("failed to subscribe to system.health".to_string()),
                            )
                            .with_subject(topics::system::health())
                            .with_role("subscriber");
                            t.reconnect_count = reconnect_cycles.load(Ordering::Relaxed);
                            t.in_bytes = Some(stats_in_bytes);
                            t.out_bytes = Some(stats_out_bytes);
                            t.in_messages = Some(stats_in_messages);
                            t.out_messages = Some(stats_out_messages);
                            t.connects = Some(stats_connects);
                            health.transport = t;
                            warn!(%err, "failed to subscribe to system.health");
                        }
                    }

                    {
                        let mut health = state.write().await;
                        health.nats_connected = false;
                        let mut t = NatsTransportHealthState::disconnected(
                            Some(nats_url.clone()),
                            Utc::now(),
                            None,
                            Some("system.health subscription ended".to_string()),
                        )
                        .with_subject(topics::system::health())
                        .with_role("subscriber");
                        t.reconnect_count = reconnect_cycles.load(Ordering::Relaxed);
                        t.in_bytes = Some(stats_in_bytes);
                        t.out_bytes = Some(stats_out_bytes);
                        t.in_messages = Some(stats_in_messages);
                        t.out_messages = Some(stats_out_messages);
                        t.connects = Some(stats_connects);
                        health.transport = t;
                    }
                }
                Err(err) => {
                    let mut health = state.write().await;
                    health.nats_connected = false;
                    let mut t = NatsTransportHealthState::disconnected(
                        Some(nats_url.clone()),
                        Utc::now(),
                        Some(err.to_string()),
                        Some("failed to connect health aggregation to NATS".to_string()),
                    )
                    .with_subject(topics::system::health())
                    .with_role("subscriber");
                    t.reconnect_count = reconnect_cycles.load(Ordering::Relaxed);
                    health.transport = t;
                    warn!(%err, "failed to connect health aggregation to NATS");
                }
            }

            reconnect_cycles.fetch_add(1, Ordering::Relaxed);
            {
                let mut health = state.write().await;
                health.transport.reconnect_count = reconnect_cycles.load(Ordering::Relaxed);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}
