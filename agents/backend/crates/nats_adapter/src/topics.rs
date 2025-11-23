//! NATS Topic Registry and Validation
//!
//! Centralized topic definitions and validation to prevent naming collisions
//! and ensure consistent topic usage across all components.

use crate::error::{NatsAdapterError, Result};

/// Topic domain prefixes
pub mod domain {
  pub const MARKET_DATA: &str = "market-data";
  pub const STRATEGY: &str = "strategy";
  pub const ORDERS: &str = "orders";
  pub const POSITIONS: &str = "positions";
  pub const RISK: &str = "risk";
  pub const SYSTEM: &str = "system";
  pub const RPC: &str = "rpc";
}

/// Market data topics
pub mod market_data {
  use super::domain;

  /// Real-time tick updates: `market-data.tick.{symbol}`
  pub fn tick(symbol: &str) -> String {
    format!("{}.tick.{}", domain::MARKET_DATA, symbol)
  }

  /// OHLCV candle updates: `market-data.candle.{symbol}`
  pub fn candle(symbol: &str) -> String {
    format!("{}.candle.{}", domain::MARKET_DATA, symbol)
  }

  /// Bid/ask quote updates: `market-data.quote.{symbol}`
  pub fn quote(symbol: &str) -> String {
    format!("{}.quote.{}", domain::MARKET_DATA, symbol)
  }

  /// Volume updates: `market-data.volume.{symbol}`
  pub fn volume(symbol: &str) -> String {
    format!("{}.volume.{}", domain::MARKET_DATA, symbol)
  }

  /// Subscribe to all market data for a symbol: `market-data.>`
  pub fn all_for_symbol(_symbol: &str) -> String {
    format!("{}.>", domain::MARKET_DATA)
  }

  /// Subscribe to all market data: `market-data.>`
  pub fn all() -> &'static str {
    "market-data.>"
  }
}

/// Strategy topics
pub mod strategy {
  use super::domain;

  /// Market signals: `strategy.signal.{symbol}`
  pub fn signal(symbol: &str) -> String {
    format!("{}.signal.{}", domain::STRATEGY, symbol)
  }

  /// Trading decisions: `strategy.decision.{symbol}`
  pub fn decision(symbol: &str) -> String {
    format!("{}.decision.{}", domain::STRATEGY, symbol)
  }

  /// Strategy status changes: `strategy.status`
  pub fn status() -> &'static str {
    "strategy.status"
  }

  /// Control commands: `strategy.control`
  pub fn control() -> &'static str {
    "strategy.control"
  }

  /// Subscribe to all strategy signals: `strategy.signal.>`
  pub fn all_signals() -> &'static str {
    "strategy.signal.>"
  }

  /// Subscribe to all strategy decisions: `strategy.decision.>`
  pub fn all_decisions() -> &'static str {
    "strategy.decision.>"
  }
}

/// Order topics
pub mod orders {
  use super::domain;

  /// New order requests: `orders.new`
  pub fn new() -> &'static str {
    "orders.new"
  }

  /// Order status updates: `orders.status.{order_id}`
  pub fn status(order_id: &str) -> String {
    format!("{}.status.{}", domain::ORDERS, order_id)
  }

  /// Order fill notifications: `orders.fill.{order_id}`
  pub fn fill(order_id: &str) -> String {
    format!("{}.fill.{}", domain::ORDERS, order_id)
  }

  /// Order cancellation: `orders.cancel.{order_id}`
  pub fn cancel(order_id: &str) -> String {
    format!("{}.cancel.{}", domain::ORDERS, order_id)
  }

  /// Subscribe to all order status updates: `orders.status.>`
  pub fn all_status() -> &'static str {
    "orders.status.>"
  }
}

/// Position topics
pub mod positions {
  use super::domain;

  /// Position updates: `positions.update.{symbol}`
  pub fn update(symbol: &str) -> String {
    format!("{}.update.{}", domain::POSITIONS, symbol)
  }

  /// Position snapshot: `positions.snapshot`
  pub fn snapshot() -> &'static str {
    "positions.snapshot"
  }

  /// Subscribe to all position updates: `positions.update.>`
  pub fn all_updates() -> &'static str {
    "positions.update.>"
  }
}

/// Risk topics
pub mod risk {
  use super::domain;

  /// Risk check requests: `risk.check`
  pub fn check() -> &'static str {
    "risk.check"
  }

  /// Risk check results: `risk.decision`
  pub fn decision() -> &'static str {
    "risk.decision"
  }

  /// Risk limit events: `risk.limit.{type}`
  pub fn limit(limit_type: &str) -> String {
    format!("{}.limit.{}", domain::RISK, limit_type)
  }

  /// Risk violations: `risk.violation`
  pub fn violation() -> &'static str {
    "risk.violation"
  }
}

/// System topics
pub mod system {

  /// System health status: `system.health`
  pub fn health() -> &'static str {
    "system.health"
  }

  /// System-wide events: `system.events`
  pub fn events() -> &'static str {
    "system.events"
  }

  /// Alert notifications: `system.alerts`
  pub fn alerts() -> &'static str {
    "system.alerts"
  }

  /// Configuration updates: `system.config`
  pub fn config() -> &'static str {
    "system.config"
  }

  /// Service control commands: `system.service.{service_name}.{action}`
  /// Actions: start, stop, restart, enable, disable
  pub fn service_control(service_name: &str, action: &str) -> String {
    format!("system.service.{}.{}", service_name, action)
  }

  /// Service status updates: `system.service.{service_name}.status`
  pub fn service_status(service_name: &str) -> String {
    format!("system.service.{}.status", service_name)
  }

  /// Subscribe to all service control commands: `system.service.>`
  pub fn all_service_control() -> &'static str {
    "system.service.>"
  }
}

