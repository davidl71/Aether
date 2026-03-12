//! Application state and event dispatch.

use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::{mpsc, watch};

use crate::config::TuiConfig;
use crate::events::{
    push_log, AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget, LogEntry, LogLevel,
};
use crate::models::{SnapshotSource, TuiSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Positions,
    Orders,
    Alerts,
    Logs,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Dashboard,
        Tab::Positions,
        Tab::Orders,
        Tab::Alerts,
        Tab::Logs,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dash",
            Tab::Positions => "Pos",
            Tab::Orders => "Orders",
            Tab::Alerts => "Alerts",
            Tab::Logs => "Logs",
        }
    }

    fn index(&self) -> usize {
        Tab::ALL.iter().position(|t| t == self).unwrap_or(0)
    }

    fn next(&self) -> Tab {
        let i = (self.index() + 1) % Tab::ALL.len();
        Tab::ALL[i].clone()
    }

    fn prev(&self) -> Tab {
        let i = (self.index() + Tab::ALL.len() - 1) % Tab::ALL.len();
        Tab::ALL[i].clone()
    }
}

pub struct App {
    pub config: TuiConfig,
    pub active_tab: Tab,
    pub snapshot: Option<TuiSnapshot>,
    pub logs: VecDeque<LogEntry>,
    pub log_scroll: u16,
    pub nats_status: ConnectionStatus,
    pub should_quit: bool,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
    config_rx: watch::Receiver<TuiConfig>,
}

impl App {
    pub fn new(
        config: TuiConfig,
        snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
        event_rx: mpsc::UnboundedReceiver<AppEvent>,
        config_rx: watch::Receiver<TuiConfig>,
    ) -> Self {
        Self {
            config,
            active_tab: Tab::Dashboard,
            snapshot: None,
            logs: VecDeque::new(),
            log_scroll: 0,
            nats_status: ConnectionStatus::new(ConnectionState::Starting, "Connecting to NATS"),
            should_quit: false,
            event_rx,
            snapshot_rx,
            config_rx,
        }
    }

    /// Pull latest snapshot and config updates, process queued events.
    pub fn tick(&mut self) {
        // Apply hot-reloaded config if it changed
        if self.config_rx.has_changed().unwrap_or(false) {
            let new_config = self.config_rx.borrow_and_update().clone();
            self.config = new_config;
            push_log(
                &mut self.logs,
                LogEntry::new(LogLevel::Info, None, "Config reloaded from disk"),
            );
        }

        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(event);
        }

        if self.snapshot_rx.has_changed().unwrap_or(false) {
            let next_snapshot = {
                let borrowed = self.snapshot_rx.borrow_and_update();
                borrowed.clone()
            };

            if let Some(snap) = next_snapshot {
                if self.should_accept_snapshot(&snap) {
                    self.snapshot = Some(snap);
                }
            }
        }
    }

    fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Connection { target, status } => match target {
                ConnectionTarget::Nats => self.nats_status = status,
            },
            AppEvent::Log(entry) => push_log(&mut self.logs, entry),
        }
    }

    fn should_accept_snapshot(&self, incoming: &TuiSnapshot) -> bool {
        let Some(current) = self.snapshot.as_ref() else {
            return true;
        };

        if incoming.source == SnapshotSource::Nats {
            return true;
        }

        if current.source != SnapshotSource::Nats {
            return true;
        }

        current.is_stale(self.config.snapshot_stale_after_secs())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
            KeyCode::Tab | KeyCode::Right => self.active_tab = self.active_tab.next(),
            KeyCode::BackTab | KeyCode::Left => self.active_tab = self.active_tab.prev(),
            KeyCode::Char('1') => self.active_tab = Tab::Dashboard,
            KeyCode::Char('2') => self.active_tab = Tab::Positions,
            KeyCode::Char('3') => self.active_tab = Tab::Orders,
            KeyCode::Char('4') => self.active_tab = Tab::Alerts,
            KeyCode::Char('5') => self.active_tab = Tab::Logs,
            KeyCode::Up if self.active_tab == Tab::Logs => self.scroll_logs_up(1),
            KeyCode::Down if self.active_tab == Tab::Logs => self.scroll_logs_down(1),
            KeyCode::PageUp if self.active_tab == Tab::Logs => self.scroll_logs_up(10),
            KeyCode::PageDown if self.active_tab == Tab::Logs => self.scroll_logs_down(10),
            KeyCode::Home if self.active_tab == Tab::Logs => self.log_scroll = 0,
            KeyCode::End if self.active_tab == Tab::Logs => self.log_scroll = self.max_log_scroll(),
            _ => {}
        }
    }

    fn scroll_logs_up(&mut self, amount: u16) {
        self.log_scroll = self.log_scroll.saturating_sub(amount);
    }

    fn scroll_logs_down(&mut self, amount: u16) {
        self.log_scroll = self
            .log_scroll
            .saturating_add(amount)
            .min(self.max_log_scroll());
    }

    fn max_log_scroll(&self) -> u16 {
        let base_lines = 4_usize;
        let content_lines = base_lines + self.logs.len();
        content_lines.saturating_sub(1).min(u16::MAX as usize) as u16
    }
}

