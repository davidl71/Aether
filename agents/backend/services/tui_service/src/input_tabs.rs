//! Per-tab key maps (Yield/Charts/Settings/… → [`Action`]).
//!
//! Keeps tab-specific bindings out of `input.rs` so the top-level router can stay simple.

use crossterm::event::KeyCode;

use crate::app::{App, InputMode, Tab};
use crate::input::Action;
use crate::input_settings::settings_key_action;

pub(crate) fn tab_key_action(app: &App, key: KeyCode, input_mode: InputMode) -> Option<Action> {
    if matches!(input_mode, InputMode::TreePanel) {
        return tree_panel_key_action(key);
    }

    match app.active_tab {
        Tab::Yield => yield_key_action(key),
        Tab::Charts => charts_key_action(key, input_mode),
        Tab::Settings => settings_key_action(app, key),
        Tab::Positions => positions_key_action(key),
        Tab::Orders => orders_key_action(key, input_mode),
        Tab::Loans => loans_key_action(key, input_mode),
        Tab::DiscountBank => discount_bank_key_action(key),
        Tab::Ledger => ledger_key_action(key),
        Tab::Alerts => alerts_key_action(key),
        Tab::Dashboard => dashboard_key_action(key),
        Tab::Scenarios => scenarios_key_action(key),
        Tab::Logs => logs_key_action(key),
    }
}

fn tree_panel_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Up => Some(Action::TreeUp),
        KeyCode::Down => Some(Action::TreeDown),
        KeyCode::Left => Some(Action::TreeLeft),
        KeyCode::Right => Some(Action::TreeRight),
        KeyCode::Enter => Some(Action::TreeToggle),
        KeyCode::Esc => Some(Action::TreeEscape),
        _ => None,
    }
}

fn yield_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Left => Some(Action::YieldSymbolPrev),
        KeyCode::Right => Some(Action::YieldSymbolNext),
        KeyCode::Up => Some(Action::YieldCurveScrollUp),
        KeyCode::Down => Some(Action::YieldCurveScrollDown),
        KeyCode::Enter => Some(Action::YieldCurveDetail),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::YieldRefresh),
        _ => None,
    }
}

fn charts_key_action(key: KeyCode, input_mode: InputMode) -> Option<Action> {
    match (key, input_mode) {
        (KeyCode::Char('/'), _) => Some(Action::ChartSearchFocus),
        (KeyCode::Left, InputMode::ChartSearch) => None,
        (KeyCode::Right, InputMode::ChartSearch) => None,
        (KeyCode::Home, InputMode::ChartSearch) => Some(Action::ChartSearchFirst),
        (KeyCode::End, InputMode::ChartSearch) => Some(Action::ChartSearchLast),
        (KeyCode::Left, _) => Some(Action::ChartPillLeft),
        (KeyCode::Right, _) => Some(Action::ChartPillRight),
        (KeyCode::Up, InputMode::ChartSearch) => Some(Action::ChartSearchUp),
        (KeyCode::Down, InputMode::ChartSearch) => Some(Action::ChartSearchDown),
        (KeyCode::Enter, InputMode::ChartSearch) => Some(Action::ChartSearchEnter),
        (KeyCode::Esc, InputMode::ChartSearch) => Some(Action::ChartSearchEscape),
        (KeyCode::Backspace, InputMode::ChartSearch) => Some(Action::ChartSearchBackspace),
        (KeyCode::Char(c), InputMode::ChartSearch) if !c.is_control() => {
            Some(Action::ChartSearchChar(c))
        }
        (KeyCode::Char('h') | KeyCode::Char('H'), _) => Some(Action::ChartPillLeft),
        (KeyCode::Char('l') | KeyCode::Char('L'), _) => Some(Action::ChartPillRight),
        (KeyCode::Char('j') | KeyCode::Char('J'), _) => Some(Action::ChartPillDown),
        (KeyCode::Char('k') | KeyCode::Char('K'), _) => Some(Action::ChartPillUp),
        (KeyCode::Up, _) => Some(Action::ChartPillUp),
        (KeyCode::Down, _) => Some(Action::ChartPillDown),
        (KeyCode::Enter, _) => Some(Action::ChartPillSelect),
        _ => None,
    }
}

fn positions_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char(' ') => {
            Some(Action::PositionsToggleCombo)
        }
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::PositionsCycleSort),
        KeyCode::Up => Some(Action::PositionsScrollUp),
        KeyCode::Down => Some(Action::PositionsScrollDown),
        KeyCode::PageUp => Some(Action::PositionsScrollPageUp),
        KeyCode::PageDown => Some(Action::PositionsScrollPageDown),
        KeyCode::Enter => Some(Action::PositionsDetail),
        _ => None,
    }
}

