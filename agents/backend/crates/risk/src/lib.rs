//! Risk limits, checks, VaR-related stats, sizing, and portfolio rollups.
//!
//! The [`quant`] submodule re-exports selected types from the **`quant` dependency crate** for a
//! narrow API surface (`risk::quant::Greeks`, etc.).
//!
//! # Repository documentation
//!
//! Ownership: `AGENTS.md` (repo root).

pub mod calculator;
pub mod checks;
pub mod limits;
pub mod model;
pub mod sizing;
pub mod stats;
pub mod var;

pub use calculator::RiskCalculator;
pub use checks::{RiskCheck, RiskEngine};
pub use limits::{LimitsChecker, RiskLimits};
pub use model::{
    BoxSpreadLeg, PortfolioRisk, PositionRisk, RiskDecision, RiskLimit, RiskViolation,
};

pub mod quant {
    pub use quant::{Greeks, OptionKind, Position, QuantCalculator};
}
