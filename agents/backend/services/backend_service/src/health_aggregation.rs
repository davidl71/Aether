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

        loop {
            match async_nats::connect(&nats_url).await {
                Ok(client) => {
                    {
                        let mut health = state.write().await;
                        health.nats_connected = true;
                        health.transport =
                            NatsTransportHealthState::connected(Some(nats_url.clone()), Utc::now())
                                .with_subject(topics::system::health())
                                .with_role("subscriber");
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
                                let transport = health
                                    .transport
                                    .observed(Utc::now())
                                    .with_subject(topics::system::health())
                                    .with_role("subscriber");
                                health.transport = transport;
                            }
                        }
                        Err(err) => {
                            let mut health = state.write().await;
                            health.nats_connected = false;
                            health.transport = NatsTransportHealthState::disconnected(
                                Some(nats_url.clone()),
                                Utc::now(),
                                Some(err.to_string()),
                                Some("failed to subscribe to system.health".to_string()),
                            )
                            .with_subject(topics::system::health())
                            .with_role("subscriber");
                            warn!(%err, "failed to subscribe to system.health");
                        }
                    }

                    {
                        let mut health = state.write().await;
                        health.nats_connected = false;
                        health.transport = NatsTransportHealthState::disconnected(
                            Some(nats_url.clone()),
                            Utc::now(),
                            None,
                            Some("system.health subscription ended".to_string()),
                        )
                        .with_subject(topics::system::health())
                        .with_role("subscriber");
                    }
                }
                Err(err) => {
                    let mut health = state.write().await;
                    health.nats_connected = false;
                    health.transport = NatsTransportHealthState::disconnected(
                        Some(nats_url.clone()),
                        Utc::now(),
                        Some(err.to_string()),
                        Some("failed to connect health aggregation to NATS".to_string()),
                    )
                    .with_subject(topics::system::health())
                    .with_role("subscriber");
                    warn!(%err, "failed to connect health aggregation to NATS");
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}
