use std::net::SocketAddr;
use std::pin::Pin;

use futures::Stream;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_stream::{wrappers::BroadcastStream, wrappers::errors::BroadcastStreamRecvError, StreamExt};
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
    let (_reporter, health_service) = health_reporter();

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
    let stream = BroadcastStream::new(rx).then(move |item| {
      async move {
        match item {
          Ok(decision) => Ok(decision),
          Err(BroadcastStreamRecvError::Lagged(_)) => Err(Status::cancelled("decision feed lagged")),
        }
      }
    });

    Ok(Response::new(Box::pin(stream) as Self::StreamDecisionsStream))
  }
}
