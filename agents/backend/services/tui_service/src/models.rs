//! TUI-local data models derived from the backend snapshot.
//!
//! Consumes `api::SystemSnapshot` from NATS; display uses a derived `RuntimeSnapshotDto`.

use chrono::{DateTime, Utc};

use api::{RuntimeSnapshotDto, SystemSnapshot};

/// Source of the snapshot. Backend is NATS-only; no REST fallback.
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotSource {
    Nats,
}

/// Latest snapshot received by the TUI (unified api::SystemSnapshot) plus display DTO and metadata.
#[derive(Clone)]
pub struct TuiSnapshot {
    /// Canonical snapshot from NATS (api::SystemSnapshot).
    pub inner: SystemSnapshot,
    /// Derived DTO for UI binding (positions, orders, metrics, etc.).
    display_dto: RuntimeSnapshotDto,
    pub received_at: DateTime<Utc>,
    pub source: SnapshotSource,
}

impl std::fmt::Debug for TuiSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TuiSnapshot")
            .field("received_at", &self.received_at)
            .field("source", &self.source)
            .finish()
    }
}

impl TuiSnapshot {
    pub fn new(inner: SystemSnapshot, source: SnapshotSource) -> Self {
        let display_dto = RuntimeSnapshotDto::from(&inner);
        Self {
            inner,
            display_dto,
            received_at: Utc::now(),
            source,
        }
    }

    /// Reference to the display DTO used by the UI (positions, orders, metrics, alerts, etc.).
    #[inline]
    pub fn dto(&self) -> &RuntimeSnapshotDto {
        &self.display_dto
    }

    #[inline]
    pub fn refresh_display_dto(&mut self) {
        self.display_dto = RuntimeSnapshotDto::from(&self.inner);
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
