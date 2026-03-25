use crate::app::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneHintMode {
    None,
    Yield,
    Settings,
    Charts,
    Orders,
    Scenarios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneSpec {
    pub tab: Tab,
    pub label: &'static str,
    pub title: &'static str,
    pub hint_mode: PaneHintMode,
}

pub fn pane_spec(tab: Tab) -> PaneSpec {
    match tab {
        Tab::Dashboard => PaneSpec {
            tab,
            label: "Dash",
            title: "Dashboard",
            hint_mode: PaneHintMode::None,
        },
        Tab::Positions => PaneSpec {
            tab,
            label: "Pos",
            title: "Positions",
            hint_mode: PaneHintMode::None,
        },
        Tab::Charts => PaneSpec {
            tab,
            label: "Charts",
            title: "Charts",
            hint_mode: PaneHintMode::Charts,
        },
        Tab::Orders => PaneSpec {
            tab,
            label: "Orders",
            title: "Orders",
            hint_mode: PaneHintMode::Orders,
        },
        Tab::Alerts => PaneSpec {
            tab,
            label: "Alerts",
            title: "Alerts",
            hint_mode: PaneHintMode::None,
        },
        Tab::Yield => PaneSpec {
            tab,
            label: "Yield",
            title: "Yield",
            hint_mode: PaneHintMode::Yield,
        },
        Tab::Loans => PaneSpec {
            tab,
            label: "Loans",
            title: "Loans",
            hint_mode: PaneHintMode::None,
        },
        Tab::DiscountBank => PaneSpec {
            tab,
            label: "Bank",
            title: "Bank",
            hint_mode: PaneHintMode::None,
        },
        Tab::Scenarios => PaneSpec {
            tab,
            label: "Scen",
            title: "Scenarios",
            hint_mode: PaneHintMode::Scenarios,
        },
        Tab::Logs => PaneSpec {
            tab,
            label: "Logs",
            title: "Logs",
            hint_mode: PaneHintMode::None,
        },
        Tab::Settings => PaneSpec {
            tab,
            label: "Set",
            title: "Settings",
            hint_mode: PaneHintMode::Settings,
        },
    }
}
