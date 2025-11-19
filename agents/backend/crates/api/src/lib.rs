pub mod grpc;
pub mod rest;
pub mod state;

#[cfg(test)]
mod ledger_integration_test;

pub mod ib_backend_proto {
  pub mod v1 {
    tonic::include_proto!("ib.backend.v1");
  }
}

pub use grpc::GrpcServer;
pub use rest::{RestServer, RestState, StrategyController};
pub use state::*;
