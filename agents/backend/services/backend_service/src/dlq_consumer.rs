//! Consumer for `system.dlq.backend.*` (dead-letter queue).
//!
//! Subscribes to backend DLQ messages for alerting and replay visibility.
//! Logs each DLQ message at WARN with component, error_type, original_topic, and error_message.

use std::time::Duration;

use futures::StreamExt;
use nats_adapter::{async_nats, extract_proto_payload, topics, DeadLetterMessage, Result};
use tracing::{info, warn};

/// Spawn a task that subscribes to `system.dlq.backend.>` and logs each message.
/// No-op if NATS_URL is not set. Reconnects with backoff on disconnect.
pub fn spawn_dlq_consumer(nats_url: Option<String>) {
    tokio::spawn(async move {
        let Some(nats_url) = nats_url.filter(|v| !v.trim().is_empty()) else {
            return;
        };

        let subject = topics::dlq::component_dlq("backend");

        loop {
            match async_nats::connect(&nats_url).await {
                Ok(client) => {
                    info!(%subject, "DLQ consumer subscribed");
                    match client.subscribe(subject.clone()).await {
                        Ok(mut sub) => {
                            while let Some(msg) = sub.next().await {
                                if let Err(e) =
                                    handle_dlq_message(msg.subject.as_str(), msg.payload.as_ref())
                                {
                                    warn!(error = %e, subject = %msg.subject, "DLQ consumer decode failed");
                                }
                            }
                        }
                        Err(err) => warn!(%err, %subject, "DLQ consumer subscribe failed"),
                    }
                }
                Err(err) => warn!(%err, "DLQ consumer NATS connect failed"),
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

fn handle_dlq_message(subject: &str, payload: &[u8]) -> Result<()> {
    let dlq: DeadLetterMessage = extract_proto_payload(payload)?;
    warn!(
        subject = %subject,
        id = %dlq.id,
        component = %dlq.component,
        error_type = %dlq.error_type,
        original_topic = %dlq.original_topic,
        error_message = %dlq.error_message,
        retry_count = dlq.retry_count,
        "DLQ message received"
    );
    Ok(())
}
