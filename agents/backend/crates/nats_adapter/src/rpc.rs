//! NATS request-reply RPC layer.
//!
//! Supports protobuf (primary) and JSON wire encodings.

use std::time::Duration;

use bytes::Bytes;
use futures::StreamExt;
use prost::Message as ProstMessage;
use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;

use crate::client::NatsClient;
use crate::error::{NatsAdapterError, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

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
    let subject_owned = subject.to_string();
    let msg = tokio::time::timeout(
        timeout,
        client.client().request(subject_owned.clone(), Bytes::from(body)),
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
    let subject_owned = subject.to_string();
    let mut sub = client
        .client()
        .subscribe(subject_owned.clone())
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

/// JSON request/reply: send a JSON request and wait for a JSON response.
pub async fn request_json<Req, Res>(
    client: &NatsClient,
    subject: &str,
    payload: &Req,
) -> Result<Res>
where
    Req: Serialize,
    Res: DeserializeOwned,
{
    request_json_with_timeout(client, subject, payload, DEFAULT_TIMEOUT).await
}

/// JSON request/reply with custom timeout.
pub async fn request_json_with_timeout<Req, Res>(
    client: &NatsClient,
    subject: &str,
    payload: &Req,
    timeout: Duration,
) -> Result<Res>
where
    Req: Serialize,
    Res: DeserializeOwned,
{
    let body = serde_json::to_vec(payload).map_err(|e| NatsAdapterError::Publish(format!("json encode: {e}")))?;
    let subject_owned = subject.to_string();
    let msg = tokio::time::timeout(
        timeout,
        client.client().request(subject_owned.clone(), Bytes::from(body)),
    )
    .await
    .map_err(|_| NatsAdapterError::Publish(format!("json request to {subject}: timeout")))?
    .map_err(|e| NatsAdapterError::Publish(format!("json request to {subject}: {e}")))?;

    serde_json::from_slice(msg.payload.as_ref()).map_err(|e| NatsAdapterError::Publish(format!("json decode reply: {e}")))
}

/// Spawn a subscription that handles JSON request/reply for a subject.
pub async fn serve_json<Req, Res, F, Fut>(
    client: &NatsClient,
    subject: &str,
    handler: F,
) -> Result<tokio::task::JoinHandle<()>>
where
    Req: DeserializeOwned + Send + 'static,
    Res: Serialize + Send + 'static,
    F: Fn(Req) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = std::result::Result<Res, String>> + Send + 'static,
{
    let subject_owned = subject.to_string();
    let mut sub = client
        .client()
        .subscribe(subject_owned.clone())
        .await
        .map_err(|e| NatsAdapterError::Subscribe(format!("{e}")))?;

    let nc = client.client().clone();
    let handle = tokio::spawn(async move {
        while let Some(msg) = sub.next().await {
            let reply = match msg.reply {
                Some(ref r) => r.clone(),
                None => continue,
            };
            let req: Req = match serde_json::from_slice(msg.payload.as_ref()) {
                Ok(r) => r,
                Err(e) => {
                    warn!("json rpc decode request: {e}");
                    continue;
                }
            };
            match handler(req).await {
                Ok(res) => {
                    let body = match serde_json::to_vec(&res) {
                        Ok(b) => b,
                        Err(e) => {
                            warn!("json rpc encode reply: {e}");
                            continue;
                        }
                    };
                    if let Err(e) = nc.publish(reply, Bytes::from(body)).await {
                        warn!("json rpc reply publish: {e}");
                    }
                }
                Err(e) => warn!("json rpc handler error: {e}"),
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
