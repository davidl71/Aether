pub mod discount_bank;
pub mod finance_rates;
mod health;
pub mod mock_data;
pub mod ib_positions;
mod loans;
pub mod project_paths;
pub mod quant;
pub mod shared_config;
mod runtime_state;
pub mod state;
mod strategy_controller;

#[cfg(test)]
mod ledger_integration_test;

pub use health::backend_health_from_message;
pub use health::{
    BackendHealthState, HealthAggregateResponse, HealthAggregateState, SharedHealthAggregate,
};
pub use ib_positions::{fetch_ib_positions, fetch_ib_positions_all, IbPositionDto};
pub use loans::{LoanAggregationInput, LoanRecord, LoanRepository, LoanStatus, LoanType};
pub use strategy_controller::StrategyController;
pub use runtime_state::{
    ProducerMetadata, ProducerType, RuntimeDecisionDto, RuntimeExecutionState,
    RuntimeHistoricPositionDto, RuntimeMarketState, RuntimeOrderDto, RuntimePositionDto,
    RuntimeProducerDecision, RuntimeRiskState, RuntimeSnapshotDto, ScenarioDto,
};
pub use state::*;
pub use shared_config::{load_shared_config, read_shared_config_at, write_example_shared_config, LoadedSharedConfig};
