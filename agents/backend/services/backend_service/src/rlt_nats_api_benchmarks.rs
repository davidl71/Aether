use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use nats_adapter::{request_json_with_timeout, topics, NatsClient};
use rlt::{cli::BenchCli, BenchSuite, IterInfo, IterReport, Status};
use tokio::time::Instant;

/// Dev-only NATS request/reply load test using `rlt`.
///
/// Targets `api.finance_rates.benchmarks` by default. Requires:
/// - a running NATS server
/// - a running `backend_service` subscribed to `api.*`
#[derive(Parser, Clone)]
struct Opts {
    /// NATS URL.
    #[arg(long, default_value = "nats://localhost:4222")]
    nats_url: String,

    /// NATS subject to request/reply against.
    #[arg(long, default_value = topics::api::finance_rates::BENCHMARKS)]
    subject: String,

    /// Per-request timeout.
    #[arg(long, default_value = "5s", value_parser = humantime::parse_duration)]
    timeout: Duration,

    /// rlt runner options (concurrency, duration, output, etc.).
    #[command(flatten)]
    bench_opts: BenchCli,
}

#[derive(Clone)]
struct NatsRequestReplyBench {
    nats_url: String,
    subject: String,
    timeout: Duration,
}

#[async_trait]
impl BenchSuite for NatsRequestReplyBench {
    type WorkerState = NatsClient;

    async fn state(&self, _worker_id: u32) -> Result<Self::WorkerState> {
        Ok(NatsClient::connect(&self.nats_url).await?)
    }

    async fn bench(&mut self, state: &mut Self::WorkerState, _: &IterInfo) -> Result<IterReport> {
        let t0 = Instant::now();
        let response: serde_json::Value =
            request_json_with_timeout(state, &self.subject, &(), self.timeout).await?;
        let duration = t0.elapsed();

        let bytes = serde_json::to_vec(&response).map(|b| b.len() as u64).unwrap_or(0);
        Ok(IterReport {
            duration,
            status: Status::success(0),
            bytes,
            items: 1,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut opts = Opts::parse();
    if let Ok(v) = std::env::var("NATS_URL") {
        if !v.trim().is_empty() {
            opts.nats_url = v;
        }
    }
    let suite = NatsRequestReplyBench {
        nats_url: opts.nats_url,
        subject: opts.subject,
        timeout: opts.timeout,
    };

    rlt::cli::run(opts.bench_opts, suite).await
}