#[cfg(test)]
mod tests {
    use api::{Metrics, RiskStatus, RuntimeSnapshotDto};
    use chrono::{Duration, Utc};
    use crossterm::event::{KeyCode, KeyEvent};
    use tokio::sync::{mpsc, watch};

    use super::{App, Tab};
    use crate::{
        config::TuiConfig,
        events::{
            AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget, LogEntry, LogLevel,
        },
        models::{SnapshotSource, TuiSnapshot},
    };

    fn snapshot(source: SnapshotSource) -> TuiSnapshot {
        TuiSnapshot {
            inner: RuntimeSnapshotDto {
                generated_at: Utc::now(),
                started_at: Utc::now(),
                mode: "paper".into(),
                strategy: "box".into(),
                account_id: "DU123".into(),
                metrics: Metrics::default(),
                symbols: Vec::new(),
                positions: Vec::new(),
                historic: Vec::new(),
                orders: Vec::new(),
                decisions: Vec::new(),
                alerts: Vec::new(),
                risk: RiskStatus::default(),
            },
            received_at: Utc::now(),
            source,
        }
    }

    fn make_app() -> (App, watch::Sender<Option<TuiSnapshot>>, mpsc::UnboundedSender<AppEvent>) {
        let (snap_tx, snap_rx) = watch::channel(None);
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (_config_tx, config_rx) = watch::channel(TuiConfig::default());
        let app = App::new(TuiConfig::default(), snap_rx, event_rx, config_rx);
        (app, snap_tx, event_tx)
    }

    #[test]
    fn rest_snapshot_does_not_replace_fresh_nats_snapshot() {
        let (mut app, tx, _) = make_app();

        app.snapshot = Some(snapshot(SnapshotSource::Nats));
        tx.send(Some(snapshot(SnapshotSource::Rest)))
            .expect("send rest snapshot");
        app.tick();

        assert_eq!(
            app.snapshot.as_ref().map(|snap| &snap.source),
            Some(&SnapshotSource::Nats)
        );
    }

    #[test]
    fn rest_snapshot_replaces_stale_nats_snapshot() {
        let (mut app, tx, _) = make_app();

        let mut stale_nats = snapshot(SnapshotSource::Nats);
        stale_nats.received_at = Utc::now() - Duration::seconds(5);
        app.snapshot = Some(stale_nats);

        tx.send(Some(snapshot(SnapshotSource::Rest)))
            .expect("send rest snapshot");
        app.tick();

        assert_eq!(
            app.snapshot.as_ref().map(|snap| &snap.source),
            Some(&SnapshotSource::Rest)
        );
    }

    #[test]
    fn app_collects_logs_and_connection_updates() {
        let (mut app, _, event_tx) = make_app();

        event_tx
            .send(AppEvent::Connection {
                target: ConnectionTarget::Nats,
                status: ConnectionStatus::new(ConnectionState::Retrying, "Connection refused"),
            })
            .expect("send connection status");
        event_tx
            .send(AppEvent::Log(LogEntry::new(
                LogLevel::Warn,
                Some(ConnectionTarget::Nats),
                "NATS connect failed",
            )))
            .expect("send log event");

        app.tick();

        assert_eq!(app.nats_status.state, ConnectionState::Retrying);
        assert_eq!(app.nats_status.detail, "Connection refused");
        assert_eq!(app.logs.len(), 1);
        assert_eq!(app.logs[0].message, "NATS connect failed");
    }

    #[test]
    fn config_hot_reload_updates_app_config() {
        let (snap_tx, snap_rx) = watch::channel(None);
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let (config_tx, config_rx) = watch::channel(TuiConfig::default());
        let mut app = App::new(TuiConfig::default(), snap_rx, event_rx, config_rx);
        drop(snap_tx);

        let mut new_config = TuiConfig::default();
        new_config.watchlist = vec!["TSLA".into()];
        config_tx.send(new_config).expect("send new config");

        app.tick();

        assert_eq!(app.config.watchlist, vec!["TSLA"]);
        assert_eq!(app.logs.len(), 1);
        assert!(app.logs[0].message.contains("Config reloaded"));
    }

    #[test]
    fn logs_tab_scrolls_and_clamps() {
        let (mut app, _, event_tx) = make_app();
        app.active_tab = Tab::Logs;

        for idx in 0..20 {
            event_tx
                .send(AppEvent::Log(LogEntry::new(
                    LogLevel::Warn,
                    Some(ConnectionTarget::Nats),
                    format!("entry {idx}"),
                )))
                .expect("send log event");
        }
        app.tick();

        app.handle_key(KeyEvent::from(KeyCode::PageDown));
        assert!(app.log_scroll > 0);

        app.handle_key(KeyEvent::from(KeyCode::End));
        let end_scroll = app.log_scroll;
        app.handle_key(KeyEvent::from(KeyCode::PageDown));
        assert_eq!(app.log_scroll, end_scroll);

        app.handle_key(KeyEvent::from(KeyCode::Home));
        assert_eq!(app.log_scroll, 0);
    }
}
