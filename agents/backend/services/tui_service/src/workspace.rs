use crate::app::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibleWorkspace {
    None,
    SplitPane,
    Market,
    Operations,
    Credit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceSpec {
    pub kind: VisibleWorkspace,
    pub title: &'static str,
    pub summary: &'static str,
    pub tabs: &'static [Tab],
    pub min_width: u16,
    pub min_height: u16,
    pub hint_label: &'static str,
}

impl WorkspaceSpec {
    pub fn focus_target(self, active_tab: Tab, forward: bool) -> Option<Tab> {
        let tabs = self.tabs;
        let index = tabs.iter().position(|tab| *tab == active_tab)?;
        let next = if forward {
            (index + 1) % tabs.len()
        } else {
            (index + tabs.len() - 1) % tabs.len()
        };
        tabs.get(next).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Health,
    Config,
    Symbols,
    Sources,
    /// Alpaca API key ID + secret (paper / live) via keyring or credential file.
    Alpaca,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsHealthFocus {
    Transport,
    Services,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryFocus {
    None,
    Settings(SettingsSection),
    SettingsHealth(SettingsHealthFocus),
}

impl SecondaryFocus {
    pub fn label(self) -> Option<&'static str> {
        match self {
            SecondaryFocus::None => None,
            SecondaryFocus::Settings(section) => Some(section.title()),
            SecondaryFocus::SettingsHealth(focus) => Some(focus.title()),
        }
    }

    pub fn title(self) -> Option<String> {
        match self {
            SecondaryFocus::None => None,
            SecondaryFocus::Settings(section) => Some(format!("Settings / {}", section.title())),
            SecondaryFocus::SettingsHealth(focus) => {
                Some(format!("Settings / Health / {}", focus.title()))
            }
        }
    }
}

impl SettingsHealthFocus {
    pub fn title(self) -> &'static str {
        match self {
            SettingsHealthFocus::Transport => "Transport",
            SettingsHealthFocus::Services => "Services",
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SettingsHealthFocus::Transport => SettingsHealthFocus::Transport,
            SettingsHealthFocus::Services => SettingsHealthFocus::Transport,
        }
    }
}

impl SettingsSection {
    pub fn title(self) -> &'static str {
        match self {
            SettingsSection::Health => "Health",
            SettingsSection::Config => "Config",
            SettingsSection::Symbols => "Symbols",
            SettingsSection::Sources => "Sources",
            SettingsSection::Alpaca => "Alpaca",
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SettingsSection::Health => SettingsSection::Alpaca,
            SettingsSection::Config => SettingsSection::Health,
            SettingsSection::Symbols => SettingsSection::Config,
            SettingsSection::Sources => SettingsSection::Symbols,
            SettingsSection::Alpaca => SettingsSection::Sources,
        }
    }

    pub fn next(self) -> Self {
        match self {
            SettingsSection::Health => SettingsSection::Config,
            SettingsSection::Config => SettingsSection::Symbols,
            SettingsSection::Symbols => SettingsSection::Sources,
            SettingsSection::Sources => SettingsSection::Alpaca,
            SettingsSection::Alpaca => SettingsSection::Health,
        }
    }
}

pub const MARKET_WORKSPACE_TABS: [Tab; 4] =
    [Tab::Dashboard, Tab::Positions, Tab::Orders, Tab::Yield];
pub const OPERATIONS_WORKSPACE_TABS: [Tab; 3] = [Tab::Alerts, Tab::Logs, Tab::Settings];
pub const CREDIT_WORKSPACE_TABS: [Tab; 2] = [Tab::Loans, Tab::DiscountBank];
pub const SPLIT_PANE_TABS: [Tab; 2] = [Tab::Dashboard, Tab::Positions];

impl VisibleWorkspace {
    pub fn spec(self) -> Option<WorkspaceSpec> {
        match self {
            VisibleWorkspace::None => None,
            VisibleWorkspace::SplitPane => Some(WorkspaceSpec {
                kind: self,
                title: "Split pane",
                summary: "Dashboard + Positions",
                tabs: &SPLIT_PANE_TABS,
                min_width: 0,
                min_height: 0,
                hint_label: "split",
            }),
            VisibleWorkspace::Market => Some(WorkspaceSpec {
                kind: self,
                title: "Market Workspace",
                summary: "Dashboard · positions · orders · yield",
                tabs: &MARKET_WORKSPACE_TABS,
                min_width: 170,
                min_height: 22,
                hint_label: "workspace",
            }),
            VisibleWorkspace::Operations => Some(WorkspaceSpec {
                kind: self,
                title: "Operations Workspace",
                summary: "Alerts · logs · embedded settings",
                tabs: &OPERATIONS_WORKSPACE_TABS,
                min_width: 170,
                min_height: 20,
                hint_label: "ops",
            }),
            VisibleWorkspace::Credit => Some(WorkspaceSpec {
                kind: self,
                title: "Credit Workspace",
                summary: "Loans + Bank visible",
                tabs: &CREDIT_WORKSPACE_TABS,
                min_width: 170,
                min_height: 18,
                hint_label: "credit",
            }),
        }
    }
}
