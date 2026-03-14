//! TUI-local data models derived from the backend snapshot.

use chrono::{DateTime, Utc};

use api::RuntimeSnapshotDto;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // TODO(T-1773357423959019000): remove when REST fallback is wired.
pub enum SnapshotSource {
    Nats,
    /// REST fallback source — only accepted when NATS snapshot is stale.
    Rest,
}

impl SnapshotSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Nats => "NATS",
            Self::Rest => "REST",
        }
    }
}

/// Latest snapshot received by the TUI plus metadata about how fresh it is.
#[derive(Debug, Clone)]
pub struct TuiSnapshot {
    pub inner: RuntimeSnapshotDto,
    pub received_at: DateTime<Utc>,
    pub source: SnapshotSource,
}

impl TuiSnapshot {
    pub fn new(inner: RuntimeSnapshotDto, source: SnapshotSource) -> Self {
        Self {
            inner,
            received_at: Utc::now(),
            source,
        }
    }

    /// Seconds since this snapshot was received.
    pub fn age_secs(&self) -> i64 {
        (Utc::now() - self.received_at).num_seconds()
    }

    /// True when the snapshot is older than `threshold_secs`.
    pub fn is_stale(&self, threshold_secs: i64) -> bool {
        self.age_secs() > threshold_secs
    }
}
