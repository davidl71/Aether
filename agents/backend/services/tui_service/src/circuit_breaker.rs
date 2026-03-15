//! Circuit breaker with exponential backoff for NATS reconnects.
//!
//! State transitions:
//!   Closed  + failures >= THRESHOLD  →  Open (pause OPEN_DURATION)
//!   Open    + timeout elapsed        →  HalfOpen (try one connection)
//!   HalfOpen + success               →  Closed (reset all counters)
//!   HalfOpen + failure               →  Open (restart timer)
//!
//! Backoff schedule uses the `backoff` crate (2s initial, 60s max, 2x multiplier).

use std::time::{Duration, Instant};

use backoff::backoff::Backoff;
use backoff::exponential::ExponentialBackoffBuilder;

const FAILURE_THRESHOLD: u32 = 3;
const OPEN_DURATION: Duration = Duration::from_secs(30);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

fn make_backoff() -> backoff::ExponentialBackoff {
    ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(2))
        .with_multiplier(2.0)
        .with_max_interval(MAX_BACKOFF)
        .with_randomization_factor(0.0)
        .with_max_elapsed_time(None)
        .build()
}

enum State {
    Closed,
    Open { until: Instant },
    HalfOpen,
}

pub struct CircuitBreaker {
    state: State,
    consecutive_failures: u32,
    backoff: backoff::ExponentialBackoff,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: State::Closed,
            consecutive_failures: 0,
            backoff: make_backoff(),
        }
    }

    /// Whether a connection attempt is permitted right now.
    /// Transitions Open → HalfOpen when the pause duration has elapsed.
    pub fn can_attempt(&mut self) -> bool {
        if let State::Open { until } = self.state {
            if Instant::now() >= until {
                self.state = State::HalfOpen;
                return true;
            }
            return false;
        }
        true
    }

    pub fn record_success(&mut self) {
        self.state = State::Closed;
        self.consecutive_failures = 0;
        self.backoff.reset();
    }

    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= FAILURE_THRESHOLD {
            self.state = State::Open {
                until: Instant::now() + OPEN_DURATION,
            };
            self.consecutive_failures = 0;
        }
    }

    /// Next backoff duration (2s, 4s, 8s, … capped at 60s). Uses `backoff` crate.
    pub fn backoff(&mut self) -> Duration {
        self.backoff
            .next_backoff()
            .unwrap_or(MAX_BACKOFF)
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, State::Open { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_consecutive_failure_threshold() {
        let mut cb = CircuitBreaker::new();
        assert!(cb.can_attempt());
        for _ in 0..FAILURE_THRESHOLD {
            cb.record_failure();
        }
        assert!(cb.is_open());
        assert!(!cb.can_attempt());
    }

    #[test]
    fn resets_fully_on_success() {
        let mut cb = CircuitBreaker::new();
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert!(!cb.is_open());
        assert!(cb.can_attempt());
    }

    #[test]
    fn backoff_grows_exponentially() {
        let mut cb = CircuitBreaker::new();
        assert_eq!(cb.backoff(), Duration::from_secs(2));
        assert_eq!(cb.backoff(), Duration::from_secs(4));
        assert_eq!(cb.backoff(), Duration::from_secs(8));
        assert_eq!(cb.backoff(), Duration::from_secs(16));
    }

    #[test]
    fn backoff_caps_at_max() {
        let mut cb = CircuitBreaker::new();
        let mut last = Duration::ZERO;
        for _ in 0..20 {
            let d = cb.backoff();
            last = d;
            assert!(d <= MAX_BACKOFF, "backoff {} exceeded max", d.as_secs());
        }
        assert_eq!(last, MAX_BACKOFF);
    }
}
