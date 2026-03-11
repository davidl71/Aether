use std::time::Duration;

use api::{BackendHealthState, SharedHealthAggregate};
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
                            }
                        }
                        Err(err) => warn!(%err, "failed to subscribe to system.health"),
                    }

                    state.write().await.nats_connected = false;
                }
                Err(err) => {
                    state.write().await.nats_connected = false;
                    warn!(%err, "failed to connect health aggregation to NATS");
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}
