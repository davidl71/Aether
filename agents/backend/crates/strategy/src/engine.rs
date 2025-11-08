use crate::model::{Decision, StrategySignal};
use anyhow::Context;
use pyo3::prelude::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::info;

pub struct StrategyEngine {
  event_tx: UnboundedSender<StrategySignal>,
  decision_rx: UnboundedReceiver<Decision>,
  _py_strategy: Py<PyAny>,
}

impl StrategyEngine {
  pub async fn bootstrap(py_module_path: &str, strategy_name: &str) -> anyhow::Result<Self> {
    let (event_tx, _event_rx) = unbounded_channel();
    let (_decision_tx, decision_rx) = unbounded_channel();

    pyo3::prepare_freethreaded_python();

    let module_path = py_module_path.to_owned();
    let strategy_name = strategy_name.to_owned();

    let py_strategy = Python::with_gil(|py| -> anyhow::Result<Py<PyAny>> {
      let module = PyModule::import(py, &module_path)
        .with_context(|| format!("failed to import Python module {module_path}"))?;
      let strategy_cls = module
        .getattr(&strategy_name)
        .with_context(|| format!("strategy class {strategy_name} missing from module"))?;
      let instance = strategy_cls
        .call0()
        .context("failed to instantiate Python strategy")?;
      Ok(instance.into())
    })?;

    info!(module = %module_path, strategy = %strategy_name, "initialised strategy scaffold");

    Ok(Self {
      event_tx,
      decision_rx,
      _py_strategy: py_strategy,
    })
  }

  pub fn sender(&self) -> UnboundedSender<StrategySignal> {
    self.event_tx.clone()
  }

  pub async fn next_decision(&mut self) -> anyhow::Result<Option<Decision>> {
    // TODO: wire to Python via pyo3-asyncio bridge
    Ok(self.decision_rx.recv().await)
  }
}
