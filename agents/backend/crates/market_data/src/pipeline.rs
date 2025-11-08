use crate::model::{MarketDataEvent, MarketDataSource};

pub struct MarketDataPipeline<S>
where
  S: MarketDataSource,
{
  source: S,
}

impl<S> MarketDataPipeline<S>
where
  S: MarketDataSource,
{
  pub fn new(source: S) -> Self {
    Self { source }
  }

  pub async fn next(&self) -> anyhow::Result<MarketDataEvent> {
    self.source.next().await
  }
}

pub struct MarketDataIngestor<S>
where
  S: MarketDataSource,
{
  pipeline: MarketDataPipeline<S>,
}

impl<S> MarketDataIngestor<S>
where
  S: MarketDataSource,
{
  pub fn new(source: S) -> Self {
    Self {
      pipeline: MarketDataPipeline::new(source),
    }
  }

  pub async fn poll(&self) -> anyhow::Result<MarketDataEvent> {
    self.pipeline.next().await
  }
}
