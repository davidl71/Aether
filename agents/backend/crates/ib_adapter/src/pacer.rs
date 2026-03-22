//! TWS API request pacer.
//!
//! Interactive Brokers enforces a rate limit of approximately 55 messages per second
//! per client connection. This module provides a rate limiter to respect that limit.
//!
//! **Usage:**
//! ```rust,ignore
//! let pacer = TwsPacer::default_ibkr();
//! pacer.acquire().await;
//! client.send_request().await;
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

/// Rate limiter for TWS API requests.
///
/// IBKR limit: ~55 messages/second per client connection.
/// Uses 50 req/s by default to stay safe.
pub struct TwsPacer {
    interval: Duration,
    last_request: Arc<Mutex<Instant>>,
}

impl TwsPacer {
    /// Creates a new pacer with the given requests per second.
    pub fn new(requests_per_second: usize) -> Self {
        let interval = Duration::from_secs_f64(1.0 / requests_per_second.max(1) as f64);
        Self {
            interval,
            last_request: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Creates a pacer with IBKR's default limit (50 req/s conservative).
    pub fn default_ibkr() -> Self {
        Self::new(50)
    }

    /// Acquires a permit, waiting if necessary to respect the rate limit.
    pub async fn acquire(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();

        if elapsed < self.interval {
            tokio::time::sleep(self.interval - elapsed).await;
        }

        *last = Instant::now();
    }

    /// Returns the configured interval between requests.
    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Default for TwsPacer {
    fn default() -> Self {
        Self::default_ibkr()
    }
}

impl std::fmt::Debug for TwsPacer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwsPacer")
            .field("interval_ms", &(self.interval.as_millis() as u64))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pacer_respects_interval() {
        let pacer = TwsPacer::new(100); // 100 req/s = 10ms interval

        let start = Instant::now();
        for _ in 0..5 {
            pacer.acquire().await;
        }
        let elapsed = start.elapsed();

        // Should take at least 40ms for 5 requests at 100 req/s
        assert!(
            elapsed >= Duration::from_millis(40),
            "elapsed={:?}, expected >=40ms",
            elapsed
        );
    }

    #[tokio::test]
    async fn pacer_debug_format() {
        let pacer = TwsPacer::new(50);
        let debug = format!("{:?}", pacer);
        assert!(debug.contains("interval_ms"));
    }
}
