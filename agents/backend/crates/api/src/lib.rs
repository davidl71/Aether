pub mod alpaca_positions;
pub mod client_portal_options;
pub mod combo_strategy;
pub mod command_proto;
pub mod commands;
/// Credential storage and env bootstrap (`credential_store` crate), re-exported as `api::credentials`.
pub mod credentials {
    pub use credential_store::*;
}
pub mod discount_bank;
pub mod finance_rates;
mod health;
pub mod ib_positions;
pub mod loans;
pub mod mock_data;
pub mod project_paths;
pub mod quant;
pub mod shared_config;
pub mod snapshot_proto;
mod snapshot_view;
pub mod state;
mod strategy_controller;
pub mod yield_curve_proto;

pub use alpaca_positions::{AlpacaAccountInfo, AlpacaPositionSource};
pub use command_proto::{command_reply_from_system_command_event, system_command_event_to_proto};
pub use commands::{
    CommandContext, CommandEvent, CommandReply, CommandStatus, SnapshotPublishReply,
};
pub use health::backend_health_from_message;
pub use health::{
    BackendHealthState, HealthAggregateResponse, HealthAggregateState, NatsTransportHealthState,
    SharedHealthAggregate,
};
pub use ib_positions::{fetch_ib_positions, fetch_ib_positions_all, IbPositionDto};
pub use loans::{
    parse_loans_import_file_json, BulkImportRowError, LoanAggregationInput, LoanRecord,
    LoanRepository, LoanStatus, LoanType, LoansBulkImportRequest, LoansBulkImportResponse,
};
pub use shared_config::{
    load_shared_config, read_shared_config_at, validate_shared_config, write_example_shared_config,
    LoadedSharedConfig,
};
pub use snapshot_view::{
    RuntimeDecisionDto, RuntimeHistoricPositionDto, RuntimeOrderDto, RuntimePositionDto,
    RuntimeSnapshotDto, ScenarioDto,
};
pub use state::*;
pub use strategy_controller::StrategyController;
