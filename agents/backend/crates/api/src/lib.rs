mod finance_rates;
mod health;
mod ib_positions;
mod discount_bank;
mod loans;
pub mod rest;
pub mod state;
pub mod websocket;

#[cfg(test)]
mod ledger_integration_test;

pub use loans::{LoanAggregationInput, LoanRecord, LoanRepository, LoanStatus, LoanType};
pub use ib_positions::IbPositionDto;
pub use health::{BackendHealthState, HealthAggregateResponse, HealthAggregateState, SharedHealthAggregate};
pub use health::backend_health_from_message;
pub use rest::{RestServer, RestState, StrategyController};
pub use state::*;
pub use websocket::WebSocketServer;
