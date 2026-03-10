use crate::model::{Decision, StrategySignal, TradeSide};
use anyhow::Context;
use pyo3::prelude::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, info, warn};

pub struct StrategyEngine {
    event_tx: UnboundedSender<StrategySignal>,
    event_rx: UnboundedReceiver<StrategySignal>,
    decision_tx: UnboundedSender<Decision>,
    decision_rx: UnboundedReceiver<Decision>,
    py_strategy: Py<PyAny>,
}

impl StrategyEngine {
    pub async fn bootstrap(py_module_path: &str, strategy_name: &str) -> anyhow::Result<Self> {
        let (event_tx, event_rx) = unbounded_channel();
        let (decision_tx, decision_rx) = unbounded_channel();

        pyo3::prepare_freethreaded_python();

        let module_path = py_module_path.to_owned();
        let strategy_name = strategy_name.to_owned();

        let py_strategy = Python::with_gil(|py| -> anyhow::Result<Py<PyAny>> {
            let module = PyModule::import(py, module_path.as_str())
                .with_context(|| format!("failed to import Python module {module_path}"))?;
            let strategy_cls = module
                .getattr(strategy_name.as_str())
                .with_context(|| format!("strategy class {strategy_name} missing from module"))?;
            let instance = strategy_cls
                .call0()
                .context("failed to instantiate Python strategy")?;
            Ok(instance.unbind())
        })?;

        info!(module = %module_path, strategy = %strategy_name, "initialised strategy engine with Python bridge");

        Ok(Self {
            event_tx,
            event_rx,
            decision_tx,
            decision_rx,
            py_strategy,
        })
    }

    pub fn sender(&self) -> UnboundedSender<StrategySignal> {
        self.event_tx.clone()
    }

    /// Process pending signals by forwarding them to the Python strategy
    /// and converting Python responses into `Decision` values.
    pub async fn process_signals(&mut self) -> anyhow::Result<usize> {
        let mut processed = 0usize;

        while let Ok(signal) = self.event_rx.try_recv() {
            debug!(symbol = %signal.symbol, price = %signal.price, "forwarding signal to Python strategy");

            let decision_opt = Python::with_gil(|py| -> anyhow::Result<Option<Decision>> {
                let kwargs = pyo3::types::PyDict::new(py);
                kwargs.set_item("symbol", &signal.symbol)?;
                kwargs.set_item("price", signal.price)?;
                kwargs.set_item("timestamp", signal.timestamp.to_rfc3339())?;

                let result = self
                    .py_strategy
                    .call_method(py, "on_signal", (), Some(&kwargs));

                match result {
                    Ok(py_result) => {
                        if py_result.is_none(py) {
                            return Ok(None);
                        }
                        let side_str: String = py_result
                            .getattr(py, "side")
                            .and_then(|v| v.extract(py))
                            .unwrap_or_else(|_| "buy".to_string());
                        let quantity: i32 = py_result
                            .getattr(py, "quantity")
                            .and_then(|v| v.extract(py))
                            .unwrap_or(0);
                        let symbol: String = py_result
                            .getattr(py, "symbol")
                            .and_then(|v| v.extract(py))
                            .unwrap_or_else(|_| signal.symbol.clone());

                        let side = if side_str.eq_ignore_ascii_case("sell") {
                            TradeSide::Sell
                        } else {
                            TradeSide::Buy
                        };

                        Ok(Some(Decision {
                            symbol,
                            quantity,
                            side,
                        }))
                    }
                    Err(e) => {
                        warn!(error = %e, symbol = %signal.symbol, "Python strategy raised exception");
                        Ok(None)
                    }
                }
            })?;

            if let Some(decision) = decision_opt {
                debug!(symbol = %decision.symbol, qty = decision.quantity, "decision from Python strategy");
                let _ = self.decision_tx.send(decision);
            }

            processed += 1;
        }

        Ok(processed)
    }

    pub async fn next_decision(&mut self) -> anyhow::Result<Option<Decision>> {
        // Process any pending signals through the Python bridge first
        let _ = self.process_signals().await;
        Ok(self.decision_rx.recv().await)
    }
}
