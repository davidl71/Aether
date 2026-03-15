//! Strategy run/stop controller used by backend_service.
//! Moved out of rest.rs when REST was removed (NATS-only backend).

use std::sync::Arc;
use tokio::sync::watch;

#[derive(Clone)]
pub struct StrategyController {
    tx: Arc<watch::Sender<bool>>,
}

impl StrategyController {
    pub fn new(tx: watch::Sender<bool>) -> Self {
        Self { tx: Arc::new(tx) }
    }

    pub fn start(&self) -> Result<(), watch::error::SendError<bool>> {
        self.tx.send(true)
    }

    pub fn stop(&self) -> Result<(), watch::error::SendError<bool>> {
        self.tx.send(false)
    }
}
