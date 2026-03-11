//! Application state and event dispatch.

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::watch;

use crate::config::TuiConfig;
use crate::models::TuiSnapshot;

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
    pub should_quit: bool,
    snapshot_rx: watch::Receiver<Option<TuiSnapshot>>,
}

impl App {
    pub fn new(config: TuiConfig, snapshot_rx: watch::Receiver<Option<TuiSnapshot>>) -> Self {
        Self {
            config,
            active_tab: Tab::Dashboard,
            snapshot: None,
            should_quit: false,
            snapshot_rx,
        }
    }

    /// Pull latest snapshot from the NATS channel.
    pub fn tick(&mut self) {
        if self.snapshot_rx.has_changed().unwrap_or(false) {
            if let Some(snap) = self.snapshot_rx.borrow_and_update().clone() {
                self.snapshot = Some(snap);
            }
        }
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
            _ => {}
        }
    }
}
