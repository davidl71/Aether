use std::{collections::HashSet, net::SocketAddr, pin::Pin, sync::Arc, time::Duration};

use anyhow::Context;
use chrono::Utc;
use futures::{Stream, StreamExt};
use market_data::{MarketDataEvent, MarketDataIngestor, MarketDataSource, MockMarketDataSource};
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{transport::Server, Request, Response, Status};
use tonic_health::server::health_reporter;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

pub mod proto {
  pub mod v1 {
    tonic::include_proto!("ib.market_data.v1");
  }
}

use proto::v1::market_data_service_server::{MarketDataService, MarketDataServiceServer};
use proto::v1::{MarketDataEvent as ProtoMarketDataEvent, MarketDataRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_tracing();

  let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 50061));
  let (health_reporter, health_service) = health_reporter();

  let (tx, _) = broadcast::channel(512);
  let producer = spawn_producer(MockMarketDataSource::new(
    ["XSP", "SPY", "QQQ", "IWM"],
    Duration::from_millis(500),
  ), tx.clone());

  info!(%addr, "starting market data gRPC service");

  let svc = MarketDataServiceImpl::new(tx, health_reporter);
  Server::builder()
    .add_service(MarketDataServiceServer::new(svc))
    .add_service(health_service)
    .serve(addr)
    .await
    .context("market data gRPC server crashed")?;

  producer.abort();

  Ok(())
}

fn init_tracing() {
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
    .with_target(false)
    .init();
}

fn spawn_producer<S>(source: S, tx: broadcast::Sender<MarketDataEvent>) -> JoinHandle<()>
where
  S: MarketDataSource + Send + Sync + 'static,
{
  tokio::spawn(async move {
    let ingestor = MarketDataIngestor::new(source);

    loop {
      match ingestor.poll().await {
        Ok(event) => {
          if let Err(err) = tx.send(event.clone()) {
            warn!(%err, "market data broadcast channel closed");
            break;
          }
        }
        Err(err) => {
          warn!(%err, "market data ingestion failed, retrying");
          tokio::time::sleep(Duration::from_secs(1)).await;
        }
      }
    }
  })
}

#[derive(Clone)]
struct MarketDataServiceImpl {
  tx: broadcast::Sender<MarketDataEvent>,
  health: tonic_health::server::HealthReporter,
}

impl MarketDataServiceImpl {
  fn new(
    tx: broadcast::Sender<MarketDataEvent>,
    health: tonic_health::server::HealthReporter,
  ) -> Self {
    Self { tx, health }
  }
}

#[tonic::async_trait]
impl MarketDataService for MarketDataServiceImpl {
  type StreamEventsStream = PinStream;

  async fn stream_events(
    &self,
    request: Request<MarketDataRequest>,
  ) -> Result<Response<Self::StreamEventsStream>, Status> {
    let req = request.into_inner();
    let allowed_symbols: Option<HashSet<String>> = if req.symbols.is_empty() {
      None
    } else {
      Some(req.symbols.into_iter().collect())
    };

    let rx = self.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |item| {
      let allowed_symbols = allowed_symbols.clone();
      async move {
        match item {
          Ok(event) => {
            if let Some(ref symbols) = allowed_symbols {
              if !symbols.contains(&event.symbol) {
                return None;
              }
            }
            Some(Ok(map_event(&event)))
          }
          Err(broadcast::error::RecvError::Lagged(_)) => None,
          Err(broadcast::error::RecvError::Closed) => Some(Err(Status::cancelled("feed closed"))),
        }
      }
    });

    if let Err(err) = self.health.set_serving("marketdata").await {
      warn!(%err, "failed to set health status");
    }

    Ok(Response::new(Box::pin(stream)))
  }
}

type PinStream = Pin<Box<dyn Stream<Item = Result<ProtoMarketDataEvent, Status>> + Send>>;

fn map_event(event: &MarketDataEvent) -> ProtoMarketDataEvent {
  ProtoMarketDataEvent {
    symbol: event.symbol.clone(),
    bid: event.bid,
    ask: event.ask,
    timestamp: Utc::now().to_rfc3339(),
  }
}
