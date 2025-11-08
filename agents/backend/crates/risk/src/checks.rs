use crate::model::{RiskDecision, RiskLimit, RiskViolation};
use async_trait::async_trait;

#[async_trait]
pub trait RiskCheck: Send + Sync {
  async fn evaluate(&self, request: &RiskLimit) -> RiskDecision;
}

pub struct RiskEngine {
  checks: Vec<Box<dyn RiskCheck>>, 
}

impl RiskEngine {
  pub fn new(checks: Vec<Box<dyn RiskCheck>>) -> Self {
    Self { checks }
  }

  pub async fn verify(&self, request: &RiskLimit) -> RiskDecision {
    for check in &self.checks {
      let decision = check.evaluate(request).await;
      if !decision.allowed {
        return decision;
      }
    }

    RiskDecision {
      allowed: true,
      reason: None,
    }
  }

  pub fn record_violation(&self, violation: RiskViolation) {
    tracing::warn!(symbol = %violation.symbol, details = %violation.details, "risk violation recorded");
  }
}
