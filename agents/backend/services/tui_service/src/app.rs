//! Application state and event dispatch.

use std::collections::{HashMap, VecDeque};

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::{mpsc, watch};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::config::TuiConfig;
use crate::events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget};
use crate::models::{SnapshotSource, TuiSnapshot};

const SPARKLINE_HISTORY_SIZE: usize = 20;

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
    /// Per-symbol ROI history for sparkline visualization (symbol -> deque of ROI values)
    pub roi_history: HashMap<String, VecDeque<f64>>,
    /// Order filter text (filters orders by symbol or status)
    pub order_filter: String,
    /// State for the tui-logger widget (scroll position, level filter).
    pub log_state: TuiWidgetState,
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
            roi_history: HashMap::new(),
            order_filter: String::new(),
            log_state: TuiWidgetState::default(),
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
            tracing::info!("Config reloaded from disk");
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
                    self.snapshot = Some(snap.clone());
                    self.update_roi_history(&snap);
                }
            }
        }
    }

    fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Connection { target, status } => match target {
                ConnectionTarget::Nats => self.nats_status = status,
            },
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

    fn update_roi_history(&mut self, snap: &TuiSnapshot) {
        for symbol_data in &snap.inner.symbols {
            let roi = symbol_data.roi;
            let entry = self
                .roi_history
                .entry(symbol_data.symbol.clone())
                .or_default();
            entry.push_back(roi);
            while entry.len() > SPARKLINE_HISTORY_SIZE {
                entry.pop_front();
            }
        }
        let current_symbols: std::collections::HashSet<_> = snap
            .inner
            .symbols
            .iter()
            .map(|s| s.symbol.clone())
            .collect();
        self.roi_history.retain(|k, _| current_symbols.contains(k));
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
            // Log tab navigation — forwarded to TuiWidgetState
            KeyCode::Up if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::UpKey);
            }
            KeyCode::Down if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::DownKey);
            }
            KeyCode::PageUp if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::PrevPageKey);
            }
            KeyCode::PageDown if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::NextPageKey);
            }
            // Log level filter (canonical tui-logger keys)
            KeyCode::Char('+') if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::PlusKey);
            }
            KeyCode::Char('-') if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::MinusKey);
            }
            KeyCode::Char('h') if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::HideKey);
            }
            KeyCode::Esc if self.active_tab == Tab::Logs => {
                self.log_state.transition(TuiWidgetEvent::EscapeKey);
            }
            // Order filter: '/' to activate, chars to add, Backspace to delete, Esc to clear
            KeyCode::Char('/') if self.active_tab == Tab::Orders => {
                self.order_filter.clear();
            }
            KeyCode::Char(c) => {
                if self.active_tab == Tab::Orders {
                    self.order_filter.push(c);
                }
            }
            KeyCode::Backspace if self.active_tab == Tab::Orders => {
                self.order_filter.pop();
            }
            KeyCode::Esc if self.active_tab == Tab::Orders => {
                self.order_filter.clear();
            }
            _ => {}
        }
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
        events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget},
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

    fn make_app() -> (
        App,
        watch::Sender<Option<TuiSnapshot>>,
        mpsc::UnboundedSender<AppEvent>,
    ) {
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
    fn app_updates_connection_status() {
        let (mut app, _, event_tx) = make_app();

        event_tx
            .send(AppEvent::Connection {
                target: ConnectionTarget::Nats,
                status: ConnectionStatus::new(ConnectionState::Retrying, "Connection refused"),
            })
            .expect("send connection status");

        app.tick();

        assert_eq!(app.nats_status.state, ConnectionState::Retrying);
        assert_eq!(app.nats_status.detail, "Connection refused");
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
    }

    #[test]
    fn log_tab_keys_do_not_panic() {
        let (mut app, _, _) = make_app();
        app.active_tab = Tab::Logs;

        // Verify scroll/filter keys are handled without panicking
        for key in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Char('+'),
            KeyCode::Char('-'),
            KeyCode::Esc,
        ] {
            app.handle_key(KeyEvent::from(key));
        }
    }
}
