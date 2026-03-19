//! Structured event routing for the TUI.
//!
//! Uses tokio::sync::broadcast for multi-listener event distribution.
//! Events are categorized by priority and type for flexible handling.
//!
//! NOTE: EventRouter infrastructure is built but not yet wired to app.rs.
//! publish_connection/snapshot/key/tab_change are called nowhere.
//! TODO(T-1773357423959019000): wire EventRouter into app state and remove allow(dead_code).

#![allow(dead_code)]

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Event priority levels for routing decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl EventPriority {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Normal => "NORM",
            Self::High => "HIGH",
            Self::Critical => "CRIT",
        }
    }
}

/// Event category for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    Connection,
    Snapshot,
    UserAction,
    Alert,
    Order,
}

impl EventCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Connection => "conn",
            Self::Snapshot => "snap",
            Self::UserAction => "user",
            Self::Alert => "alert",
            Self::Order => "order",
        }
    }
}

/// Base event envelope with metadata
#[derive(Debug, Clone)]
pub struct Event {
    pub id: String,
    pub priority: EventPriority,
    pub category: EventCategory,
    pub timestamp: DateTime<Utc>,
    pub payload: EventPayload,
}

impl Event {
    pub fn new(priority: EventPriority, category: EventCategory, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            priority,
            category,
            timestamp: Utc::now(),
            payload,
        }
    }
}

/// Event payloads by category
#[derive(Debug, Clone)]
pub enum EventPayload {
    ConnectionChanged(ConnectionStatus),
    SnapshotReceived {
        source: SnapshotSource,
        age_secs: i64,
    },
    KeyPressed {
        key: String,
        tab: String,
    },
    TabChanged(String),
    AlertReceived {
        level: String,
        message: String,
    },
    OrderUpdated {
        order_id: String,
        status: String,
    },
}

// ============================================================================
// Connection events
// ============================================================================

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

// ============================================================================
// Snapshot source
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotSource {
    Nats,
}

impl SnapshotSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Nats => "NATS",
        }
    }
}

// ============================================================================
// Event router using broadcast channel
// ============================================================================

pub struct EventRouter {
    sender: broadcast::Sender<Event>,
}

impl EventRouter {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub fn publish_connection(&self, status: ConnectionStatus) {
        let payload = EventPayload::ConnectionChanged(status);
        let event = Event::new(EventPriority::High, EventCategory::Connection, payload);
        self.publish(event);
    }

    pub fn publish_snapshot(&self, source: SnapshotSource, age_secs: i64) {
        let payload = EventPayload::SnapshotReceived { source, age_secs };
        let event = Event::new(EventPriority::Normal, EventCategory::Snapshot, payload);
        self.publish(event);
    }

    pub fn publish_key(&self, key: String, tab: String) {
        let payload = EventPayload::KeyPressed { key, tab };
        let event = Event::new(EventPriority::Normal, EventCategory::UserAction, payload);
        self.publish(event);
    }

    pub fn publish_tab_change(&self, tab: String) {
        let payload = EventPayload::TabChanged(tab);
        let event = Event::new(EventPriority::Normal, EventCategory::UserAction, payload);
        self.publish(event);
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper for EventRouter
pub type SharedEventRouter = Arc<EventRouter>;

pub fn create_event_router() -> SharedEventRouter {
    Arc::new(EventRouter::new())
}

// ============================================================================
// Strategy command (used by main.rs and app.rs)
// ============================================================================

#[derive(Debug, Clone)]
pub enum StrategyCommand {
    Start,
    Stop,
    CancelAll,
    PublishSnapshot,
    SetMode(String),
}

// ============================================================================
// Legacy AppEvent for backward compatibility
// ============================================================================

/// Legacy event type for backward compatibility (use Event instead)
#[derive(Debug, Clone)]
pub enum AppEvent {
    Connection {
        target: ConnectionTarget,
        status: ConnectionStatus,
    },
}
