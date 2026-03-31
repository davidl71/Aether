//! Minimal exponential backoff utility.
//!
//! We previously used the `backoff` crate, but it pulled in unmaintained transitive deps
//! (notably `instant`). This module provides the small subset of functionality we use.

use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ExponentialBackoff
{
  initial_interval: Duration,
  current_interval: Duration,
  multiplier: f64,
  max_interval: Duration,
}

impl ExponentialBackoff
{
  pub fn next_backoff(&mut self) -> Option<Duration>
  {
    let out = self.current_interval;

    let next_secs = (out.as_secs_f64() * self.multiplier).max(0.0);
    let next = Duration::from_secs_f64(next_secs);
    self.current_interval = std::cmp::min(next, self.max_interval);

    Some(out)
  }

  pub fn reset(&mut self)
  {
    self.current_interval = self.initial_interval;
  }
}

#[derive(Clone, Debug)]
pub struct ExponentialBackoffBuilder
{
  initial_interval: Duration,
  multiplier: f64,
  max_interval: Duration,
}

impl ExponentialBackoffBuilder
{
  pub fn new() -> Self
  {
    Self {
      initial_interval: Duration::from_secs(1),
      multiplier: 2.0,
      max_interval: Duration::from_secs(60),
    }
  }

  pub fn with_initial_interval(mut self, d: Duration) -> Self
  {
    self.initial_interval = d;
    self
  }

  pub fn with_multiplier(mut self, m: f64) -> Self
  {
    self.multiplier = m;
    self
  }

  pub fn with_max_interval(mut self, d: Duration) -> Self
  {
    self.max_interval = d;
    self
  }

  // Compatibility no-ops (kept so call sites can remain unchanged).
  pub fn with_randomization_factor(self, _f: f64) -> Self
  {
    self
  }

  pub fn with_max_elapsed_time(self, _d: Option<Duration>) -> Self
  {
    self
  }

  pub fn build(self) -> ExponentialBackoff
  {
    ExponentialBackoff {
      initial_interval: self.initial_interval,
      current_interval: self.initial_interval,
      multiplier: self.multiplier,
      max_interval: self.max_interval,
    }
  }
}

