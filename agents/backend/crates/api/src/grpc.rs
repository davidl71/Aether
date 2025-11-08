use std::net::SocketAddr;
use std::pin::Pin;

use futures::Stream;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tonic::{transport::Server, Request, Response, Status};
use tonic_health::server::health_reporter;
use tracing::info;

use crate::ib_backend_proto::v1::strategy_service_server::{StrategyService, StrategyServiceServer};
use crate::ib_backend_proto::v1::{StrategyDecision, StrategyRequest};
use crate::state::SharedSnapshot;

pub struct GrpcServer;

impl GrpcServer {
  pub async fn serve(
    addr: SocketAddr,
    state: SharedSnapshot,
    decisions: broadcast::Sender<StrategyDecision>,
  ) -> anyhow::Result<JoinHandle<()>> {
    info!(%addr, "starting gRPC server");
    let (mut reporter, health_service) = health_reporter();
    if let Err(err) = reporter.set_serving("nautilus").await {
      tracing::warn!(%err, "failed to set initial health status");
    }

    let service = StrategyStreamService { state, decisions };

    let handle = tokio::spawn(async move {
      Server::builder()
        .add_service(StrategyServiceServer::new(service))
        .add_service(health_service)
        .serve(addr)
        .await
        .expect("gRPC server crashed");
    });
    Ok(handle)
  }
}

#[derive(Clone)]
struct StrategyStreamService {
  #[allow(dead_code)]
  state: SharedSnapshot,
  decisions: broadcast::Sender<StrategyDecision>,
}

#[tonic::async_trait]
impl StrategyService for StrategyStreamService {
  type StreamDecisionsStream = Pin<Box<dyn Stream<Item = Result<StrategyDecision, Status>> + Send + 'static>>;

  async fn stream_decisions(
    &self,
    _request: Request<StrategyRequest>,
  ) -> Result<Response<Self::StreamDecisionsStream>, Status> {
    let rx = self.decisions.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| async move {
      match result {
        Ok(decision) => Some(Ok(decision)),
        Err(broadcast::error::RecvError::Lagged(_)) => None,
        Err(broadcast::error::RecvError::Closed) => Some(Err(Status::cancelled("decision feed closed"))),
      }
    });

    Ok(Response::new(Box::pin(stream) as Self::StreamDecisionsStream))
  }
}
