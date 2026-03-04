//! NATS request-reply RPC layer.
//!
//! Two encoding paths:
//!   - `request` / `serve`  -- JSON  (serde, backward-compat)
//!   - `request_proto` / `serve_proto` -- protobuf (prost, preferred)

use std::time::Duration;

use bytes::Bytes;
use futures::StreamExt;
use prost::Message as ProstMessage;
use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;

use crate::client::NatsClient;
use crate::error::{NatsAdapterError, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

// ---------------------------------------------------------------------------
// JSON path (backward-compatible)
// ---------------------------------------------------------------------------

pub async fn request<Req, Res>(client: &NatsClient, subject: &str, payload: &Req) -> Result<Res>
where
    Req: Serialize,
    Res: DeserializeOwned,
{
    request_with_timeout(client, subject, payload, DEFAULT_TIMEOUT).await
}

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
    let msg = tokio::time::timeout(
        timeout,
        client.client().request(subject.into(), Bytes::from(body)),
    )
    .await
    .map_err(|_| NatsAdapterError::Publish(format!("request to {subject}: timeout")))?
    .map_err(|e| NatsAdapterError::Publish(format!("request to {subject}: {e}")))?;

    serde_json::from_slice(&msg.payload).map_err(NatsAdapterError::Serialization)
}

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

// ---------------------------------------------------------------------------
// Protobuf path (preferred for cross-language services)
// ---------------------------------------------------------------------------

pub async fn request_proto<Req, Res>(
    client: &NatsClient,
    subject: &str,
    payload: &Req,
) -> Result<Res>
where
    Req: ProstMessage,
    Res: ProstMessage + Default,
{
    request_proto_with_timeout(client, subject, payload, DEFAULT_TIMEOUT).await
}

pub async fn request_proto_with_timeout<Req, Res>(
    client: &NatsClient,
    subject: &str,
    payload: &Req,
    timeout: Duration,
) -> Result<Res>
where
    Req: ProstMessage,
    Res: ProstMessage + Default,
{
    let body = payload.encode_to_vec();
    let msg = tokio::time::timeout(
        timeout,
        client.client().request(subject.into(), Bytes::from(body)),
    )
    .await
    .map_err(|_| NatsAdapterError::Publish(format!("proto request to {subject}: timeout")))?
    .map_err(|e| NatsAdapterError::Publish(format!("proto request to {subject}: {e}")))?;

    Res::decode(msg.payload.as_ref()).map_err(NatsAdapterError::ProtoDecode)
}

pub async fn serve_proto<Req, Res, F, Fut>(
    client: &NatsClient,
    subject: &str,
    handler: F,
) -> Result<tokio::task::JoinHandle<()>>
where
    Req: ProstMessage + Default + Send + 'static,
    Res: ProstMessage + Send + 'static,
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
            match Req::decode(msg.payload.as_ref()) {
                Ok(req) => {
                    let res = handler(req).await;
                    let body = res.encode_to_vec();
                    if let Err(e) = nc.publish(reply, Bytes::from(body)).await {
                        warn!("proto rpc reply publish: {e}");
                    }
                }
                Err(e) => warn!("proto rpc decode request: {e}"),
            }
        }
    });

    Ok(handle)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rpc_module_compiles() {
        // Compile-time test only; integration tests need a live NATS server
    }
}
