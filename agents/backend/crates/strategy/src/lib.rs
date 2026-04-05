#![warn(missing_docs)]
//! Lightweight strategy signal and decision types shared by services.
//!
//! Execution is gated by product mode; this crate holds **data shapes** only. See `AGENTS.md`
//! and `docs/DATA_EXPLORATION_MODE.md` for current behavior.

pub mod model;

pub use model::{Decision, StrategySignal};
