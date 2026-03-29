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
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::warn;

/// Default concurrency limit for parallel message handling.
pub const DEFAULT_CONCURRENCY_LIMIT: usize = 100;

/// Get concurrency limit from environment or use default.
pub fn concurrency_limit() -> usize {
    std::env::var("NATS_HANDLER_CONCURRENCY_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_CONCURRENCY_LIMIT)
}

/// Helper function to handle subscription with a closure - sequential processing.
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

/// Helper function to handle subscription with bounded parallelism.
pub async fn handle_sub_parallel<F, Fut>(
    nc: Client,
    mut sub: nats_adapter::async_nats::Subscriber,
    handler: F,
    limit: usize,
) where
    F: Fn(Option<Vec<u8>>) -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Vec<u8>> + Send + 'static,
{
    let semaphore = Arc::new(Semaphore::new(limit));

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

        let permit = match semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, "failed to acquire semaphore permit");
                continue;
            }
        };

        let nc = nc.clone();
        let handler = handler.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let response = handler(body).await;
            if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
                warn!(error = %e, "reply publish failed");
            }
        });
    }
}

/// Run CPU-bound work on Tokio's blocking thread pool (not trading execution).
pub async fn spawn_cpu_work<F>(work: F) -> Vec<u8>
where
    F: FnOnce() -> Vec<u8> + Send + 'static,
{
    tokio::task::spawn_blocking(work).await.unwrap_or_else(|e| {
        warn!(error = %e, "CPU work panicked");
        b"{\"error\":\"internal error\"}".to_vec()
    })
}

/// Builder for handler configuration.
#[derive(Clone, Debug)]
pub struct HandlerConfig {
    pub concurrency_limit: usize,
    pub timeout_secs: Option<u64>,
    pub use_blocking: bool,
}

impl Default for HandlerConfig {
    fn default() -> Self {
        Self {
            concurrency_limit: DEFAULT_CONCURRENCY_LIMIT,
            timeout_secs: None,
            use_blocking: false,
        }
    }
}

impl HandlerConfig {
    pub fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.concurrency_limit = limit;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn with_blocking(mut self, blocking: bool) -> Self {
        self.use_blocking = blocking;
        self
    }
}
