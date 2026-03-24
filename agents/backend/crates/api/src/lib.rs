pub mod client_portal_options;
pub mod combo_strategy;
pub mod credentials;
pub mod discount_bank;
pub mod finance_rates;
mod health;
pub mod ib_positions;
pub mod loans;
pub mod mock_data;
pub mod project_paths;
pub mod quant;
mod runtime_state;
pub mod shared_config;
pub mod snapshot_proto;
pub mod state;
mod strategy_controller;
pub mod yield_curve_proto;

#[cfg(test)]
mod ledger_integration_test;

pub use health::backend_health_from_message;
pub use health::{
    BackendHealthState, HealthAggregateResponse, HealthAggregateState, SharedHealthAggregate,
};
pub use ib_positions::{fetch_ib_positions, fetch_ib_positions_all, IbPositionDto};
pub use loans::{LoanAggregationInput, LoanRecord, LoanRepository, LoanStatus, LoanType};
pub use runtime_state::{
    ProducerMetadata, ProducerType, RuntimeDecisionDto, RuntimeExecutionState,
    RuntimeHistoricPositionDto, RuntimeMarketState, RuntimeOrderDto, RuntimePositionDto,
    RuntimeProducerDecision, RuntimeRiskState, RuntimeSnapshotDto, ScenarioDto,
};
pub use shared_config::{
    load_shared_config, read_shared_config_at, validate_shared_config, write_example_shared_config,
    LoadedSharedConfig,
};
pub use state::*;
pub use strategy_controller::StrategyController;