fn orders_key_action(key: KeyCode, input_mode: InputMode) -> Option<Action> {
    match (key, input_mode) {
        (KeyCode::Up, _) => Some(Action::OrdersScrollUp),
        (KeyCode::Down, _) => Some(Action::OrdersScrollDown),
        (KeyCode::PageUp, _) => Some(Action::OrdersScrollPageUp),
        (KeyCode::PageDown, _) => Some(Action::OrdersScrollPageDown),
        (KeyCode::Enter, _) => Some(Action::OrdersDetail),
        (KeyCode::Char('x') | KeyCode::Char('X'), _) => Some(Action::OrdersCancel),
        (KeyCode::Char('/'), _) => Some(Action::OrdersFilterFocus),
        (KeyCode::Esc, InputMode::OrdersFilter) => Some(Action::OrdersFilterClear),
        (KeyCode::Backspace, InputMode::OrdersFilter) => Some(Action::OrdersFilterBackspace),
        (KeyCode::Char(c), InputMode::OrdersFilter) if !c.is_control() => {
            Some(Action::OrdersFilterChar(c))
        }
        _ => None,
    }
}

fn loans_key_action(key: KeyCode, input_mode: InputMode) -> Option<Action> {
    match (key, input_mode) {
        (KeyCode::Esc, InputMode::LoanImportPath) => Some(Action::LoansImportPathEscape),
        (KeyCode::Enter, InputMode::LoanImportPath) => Some(Action::LoansImportPathEnter),
        (KeyCode::Backspace, InputMode::LoanImportPath) => Some(Action::LoansImportPathBackspace),
        (KeyCode::Char(c), InputMode::LoanImportPath) if !c.is_control() => {
            Some(Action::LoansImportPathChar(c))
        }
        (_, InputMode::LoanImportPath) => Some(Action::NoOp),
        (KeyCode::Up, _) => Some(Action::LoansScrollUp),
        (KeyCode::Down, _) => Some(Action::LoansScrollDown),
        (KeyCode::PageUp, _) => Some(Action::LoansScrollPageUp),
        (KeyCode::PageDown, _) => Some(Action::LoansScrollPageDown),
        (KeyCode::Char('n') | KeyCode::Char('N'), _) => Some(Action::LoansNewLoan),
        (KeyCode::Char('b') | KeyCode::Char('B'), _) => Some(Action::LoansBulkImportFocus),
        (KeyCode::Char('i') | KeyCode::Char('I'), _) => Some(Action::LoansBulkImportFocus),
        _ => None,
    }
}

fn discount_bank_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Up => Some(Action::DiscountBankScrollUp),
        KeyCode::Down => Some(Action::DiscountBankScrollDown),
        KeyCode::PageUp => Some(Action::DiscountBankScrollPageUp),
        KeyCode::PageDown => Some(Action::DiscountBankScrollPageDown),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::DiscountBankRefresh),
        _ => None,
    }
}

fn ledger_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Up => Some(Action::LedgerScrollUp),
        KeyCode::Down => Some(Action::LedgerScrollDown),
        KeyCode::PageUp => Some(Action::LedgerScrollPageUp),
        KeyCode::PageDown => Some(Action::LedgerScrollPageDown),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::LedgerRefresh),
        _ => None,
    }
}

fn alerts_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Up => Some(Action::AlertsScrollUp),
        KeyCode::Down => Some(Action::AlertsScrollDown),
        KeyCode::PageUp => Some(Action::AlertsScrollPageUp),
        KeyCode::PageDown => Some(Action::AlertsScrollPageDown),
        _ => None,
    }
}

fn dashboard_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Up => Some(Action::DashboardScrollUp),
        KeyCode::Down => Some(Action::DashboardScrollDown),
        KeyCode::Enter => Some(Action::DashboardNavigateToChart),
        _ => None,
    }
}

fn scenarios_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Up => Some(Action::ScenariosScrollUp),
        KeyCode::Down => Some(Action::ScenariosScrollDown),
        KeyCode::PageUp => Some(Action::ScenariosScrollPageUp),
        KeyCode::PageDown => Some(Action::ScenariosScrollPageDown),
        KeyCode::Enter => Some(Action::ScenariosDetail),
        KeyCode::Char('o') | KeyCode::Char('O') => Some(Action::ScenariosExecute),
        KeyCode::Char('[') => Some(Action::ScenariosDteContract),
        KeyCode::Char(']') => Some(Action::ScenariosDteExpand),
        KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::ScenariosCycleStrikeWidth),
        _ => None,
    }
}

fn logs_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Up => Some(Action::LogScrollUp),
        KeyCode::Down => Some(Action::LogScrollDown),
        KeyCode::PageUp => Some(Action::LogPageUp),
        KeyCode::PageDown => Some(Action::LogPageDown),
        KeyCode::Char('+') => Some(Action::LogLevelUp),
        KeyCode::Char('-') => Some(Action::LogLevelDown),
        KeyCode::Char('h') | KeyCode::Char('H') => Some(Action::LogHide),
        KeyCode::Esc => Some(Action::LogEscape),
        KeyCode::Char('e') | KeyCode::Char('E') => Some(Action::LogLevelError),
        KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::LogLevelWarn),
        KeyCode::Char('i') | KeyCode::Char('I') => Some(Action::LogLevelInfo),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(Action::LogLevelDebug),
        _ => None,
    }
}