/// Dead Letter Queue (DLQ) topics
pub mod dlq {
  use super::domain;

  /// Dead letter queue topic: `system.dlq.{component}.{error_type}`
  ///
  /// # Arguments
  /// * `component` - Component that failed (e.g., "backend", "strategy", "market-data")
  /// * `error_type` - Type of error (e.g., "publish_error", "deserialization_error", "timeout")
  ///
  /// # Example
  /// ```
  /// use nats_adapter::topics;
  /// let dlq_topic = topics::dlq::dead_letter("backend", "publish_error");
  /// // Returns: "system.dlq.backend.publish_error"
  /// ```
  pub fn dead_letter(component: &str, error_type: &str) -> String {
    format!("{}.dlq.{}.{}", domain::SYSTEM, component, error_type)
  }

  /// Subscribe to all DLQ messages for a component: `system.dlq.{component}.>`
  pub fn component_dlq(component: &str) -> String {
    format!("{}.dlq.{}.>", domain::SYSTEM, component)
  }

  /// Subscribe to all DLQ messages: `system.dlq.>`
  pub fn all() -> &'static str {
    "system.dlq.>"
  }
}

/// RPC (Request/Reply) topics
pub mod rpc {

  /// Request strategy status: `rpc.strategy.status`
  pub fn strategy_status() -> &'static str {
    "rpc.strategy.status"
  }

  /// Request system snapshot: `rpc.system.snapshot`
  pub fn system_snapshot() -> &'static str {
    "rpc.system.snapshot"
  }
}

/// Validate a topic name
///
/// # Rules
/// - Must not be empty
/// - Must not start or end with `.`
/// - Must not contain consecutive `.`
/// - Must not contain invalid characters (only alphanumeric, `.`, `-`, `_`, `>`, `*`)
/// - Must not exceed 256 characters
///
/// # Returns
/// Ok(()) if valid, Err if invalid
pub fn validate_topic(topic: &str) -> Result<()> {
  if topic.is_empty() {
    return Err(NatsAdapterError::InvalidSubject("Topic cannot be empty".to_string()));
  }

  if topic.len() > 256 {
    return Err(NatsAdapterError::InvalidSubject(
      "Topic exceeds maximum length of 256 characters".to_string(),
    ));
  }

  if topic.starts_with('.') || topic.ends_with('.') {
    return Err(NatsAdapterError::InvalidSubject(
      "Topic cannot start or end with '.'".to_string(),
    ));
  }

  if topic.contains("..") {
    return Err(NatsAdapterError::InvalidSubject(
      "Topic cannot contain consecutive '.'".to_string(),
    ));
  }

  // Check for valid characters: alphanumeric, ., -, _, >, *
  for ch in topic.chars() {
    if !ch.is_alphanumeric()
      && ch != '.'
      && ch != '-'
      && ch != '_'
      && ch != '>'
      && ch != '*'
    {
      return Err(NatsAdapterError::InvalidSubject(format!(
        "Topic contains invalid character: '{}'",
        ch
      )));
    }
  }

  Ok(())
}

/// Check if a topic matches a pattern (supports wildcards)
///
/// # Wildcards
/// - `>` matches one or more tokens
/// - `*` matches exactly one token
pub fn topic_matches(pattern: &str, topic: &str) -> bool {
  // Simple implementation - can be enhanced
  if pattern == topic {
    return true;
  }

  // Handle wildcards
  if pattern.ends_with(".>") {
    let prefix = &pattern[..pattern.len() - 2];
    return topic.starts_with(prefix);
  }

  if pattern.contains('*') {
    // Simple wildcard matching
    let pattern_parts: Vec<&str> = pattern.split('.').collect();
    let topic_parts: Vec<&str> = topic.split('.').collect();

    if pattern_parts.len() != topic_parts.len() {
      return false;
    }

    for (p, t) in pattern_parts.iter().zip(topic_parts.iter()) {
      if *p != "*" && *p != *t {
        return false;
      }
    }
    return true;
  }

  false
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_validate_topic_valid() {
    assert!(validate_topic("market-data.tick.SPY").is_ok());
    assert!(validate_topic("strategy.signal.>").is_ok());
    assert!(validate_topic("orders.status.123").is_ok());
  }

  #[test]
  fn test_validate_topic_invalid() {
    assert!(validate_topic("").is_err());
    assert!(validate_topic(".invalid").is_err());
    assert!(validate_topic("invalid.").is_err());
    assert!(validate_topic("invalid..topic").is_err());
    assert!(validate_topic("invalid topic").is_err()); // space
    assert!(validate_topic(&"a".repeat(257)).is_err()); // too long
  }

  #[test]
  fn test_topic_matches() {
    assert!(topic_matches("market-data.tick.SPY", "market-data.tick.SPY"));
    assert!(topic_matches("market-data.>", "market-data.tick.SPY"));
    assert!(topic_matches("market-data.*.SPY", "market-data.tick.SPY"));
    assert!(!topic_matches("market-data.tick.SPY", "market-data.quote.SPY"));
  }

  #[test]
  fn test_topic_generation() {
    assert_eq!(market_data::tick("SPY"), "market-data.tick.SPY");
    assert_eq!(strategy::signal("XSP"), "strategy.signal.XSP");
    assert_eq!(orders::status("ORD-123"), "orders.status.ORD-123");
  }
}
