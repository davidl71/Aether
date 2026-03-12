//! NATS request-reply RPC layer.
//!
//! Protobuf is the only supported wire encoding.

use std::time::Duration;

use bytes::Bytes;
use futures::StreamExt;
use prost::Message as ProstMessage;
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

#[cfg(test)]
mod tests {
    #[test]
    fn rpc_module_compiles() {
        // Compile-time test only; integration tests need a live NATS server
    }
}
