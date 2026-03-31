//! NATS request/reply handlers for api.* subjects.
//!
//! This module is now a thin wrapper that delegates to domain-specific handlers
//! organized in the `handlers/` subdirectory. Each domain handler manages its
//! own subject subscriptions and message handling.
//!
//! Domains:
//! - discount_bank: api.discount_bank.*
//! - loans: api.loans.*
//! - fmp: api.fmp.*
//! - strategy: api.strategy.* (deprecated, read-only mode)
//! - finance_rates: api.finance_rates.*, api.yield_curve.*
//! - calculate: api.calculate.*
//! - admin: api.admin.*, api.snapshot.*, api.ib.*

use std::sync::Arc;

use broker_engine::BrokerEngine;
use market_data::FmpClient;
use nats_adapter::NatsClient;
use tracing::info;

use crate::handlers;
use crate::shared_state::SharedSnapshot;

/// Spawn all NATS API handlers across all domains.
pub fn spawn(
    nats_client: Arc<NatsClient>,
    loan_repo: Option<Arc<api::LoanRepository>>,
    fmp_client: Option<Arc<FmpClient>>,
    state: SharedSnapshot,
    yield_curve_refresh_tx: Option<tokio::sync::mpsc::Sender<()>>,
    broker_engine: Option<Arc<dyn BrokerEngine>>,
    backend_id: String,
) {
    let nc = nats_client.client().clone();

    // Admin handlers (snapshot, ib positions, set_mode)
    let nc_admin = nc.clone();
    let state_admin = state.clone();
    let backend_id_admin = backend_id.clone();
    tokio::spawn(async move {
        handlers::admin::spawn(nc_admin, state_admin, backend_id_admin);
    });

    // Discount Bank handlers
    let nc_discount = nc.clone();
    tokio::spawn(async move {
        handlers::discount_bank::spawn(nc_discount).await;
    });

    // Ledger handlers
    let nc_ledger = nc.clone();
    tokio::spawn(async move {
        handlers::ledger::spawn(nc_ledger).await;
    });

    // Loans handlers
    let nc_loans = nc.clone();
    tokio::spawn(async move {
        if let Some(repo) = loan_repo {
            handlers::loans::spawn(nc_loans, repo).await;
        } else {
            handlers::loans::spawn_unconfigured(nc_loans).await;
        }
    });

    // Strategy handlers (read-only mode)
    let nc_strategy = nats_client.client().clone();
    let state_strategy = state.clone();
    tokio::spawn(async move {
        handlers::strategy::spawn(nc_strategy, state_strategy, broker_engine).await;
    });

    // Finance Rates handlers
    let nc_finance = nats_client.client().clone();
    tokio::spawn(async move {
        handlers::finance_rates::spawn(nc_finance, yield_curve_refresh_tx).await;
    });

    // Calculate handlers
    let nc_calculate = nats_client.client().clone();
    tokio::spawn(async move {
        handlers::calculate::spawn(nc_calculate).await;
    });

    // FMP handlers
    let nc_fmp = nats_client.client().clone();
    if let Some(fmp) = fmp_client {
        tokio::spawn(async move {
            handlers::fmp::spawn(nc_fmp, fmp).await;
        });
        info!("NATS API handlers spawned (discount_bank, ledger, loans, fmp, finance_rates, calculate, strategy, admin)");
    } else {
        tokio::spawn(async move {
            handlers::fmp::spawn_unconfigured(nc_fmp).await;
        });
        info!("NATS API handlers spawned (discount_bank, ledger, loans, fmp_unconfigured, finance_rates, calculate, strategy, admin)");
    }
}
