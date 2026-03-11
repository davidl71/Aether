use std::marker::PhantomData;

use futures::StreamExt;
use prost::Message as ProstMessage;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::client::NatsClient;
use crate::dlq::{error_type_from_error, DlqService};
use crate::error::{NatsAdapterError, Result};
use crate::serde::{encode_envelope, extract_proto_payload};

/// Bridge between Tokio channels and NATS topics using protobuf envelopes only.
pub struct ChannelBridge<T> {
    client: NatsClient,
    _phantom: PhantomData<T>,
}

impl<T> ChannelBridge<T>
where
    T: ProstMessage + Default + Send + Sync + 'static,
{
    pub fn new(client: NatsClient) -> Self {
        Self {
            client,
            _phantom: PhantomData,
        }
    }

    pub fn create_publisher(
        &self,
        subject: impl Into<String>,
        source: impl Into<String>,
        message_type: impl Into<String>,
    ) -> Publisher<T> {
        Publisher::new(
            self.client.clone(),
            subject.into(),
            source.into(),
            message_type.into(),
        )
    }

    pub fn create_publisher_with_dlq(
        &self,
        subject: impl Into<String>,
        source: impl Into<String>,
        message_type: impl Into<String>,
        dlq_service: DlqService,
    ) -> Publisher<T> {
        Publisher::with_dlq(
            self.client.clone(),
            subject.into(),
            source.into(),
            message_type.into(),
            dlq_service,
        )
    }

    pub fn create_subscriber(&self, subject: impl Into<String>) -> Subscriber<T> {
        Subscriber::new(self.client.clone(), subject.into())
    }
}

/// Publisher that bridges Tokio channel to NATS.
pub struct Publisher<T> {
    client: NatsClient,
    subject: String,
    source: String,
    message_type: String,
    dlq_service: Option<DlqService>,
    _phantom: PhantomData<T>,
}

impl<T> Publisher<T>
where
    T: ProstMessage + Default + Send + Sync + 'static,
{
    pub fn new(
        client: NatsClient,
        subject: impl Into<String>,
        source: impl Into<String>,
        message_type: impl Into<String>,
    ) -> Self {
        Self {
            client,
            subject: subject.into(),
            source: source.into(),
            message_type: message_type.into(),
            dlq_service: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_dlq(
        client: NatsClient,
        subject: impl Into<String>,
        source: impl Into<String>,
        message_type: impl Into<String>,
        dlq_service: DlqService,
    ) -> Self {
        Self {
            client,
            subject: subject.into(),
            source: source.into(),
            message_type: message_type.into(),
            dlq_service: Some(dlq_service),
            _phantom: PhantomData,
        }
    }

    pub async fn publish(&self, payload: &T) -> Result<()> {
        let original_payload = payload.encode_to_vec();
        let bytes = encode_envelope(&self.source, &self.message_type, payload)?;

        if let Some(ref dlq_service) = self.dlq_service {
            let config = dlq_service.config();
            let mut last_error = None;

            for attempt in 0..=config.max_retries {
                match self
                    .client
                    .client()
                    .publish(self.subject.clone(), bytes.clone())
                    .await
                {
                    Ok(_) => {
                        if attempt > 0 {
                            info!(
                                subject = %self.subject,
                                attempt = attempt,
                                "Message published successfully after {} retries",
                                attempt
                            );
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        last_error = Some(NatsAdapterError::from(e));

                        if attempt >= config.max_retries {
                            let error = last_error.unwrap();
                            let error_type = error_type_from_error(&error);
                            let error_message = error.to_string();

                            if let Err(dlq_err) = dlq_service
                                .publish_failed_message(
                                    &self.subject,
                                    error_type,
                                    &error_message,
                                    attempt,
                                    original_payload.clone(),
                                    None,
                                )
                                .await
                            {
                                error!(
                                    dlq_error = %dlq_err,
                                    "Failed to send message to DLQ after publish failure"
                                );
                            }

                            return Err(error);
                        }

                        let delay = dlq_service.calculate_retry_delay(attempt);
                        warn!(
                            subject = %self.subject,
                            attempt = attempt + 1,
                            max_retries = config.max_retries,
                            delay_ms = delay.as_millis(),
                            error = %last_error.as_ref().unwrap(),
                            "Publish failed, retrying in {:?}",
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }

            Err(last_error.unwrap_or_else(|| {
                NatsAdapterError::Publish("Unknown error during retry loop".to_string())
            }))
        } else {
            self.client
                .client()
                .publish(self.subject.clone(), bytes)
                .await
                .map_err(|e| {
                    error!(
                        subject = %self.subject,
                        error = %e,
                        "Failed to publish message to NATS"
                    );
                    NatsAdapterError::from(e)
                })?;

            Ok(())
        }
    }

    pub fn spawn_bridge(&self, mut rx: mpsc::UnboundedReceiver<T>) -> tokio::task::JoinHandle<()> {
        let publisher = self.clone();
        tokio::spawn(async move {
            while let Some(payload) = rx.recv().await {
                if let Err(e) = publisher.publish(&payload).await {
                    error!(error = %e, "Failed to publish message in bridge");
                }
            }
            info!(subject = %publisher.subject, "Channel bridge closed");
        })
    }
}

impl<T> Clone for Publisher<T> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            subject: self.subject.clone(),
            source: self.source.clone(),
            message_type: self.message_type.clone(),
            dlq_service: self.dlq_service.clone(),
            _phantom: PhantomData,
        }
    }
}

/// Subscriber that bridges NATS to a Tokio channel.
pub struct Subscriber<T> {
    client: NatsClient,
    subject: String,
    _phantom: PhantomData<T>,
}

impl<T> Subscriber<T>
where
    T: ProstMessage + Default + Send + Sync + 'static,
{
    pub fn new(client: NatsClient, subject: impl Into<String>) -> Self {
        Self {
            client,
            subject: subject.into(),
            _phantom: PhantomData,
        }
    }

    pub async fn spawn_bridge(
        &self,
        tx: mpsc::UnboundedSender<T>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let subject = self.subject.clone();
        let mut subscriber = self
            .client
            .client()
            .subscribe(subject.clone())
            .await
            .map_err(|e| {
                error!(
                    subject = %subject,
                    error = %e,
                    "Failed to subscribe to NATS subject"
                );
                NatsAdapterError::from(e)
            })?;

        info!(subject = %subject, "Subscribed to NATS subject");

        let handle = tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                match extract_proto_payload::<T>(&msg.payload) {
                    Ok(payload) => {
                        if let Err(e) = tx.send(payload) {
                            warn!(
                                error = %e,
                                "Failed to send message to channel, channel closed"
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            subject = %subject,
                            "Failed to decode protobuf NATS message"
                        );
                    }
                }
            }
            info!(subject = %subject, "NATS subscription closed");
        });

        Ok(handle)
    }
}
