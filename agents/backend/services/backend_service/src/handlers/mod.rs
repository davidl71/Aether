//! NATS request/reply handlers organized by domain surface.
//!
//! This module splits the monolithic api_handlers.rs into domain-specific modules:
//! - discount_bank: Discount bank operations (api.discount_bank.*)
//! - loans: Loan management (api.loans.*)
//! - fmp: Financial Modeling Prep integration (api.fmp.*)
//! - strategy: Strategy control (api.strategy.*) - read-only mode
//! - finance_rates: Yield curves and rate calculations (api.finance_rates.*)
//! - calculate: Quantitative calculations (api.calculate.*)
//! - admin: Administrative operations (api.admin.*, api.snapshot.*, api.ib.*)

pub mod admin;
pub mod calculate;
pub mod discount_bank;
pub mod finance_rates;
pub mod fmp;
pub mod loans;
pub mod strategy;

/// Default queue group for api.* request/reply when scaling multiple backends.
pub const DEFAULT_API_QUEUE_GROUP: &str = "api";

/// Get the API queue group from environment or use default.
pub fn api_queue_group() -> String {
    std::env::var("NATS_API_QUEUE_GROUP").unwrap_or_else(|_| DEFAULT_API_QUEUE_GROUP.into())
}

use bytes::Bytes;
use futures::StreamExt;
use nats_adapter::async_nats::Client;
use tracing::warn;

/// Helper function to handle subscription with a closure.
pub async fn handle_sub<F, Fut>(
    nc: Client,
    mut sub: nats_adapter::async_nats::Subscriber,
    handler: F,
) where
    F: Fn(Option<Vec<u8>>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Vec<u8>> + Send + 'static,
{
    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let body = if msg.payload.is_empty() {
            None
        } else {
            Some(msg.payload.to_vec())
        };
        let response = handler(body).await;
        if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
            warn!(error = %e, "reply publish failed");
        }
    }
}

/// Helper function to handle subscription with a reqwest client reference.
pub async fn handle_sub_with_client<F, Fut>(
    nc: Client,
    sub: nats_adapter::async_nats::Subscriber,
    client: reqwest::Client,
) where
    F: Fn(reqwest::Client, nats_adapter::async_nats::Subscriber) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    use futures::StreamExt;
    let mut sub = sub;
    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let body = msg.payload.to_vec();
        // Handler is responsible for processing and replying
    }
}
