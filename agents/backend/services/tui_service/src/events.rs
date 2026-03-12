use std::collections::VecDeque;

use chrono::{DateTime, Duration, Utc};

pub const MAX_LOG_ENTRIES: usize = 200;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
}

impl LogLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: Option<ConnectionTarget>,
    pub message: String,
    pub repeat_count: u32,
}

impl LogEntry {
    pub fn new(
        level: LogLevel,
        target: Option<ConnectionTarget>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            target,
            message: message.into(),
            repeat_count: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Connection {
        target: ConnectionTarget,
        status: ConnectionStatus,
    },
    Log(LogEntry),
}

pub fn push_log(logs: &mut VecDeque<LogEntry>, entry: LogEntry) {
    if let Some(latest) = logs.front_mut() {
        let same_entry = latest.level == entry.level
            && latest.target == entry.target
            && latest.message == entry.message;
        let within_dedupe_window = entry.timestamp - latest.timestamp <= Duration::seconds(5);

        if same_entry && within_dedupe_window {
            latest.timestamp = entry.timestamp;
            latest.repeat_count += 1;
            return;
        }
    }

    logs.push_front(entry);
    if logs.len() > MAX_LOG_ENTRIES {
        logs.truncate(MAX_LOG_ENTRIES);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use chrono::Duration;

    use super::{push_log, ConnectionTarget, LogEntry, LogLevel};

    #[test]
    fn push_log_dedupes_repeated_entries_within_window() {
        let mut logs = VecDeque::new();
        let first = LogEntry::new(
            LogLevel::Warn,
            Some(ConnectionTarget::Nats),
            "connect failed",
        );
        let mut second = LogEntry::new(
            LogLevel::Warn,
            Some(ConnectionTarget::Nats),
            "connect failed",
        );
        second.timestamp = first.timestamp + Duration::seconds(2);

        push_log(&mut logs, first);
        push_log(&mut logs, second);

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].repeat_count, 2);
    }

    #[test]
    fn push_log_keeps_distinct_entries() {
        let mut logs = VecDeque::new();

        push_log(
            &mut logs,
            LogEntry::new(
                LogLevel::Warn,
                Some(ConnectionTarget::Nats),
                "connect failed",
            ),
        );
        push_log(
            &mut logs,
            LogEntry::new(LogLevel::Info, Some(ConnectionTarget::Nats), "connected"),
        );

        assert_eq!(logs.len(), 2);
    }
}
