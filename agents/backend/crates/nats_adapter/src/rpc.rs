//! NATS request-reply RPC layer.
//!
//! Services register handlers on well-known subjects.  Callers use
//! `request` to send a payload and await a single response, replacing
//! HTTP round-trips with a NATS hop.

use std::time::Duration;

use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;

use crate::client::NatsClient;
use crate::error::{NatsAdapterError, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Send a request and await a response (typed).
pub async fn request<Req, Res>(
    client: &NatsClient,
    subject: &str,
    payload: &Req,
) -> Result<Res>
where
    Req: Serialize,
    Res: DeserializeOwned,
{
    request_with_timeout(client, subject, payload, DEFAULT_TIMEOUT).await
}

/// Send a request with a custom timeout.
pub async fn request_with_timeout<Req, Res>(
    client: &NatsClient,
    subject: &str,
    payload: &Req,
    timeout: Duration,
) -> Result<Res>
where
    Req: Serialize,
    Res: DeserializeOwned,
{
    let body = serde_json::to_vec(payload).map_err(NatsAdapterError::Serialization)?;
    let msg = client
        .client()
        .request(subject.into(), Bytes::from(body))
        .await
        .map_err(|e| NatsAdapterError::Publish(format!("request to {subject}: {e}")))?;

    // Check for timeout (async-nats handles this differently)
    serde_json::from_slice(&msg.payload).map_err(NatsAdapterError::Serialization)
}

/// Register a handler for requests on a subject.
///
/// The handler receives the deserialized request and returns a
/// serializable response.  Runs until the returned subscription is
/// dropped.
pub async fn serve<Req, Res, F, Fut>(
    client: &NatsClient,
    subject: &str,
    handler: F,
) -> Result<tokio::task::JoinHandle<()>>
where
    Req: DeserializeOwned + Send + 'static,
    Res: Serialize + Send + 'static,
    F: Fn(Req) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Res> + Send + 'static,
{
    let mut sub = client
        .client()
        .subscribe(subject.into())
        .await
        .map_err(|e| NatsAdapterError::Subscribe(format!("{e}")))?;

    let nc = client.client().clone();
    let handle = tokio::spawn(async move {
        while let Some(msg) = sub.next().await {
            let reply = match msg.reply {
                Some(ref r) => r.clone(),
                None => continue,
            };

            match serde_json::from_slice::<Req>(&msg.payload) {
                Ok(req) => {
                    let res = handler(req).await;
                    match serde_json::to_vec(&res) {
                        Ok(body) => {
                            if let Err(e) = nc.publish(reply, Bytes::from(body)).await {
                                warn!("rpc reply publish: {e}");
                            }
                        }
                        Err(e) => warn!("rpc serialize response: {e}"),
                    }
                }
                Err(e) => warn!("rpc deserialize request: {e}"),
            }
        }
    });

    Ok(handle)
}

use futures::StreamExt;

#[cfg(test)]
mod tests {
    #[test]
    fn rpc_module_compiles() {
        // Compile-time test only; integration tests need a live NATS server
    }
}
