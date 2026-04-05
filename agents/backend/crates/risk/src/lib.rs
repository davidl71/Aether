//! Risk limits, checks, VaR-related stats, sizing, and portfolio rollups.
//!
//! The [`quant`] submodule re-exports selected types from the **`quant` dependency crate** for a
//! narrow API surface (`risk::quant::Greeks`, etc.).
//!
//! # Submodules
//!
//! - [`calculator`] — [`RiskCalculator`]: orchestrates limit checks and rollups.
//! - [`checks`] / [`limits`] — [`RiskEngine`], [`LimitsChecker`], limit configuration.
//! - [`model`] — Portfolio/position risk DTOs and decisions ([`PortfolioRisk`], [`RiskViolation`], …).
//! - [`sizing`] — Position sizing helpers.
//! - [`stats`] — Summary statistics used in risk views.
//! - [`var`] — Value-at-Risk style helpers (where implemented).
//!
//! # See also
//!
//! - `AGENTS.md` (ownership).
//! - Pricing inputs: the **`quant` workspace crate** (distinct from [`quant`], which re-exports only
//!   a few types for convenience).

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
