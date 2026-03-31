use std::sync::Arc;
use std::time::Duration;

use backoff::backoff::Backoff;
use backoff::exponential::ExponentialBackoffBuilder;
use futures::future::BoxFuture;
use tracing::{info, warn};

const MAX_BACKOFF: Duration = Duration::from_secs(60);

pub type TaskFactory = Arc<dyn Fn() -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>;

/// Spawn a task that supervises a long-lived async loop.
///
/// - If the task returns `Err` or panics, it is restarted with exponential backoff.
/// - If the task returns `Ok(())`, it is treated as an unexpected exit and is restarted (also with backoff).
///
/// This is a small safety net for subscription loops and other "should never die" background tasks.
pub fn spawn_supervised(name: &'static str, factory: TaskFactory) {
    tokio::spawn(async move {
        let mut backoff: backoff::ExponentialBackoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_secs(2))
            .with_max_interval(MAX_BACKOFF)
            .build();

        loop {
            let join = tokio::spawn((factory)());
            match join.await {
                Ok(Ok(())) => {
                    warn!(task = name, "supervised task exited cleanly; restarting");
                }
                Ok(Err(e)) => {
                    warn!(task = name, error = %e, "supervised task failed; restarting");
                }
                Err(e) => {
                    warn!(task = name, error = %e, "supervised task panicked/cancelled; restarting");
                }
            }

            let delay = backoff.next_backoff().unwrap_or(MAX_BACKOFF);
            info!(
                task = name,
                delay_secs = delay.as_secs(),
                "supervisor backoff before restart"
            );
            tokio::time::sleep(delay).await;
        }
    });
}
