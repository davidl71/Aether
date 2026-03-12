//! Connection status events for the NATS badge in the status bar.
//!
//! Log events are handled by tui-logger (via tracing macros) — see the Logs tab.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionTarget {
    Nats,
}

impl ConnectionTarget {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Nats => "NATS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Starting,
    Connected,
    Retrying,
}

impl ConnectionState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Starting => "START",
            Self::Connected => "UP",
            Self::Retrying => "DOWN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    pub state: ConnectionState,
    pub detail: String,
}

impl ConnectionStatus {
    pub fn new(state: ConnectionState, detail: impl Into<String>) -> Self {
        Self {
            state,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Connection {
        target: ConnectionTarget,
        status: ConnectionStatus,
    },
}
