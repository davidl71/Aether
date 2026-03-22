//! Input handling: key events to application actions.
//!
//! Converts crossterm key events into typed actions, keeping input parsing
//! separate from application state mutation.

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, DetailPopupContent, Tab};
use crate::events::StrategyCommand;

/// Actions that can result from key events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Quit,
    ShowHelp,
    ToggleLogPanel,
    TabNext,
    TabPrev,
    JumpToTab(u8),
    YieldSymbolPrev,
    YieldSymbolNext,
    PositionsScrollUp,
    PositionsScrollDown,
    PositionsScrollPageUp,
    PositionsScrollPageDown,
    PositionsToggleCombo,
    PositionsDetail,
    OrdersScrollUp,
    OrdersScrollDown,
    OrdersScrollPageUp,
    OrdersScrollPageDown,
    OrdersDetail,
    OrdersFilterFocus,
    OrdersFilterChar(char),
    OrdersFilterBackspace,
    OrdersFilterClear,
    LoansScrollUp,
    LoansScrollDown,
    LoansScrollPageUp,
    LoansScrollPageDown,
    AlertsScrollUp,
    AlertsScrollDown,
    AlertsScrollPageUp,
    AlertsScrollPageDown,
    ScenariosScrollUp,
    ScenariosScrollDown,
    ScenariosScrollPageUp,
    ScenariosScrollPageDown,
    ScenariosDetail,
    ScenariosDteContract,
    ScenariosDteExpand,
    ScenariosCycleStrikeWidth,
    ScenariosExecute,
    SettingsScrollUp,
    SettingsScrollDown,
    SettingsAddSymbol,
    SettingsEditConfig,
    SettingsDelete,
    SettingsReset,
    LogScrollUp,
    LogScrollDown,
    LogPageUp,
    LogPageDown,
    LogLevelUp,
    LogLevelDown,
    LogHide,
    LogEscape,
    LogLevelError,
    LogLevelWarn,
    LogLevelInfo,
    LogLevelDebug,
    ModeCycle,
    StrategyStart,
    StrategyStop,
    StrategyCancelAll,
    OrdersCancel,
    ForceSnapshot,
    SplitPaneToggle,
    NoOp,
}

/// Converts a key event to an action, or None if the key is not handled.
pub fn key_to_action(app: &App, key: KeyEvent) -> Option<Action> {
    // Only handle Press events (crossterm 0.27+)
    if key.kind != crossterm::event::KeyEventKind::Press {
        return None;
    }

    // Global actions that work even with overlays
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Some(Action::Quit),
        KeyCode::Char('?') => return Some(Action::ShowHelp),
        KeyCode::Char('`') | KeyCode::Char('~') => return Some(Action::ToggleLogPanel),
        KeyCode::Esc => return Some(Action::ToggleLogPanel),
        _ => {}
    }

    // Skip other input when help or detail popup is open
    if app.show_help || app.detail_popup.is_some() {
        return Some(Action::NoOp);
    }

    // Settings input mode (add symbol / edit config) - handled separately
    if app.settings_add_symbol_input.is_some() {
        return Some(Action::NoOp); // Input mode handles its own keys
    }

    match key.code {
        // Yield tab symbol navigation (before generic tab switch)
        KeyCode::Left if app.active_tab == Tab::Yield => Some(Action::YieldSymbolPrev),
        KeyCode::Right if app.active_tab == Tab::Yield => Some(Action::YieldSymbolNext),

        // Tab navigation
        KeyCode::Tab | KeyCode::Right => Some(Action::TabNext),
        KeyCode::BackTab | KeyCode::Left => Some(Action::TabPrev),
        KeyCode::Char('1') => Some(Action::JumpToTab(1)),
        KeyCode::Char('2') => Some(Action::JumpToTab(2)),
        KeyCode::Char('3') => Some(Action::JumpToTab(3)),
        KeyCode::Char('4') => Some(Action::JumpToTab(4)),
        KeyCode::Char('5') => Some(Action::JumpToTab(5)),
        KeyCode::Char('6') => Some(Action::JumpToTab(6)),
        KeyCode::Char('7') => Some(Action::JumpToTab(7)),
        KeyCode::Char('8') => Some(Action::JumpToTab(8)),
        KeyCode::Char('9') => Some(Action::JumpToTab(9)),
        KeyCode::Char('0') => Some(Action::JumpToTab(0)),

        // Positions
        KeyCode::Char('c') | KeyCode::Char('C')
            if app.active_tab == Tab::Positions || app.split_pane =>
        {
            Some(Action::PositionsToggleCombo)
        }
        KeyCode::Up if app.active_tab == Tab::Positions || app.split_pane => {
            Some(Action::PositionsScrollUp)
        }
        KeyCode::Down if app.active_tab == Tab::Positions || app.split_pane => {
            Some(Action::PositionsScrollDown)
        }
        KeyCode::PageUp if app.active_tab == Tab::Positions || app.split_pane => {
            Some(Action::PositionsScrollPageUp)
        }
        KeyCode::PageDown if app.active_tab == Tab::Positions || app.split_pane => {
            Some(Action::PositionsScrollPageDown)
        }
        KeyCode::Enter if app.active_tab == Tab::Positions || app.split_pane => {
            Some(Action::PositionsDetail)
        }

        // Orders
        KeyCode::Up if app.active_tab == Tab::Orders => Some(Action::OrdersScrollUp),
        KeyCode::Down if app.active_tab == Tab::Orders => Some(Action::OrdersScrollDown),
        KeyCode::PageUp if app.active_tab == Tab::Orders => Some(Action::OrdersScrollPageUp),
        KeyCode::PageDown if app.active_tab == Tab::Orders => Some(Action::OrdersScrollPageDown),
        KeyCode::Enter if app.active_tab == Tab::Orders => Some(Action::OrdersDetail),
        KeyCode::Char('x') | KeyCode::Char('X') if app.active_tab == Tab::Orders => {
            Some(Action::OrdersCancel)
        }
        KeyCode::Char('/') if app.active_tab == Tab::Orders => Some(Action::OrdersFilterFocus),
        KeyCode::Esc if app.active_tab == Tab::Orders => Some(Action::OrdersFilterClear),
        KeyCode::Char(c) if app.active_tab == Tab::Orders && !c.is_control() => {
            Some(Action::OrdersFilterChar(c))
        }
        KeyCode::Backspace if app.active_tab == Tab::Orders => Some(Action::OrdersFilterBackspace),

        // Loans
        KeyCode::Up if app.active_tab == Tab::Loans => Some(Action::LoansScrollUp),
        KeyCode::Down if app.active_tab == Tab::Loans => Some(Action::LoansScrollDown),
        KeyCode::PageUp if app.active_tab == Tab::Loans => Some(Action::LoansScrollPageUp),
        KeyCode::PageDown if app.active_tab == Tab::Loans => Some(Action::LoansScrollPageDown),

        // Alerts
        KeyCode::Up if app.active_tab == Tab::Alerts => Some(Action::AlertsScrollUp),
        KeyCode::Down if app.active_tab == Tab::Alerts => Some(Action::AlertsScrollDown),
        KeyCode::PageUp if app.active_tab == Tab::Alerts => Some(Action::AlertsScrollPageUp),
        KeyCode::PageDown if app.active_tab == Tab::Alerts => Some(Action::AlertsScrollPageDown),

        // Scenarios
        KeyCode::Up if app.active_tab == Tab::Scenarios => Some(Action::ScenariosScrollUp),
        KeyCode::Down if app.active_tab == Tab::Scenarios => Some(Action::ScenariosScrollDown),
        KeyCode::PageUp if app.active_tab == Tab::Scenarios => Some(Action::ScenariosScrollPageUp),
        KeyCode::PageDown if app.active_tab == Tab::Scenarios => {
            Some(Action::ScenariosScrollPageDown)
        }
        KeyCode::Enter if app.active_tab == Tab::Scenarios => Some(Action::ScenariosDetail),
        KeyCode::Char('o') | KeyCode::Char('O') if app.active_tab == Tab::Scenarios => {
            Some(Action::ScenariosExecute)
        }
        KeyCode::Char('[') if app.active_tab == Tab::Scenarios => {
            Some(Action::ScenariosDteContract)
        }
        KeyCode::Char(']') if app.active_tab == Tab::Scenarios => Some(Action::ScenariosDteExpand),
        KeyCode::Char('w') | KeyCode::Char('W') if app.active_tab == Tab::Scenarios => {
            Some(Action::ScenariosCycleStrikeWidth)
        }

        // Settings
        KeyCode::Up if app.active_tab == Tab::Settings => Some(Action::SettingsScrollUp),
        KeyCode::Down if app.active_tab == Tab::Settings => Some(Action::SettingsScrollDown),
        KeyCode::Char('a') | KeyCode::Char('A')
            if app.active_tab == Tab::Settings && app.settings_section_index == 2 =>
        {
            Some(Action::SettingsAddSymbol)
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter
            if app.active_tab == Tab::Settings && app.settings_section_index == 1 =>
        {
            Some(Action::SettingsEditConfig)
        }
        KeyCode::Char('r') | KeyCode::Char('R') if app.active_tab == Tab::Settings => {
            Some(Action::SettingsReset)
        }
        KeyCode::Delete if app.active_tab == Tab::Settings => Some(Action::SettingsDelete),

        // Logs tab
        KeyCode::Up if app.active_tab == Tab::Logs => Some(Action::LogScrollUp),
        KeyCode::Down if app.active_tab == Tab::Logs => Some(Action::LogScrollDown),
        KeyCode::PageUp if app.active_tab == Tab::Logs => Some(Action::LogPageUp),
        KeyCode::PageDown if app.active_tab == Tab::Logs => Some(Action::LogPageDown),
        KeyCode::Char('+') if app.active_tab == Tab::Logs => Some(Action::LogLevelUp),
        KeyCode::Char('-') if app.active_tab == Tab::Logs => Some(Action::LogLevelDown),
        KeyCode::Char('h') | KeyCode::Char('H') if app.active_tab == Tab::Logs => {
            Some(Action::LogHide)
        }
        KeyCode::Esc if app.active_tab == Tab::Logs => Some(Action::LogEscape),
        KeyCode::Char('e') | KeyCode::Char('E') if app.active_tab == Tab::Logs => {
            Some(Action::LogLevelError)
        }
        KeyCode::Char('w') | KeyCode::Char('W') if app.active_tab == Tab::Logs => {
            Some(Action::LogLevelWarn)
        }
        KeyCode::Char('i') | KeyCode::Char('I') if app.active_tab == Tab::Logs => {
            Some(Action::LogLevelInfo)
        }
        KeyCode::Char('d') | KeyCode::Char('D') if app.active_tab == Tab::Logs => {
            Some(Action::LogLevelDebug)
        }

        // Global commands
        KeyCode::Char('m') | KeyCode::Char('M') => Some(Action::ModeCycle),
        KeyCode::Char('s') | KeyCode::Char('S') if app.active_tab != Tab::Orders => {
            Some(Action::StrategyStart)
        }
        KeyCode::Char('t') | KeyCode::Char('T') if app.active_tab != Tab::Orders => {
            Some(Action::StrategyStop)
        }
        KeyCode::Char('k') | KeyCode::Char('K') if app.active_tab != Tab::Orders => {
            Some(Action::StrategyCancelAll)
        }
        KeyCode::Char('f') | KeyCode::Char('F') => Some(Action::ForceSnapshot),
        KeyCode::Char('p') | KeyCode::Char('P') => Some(Action::SplitPaneToggle),

        _ => None,
    }
}

/// Applies an action to the app state.
pub fn apply_action(app: &mut App, action: Action) {
    match action {
        Action::Quit => {
            app.should_quit = true;
        }
        Action::ShowHelp => {
            app.show_help = true;
        }
        Action::ToggleLogPanel => {
            app.show_log_panel = !app.show_log_panel;
        }
        Action::TabNext => {
            app.active_tab = app.active_tab.next();
        }
        Action::TabPrev => {
            app.active_tab = app.active_tab.prev();
        }
        Action::JumpToTab(n) => {
            app.active_tab = match n {
                1 => Tab::Dashboard,
                2 => Tab::Positions,
                3 => Tab::Charts,
                4 => Tab::Orders,
                5 => Tab::Alerts,
                6 => Tab::Yield,
                7 => Tab::Loans,
                8 => Tab::Scenarios,
                9 => Tab::Logs,
                0 => Tab::Settings,
                _ => app.active_tab.clone(),
            };
            // Trigger data fetch when entering Yield or Loans tab
            if app.active_tab == Tab::Yield {
                let wl = app.watchlist();
                if !wl.is_empty() {
                    let idx = app.yield_symbol_index.min(wl.len().saturating_sub(1));
                    let symbol = wl[idx].clone();
                    app.request_yield_fetch(&symbol);
                }
            } else if app.active_tab == Tab::Loans {
                app.request_loans_fetch();
            }
        }
        Action::YieldSymbolPrev => {
            let len = app.watchlist().len();
            if len > 0 {
                app.yield_symbol_index = (app.yield_symbol_index + len - 1) % len;
                let symbol = app.watchlist()[app.yield_symbol_index].clone();
                app.request_yield_fetch(&symbol);
            }
        }
        Action::YieldSymbolNext => {
            let len = app.watchlist().len();
            if len > 0 {
                app.yield_symbol_index = (app.yield_symbol_index + 1) % len;
                let symbol = app.watchlist()[app.yield_symbol_index].clone();
                app.request_yield_fetch(&symbol);
            }
        }
        Action::PositionsToggleCombo => {
            app.positions_combo_view = !app.positions_combo_view;
            app.positions_scroll = 0;
        }
        Action::PositionsScrollUp => {
            app.positions_scroll = app.positions_scroll.saturating_sub(1);
        }
        Action::PositionsScrollDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| {
                    crate::ui::positions_display_info(
                        &s.dto().positions,
                        app.positions_combo_view,
                        &app.positions_expanded_combos,
                    )
                    .0
                })
                .unwrap_or(0);
            if len > 0 {
                app.positions_scroll = (app.positions_scroll + 1).min(len - 1);
            }
        }
        Action::PositionsScrollPageUp => {
            app.positions_scroll = app.positions_scroll.saturating_sub(10);
        }
        Action::PositionsScrollPageDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| {
                    crate::ui::positions_display_info(
                        &s.dto().positions,
                        app.positions_combo_view,
                        &app.positions_expanded_combos,
                    )
                    .0
                })
                .unwrap_or(0);
            if len > 0 {
                app.positions_scroll = (app.positions_scroll + 10).min(len - 1);
            }
        }
        Action::PositionsDetail => {
            if let Some(ref snap) = app.snapshot() {
                let (_display_len, index_map, combo_key_per_row) =
                    crate::ui::positions_display_info(
                        &snap.dto().positions,
                        app.positions_combo_view,
                        &app.positions_expanded_combos,
                    );
                if let Some(Some(combo_key)) = combo_key_per_row.get(app.positions_scroll) {
                    if app.positions_expanded_combos.contains(combo_key) {
                        app.positions_expanded_combos.remove(combo_key);
                    } else {
                        app.positions_expanded_combos.insert(combo_key.clone());
                    }
                } else if let Some(Some(pos_idx)) = index_map.get(app.positions_scroll) {
                    if let Some(pos) = snap.dto().positions.get(*pos_idx) {
                        app.detail_popup = Some(DetailPopupContent::Position(pos.clone()));
                    }
                }
            }
        }
        Action::OrdersScrollUp => {
            app.orders_scroll = app.orders_scroll.saturating_sub(1);
        }
        Action::OrdersScrollDown => {
            let len = app.filtered_orders_len();
            if len > 0 {
                app.orders_scroll = (app.orders_scroll + 1).min(len - 1);
            }
        }
        Action::OrdersScrollPageUp => {
            app.orders_scroll = app.orders_scroll.saturating_sub(10);
        }
        Action::OrdersScrollPageDown => {
            let len = app.filtered_orders_len();
            if len > 0 {
                app.orders_scroll = (app.orders_scroll + 10).min(len - 1);
            }
        }
        Action::OrdersDetail => {
            if let Some(ref snap) = app.snapshot() {
                let filtered = app.filtered_orders(snap);
                let idx = app.orders_scroll.min(filtered.len().saturating_sub(1));
                if let Some(order) = filtered.get(idx) {
                    app.detail_popup = Some(DetailPopupContent::Order(order.clone()));
                }
            }
        }
        Action::OrdersFilterFocus => {
            app.order_filter.clear();
        }
        Action::OrdersFilterChar(c) => {
            app.order_filter.push(c);
        }
        Action::OrdersFilterBackspace => {
            app.order_filter.pop();
        }
        Action::OrdersFilterClear => {
            app.order_filter.clear();
        }
        Action::OrdersCancel => {
            if let Some(ref tx) = app.strategy_cmd_tx {
                let _ = tx.send(StrategyCommand::CancelAll);
            }
        }
        Action::LoansScrollUp => {
            app.loans_scroll = app.loans_scroll.saturating_sub(1);
        }
        Action::LoansScrollDown => {
            let len = app
                .loans_list
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|l| l.len())
                .unwrap_or(0);
            if len > 0 {
                app.loans_scroll = (app.loans_scroll + 1).min(len - 1);
            }
        }
        Action::LoansScrollPageUp => {
            app.loans_scroll = app.loans_scroll.saturating_sub(10);
        }
        Action::LoansScrollPageDown => {
            let len = app
                .loans_list
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|l| l.len())
                .unwrap_or(0);
            if len > 0 {
                app.loans_scroll = (app.loans_scroll + 10).min(len - 1);
            }
        }
        Action::AlertsScrollUp => {
            app.alerts_scroll = app.alerts_scroll.saturating_sub(1);
        }
        Action::AlertsScrollDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| s.dto().alerts.len())
                .unwrap_or(0);
            if len > 0 {
                app.alerts_scroll = (app.alerts_scroll + 1).min(len - 1);
            }
        }
        Action::AlertsScrollPageUp => {
            app.alerts_scroll = app.alerts_scroll.saturating_sub(10);
        }
        Action::AlertsScrollPageDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| s.dto().alerts.len())
                .unwrap_or(0);
            if len > 0 {
                app.alerts_scroll = (app.alerts_scroll + 10).min(len - 1);
            }
        }
        Action::ScenariosScrollUp => {
            app.scenarios_scroll = app.scenarios_scroll.saturating_sub(1);
        }
        Action::ScenariosScrollDown => {
            let filtered = crate::ui::filtered_scenarios(app);
            if !filtered.is_empty() {
                app.scenarios_scroll =
                    (app.scenarios_scroll + 1).min(filtered.len().saturating_sub(1));
            }
        }
        Action::ScenariosScrollPageUp => {
            app.scenarios_scroll = app.scenarios_scroll.saturating_sub(10);
        }
        Action::ScenariosScrollPageDown => {
            let filtered = crate::ui::filtered_scenarios(app);
            if !filtered.is_empty() {
                app.scenarios_scroll =
                    (app.scenarios_scroll + 10).min(filtered.len().saturating_sub(1));
            }
        }
        Action::ScenariosDetail => {
            let filtered = crate::ui::filtered_scenarios(app);
            let idx = app.scenarios_scroll.min(filtered.len().saturating_sub(1));
            if let Some(scenario) = filtered.get(idx) {
                app.detail_popup = Some(DetailPopupContent::Scenario(scenario.clone()));
            }
        }
        Action::ScenariosDteContract => {
            app.scenarios_dte_half_width = (app.scenarios_dte_half_width - 1).max(0);
        }
        Action::ScenariosDteExpand => {
            app.scenarios_dte_half_width = (app.scenarios_dte_half_width + 1).min(60);
        }
        Action::ScenariosExecute => {
            let filtered = crate::ui::filtered_scenarios(app);
            let idx = app.scenarios_scroll.min(filtered.len().saturating_sub(1));
            if let Some(scenario) = filtered.get(idx) {
                if let Some(ref tx) = app.strategy_cmd_tx {
                    let _ = tx.send(StrategyCommand::ExecuteScenario(scenario.clone()));
                }
            }
        }
        Action::ScenariosCycleStrikeWidth => {
            app.scenarios_strike_width_filter = match app.scenarios_strike_width_filter {
                None => Some(25),
                Some(25) => Some(50),
                Some(50) => Some(100),
                Some(_) => None,
            };
        }
        Action::SettingsScrollUp => {
            if app.settings_section_index == 2 {
                app.settings_symbol_index = app.settings_symbol_index.saturating_sub(1);
            } else if app.settings_section_index == 1 {
                app.settings_config_key_index = app.settings_config_key_index.saturating_sub(1);
            } else {
                app.settings_section_index = app.settings_section_index.saturating_sub(1);
            }
        }
        Action::SettingsScrollDown => {
            if app.settings_section_index == 2 {
                let len = app.watchlist().len();
                if len > 0 {
                    app.settings_symbol_index =
                        (app.settings_symbol_index + 1).min(len.saturating_sub(1));
                }
            } else if app.settings_section_index == 1 {
                app.settings_config_key_index = (app.settings_config_key_index + 1).min(4);
            } else {
                app.settings_section_index = (app.settings_section_index + 1).min(2);
            }
        }
        Action::SettingsAddSymbol => {
            if app.settings_section_index != 2 {
                return;
            }
            app.settings_add_symbol_input = Some(String::new());
        }
        Action::SettingsEditConfig => {
            if let Some((key, value)) = app.config_key_value_at(app.settings_config_key_index) {
                app.settings_edit_config_key = Some(key);
                app.settings_add_symbol_input = Some(value);
            }
        }
        Action::SettingsDelete => {
            let wl = app.watchlist();
            if !wl.is_empty() && app.settings_symbol_index < wl.len() {
                let mut list = app
                    .watchlist_override
                    .clone()
                    .unwrap_or_else(|| app.config.watchlist.clone());
                list.remove(app.settings_symbol_index);
                let new_len = list.len();
                app.watchlist_override = Some(list);
                app.settings_symbol_index =
                    app.settings_symbol_index.min(new_len.saturating_sub(1));
            }
        }
        Action::SettingsReset => {
            app.watchlist_override = None;
        }
        Action::LogScrollUp => {
            app.log_state.transition(tui_logger::TuiWidgetEvent::UpKey);
        }
        Action::LogScrollDown => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::DownKey);
        }
        Action::LogPageUp => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::PrevPageKey);
        }
        Action::LogPageDown => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::NextPageKey);
        }
        Action::LogLevelUp => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::PlusKey);
            app.log_display_level = match app.log_display_level {
                log::LevelFilter::Trace => log::LevelFilter::Debug,
                log::LevelFilter::Debug => log::LevelFilter::Info,
                log::LevelFilter::Info => log::LevelFilter::Warn,
                log::LevelFilter::Warn => log::LevelFilter::Error,
                log::LevelFilter::Error => log::LevelFilter::Error,
                _ => app.log_display_level,
            };
            tui_logger::set_default_level(app.log_display_level);
        }
        Action::LogLevelDown => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::MinusKey);
            app.log_display_level = match app.log_display_level {
                log::LevelFilter::Error => log::LevelFilter::Warn,
                log::LevelFilter::Warn => log::LevelFilter::Info,
                log::LevelFilter::Info => log::LevelFilter::Debug,
                log::LevelFilter::Debug => log::LevelFilter::Trace,
                log::LevelFilter::Trace => log::LevelFilter::Trace,
                _ => app.log_display_level,
            };
            tui_logger::set_default_level(app.log_display_level);
        }
        Action::LogHide => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::HideKey);
        }
        Action::LogEscape => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::EscapeKey);
        }
        Action::LogLevelError => {
            app.log_display_level = log::LevelFilter::Error;
            tui_logger::set_default_level(log::LevelFilter::Error);
        }
        Action::LogLevelWarn => {
            app.log_display_level = log::LevelFilter::Warn;
            tui_logger::set_default_level(log::LevelFilter::Warn);
        }
        Action::LogLevelInfo => {
            app.log_display_level = log::LevelFilter::Info;
            tui_logger::set_default_level(log::LevelFilter::Info);
        }
        Action::LogLevelDebug => {
            app.log_display_level = log::LevelFilter::Debug;
            tui_logger::set_default_level(log::LevelFilter::Debug);
        }
        Action::ModeCycle => {
            if let Some(ref tx) = app.strategy_cmd_tx {
                let current = app
                    .snapshot()
                    .as_ref()
                    .map(|s| s.dto().mode.to_uppercase())
                    .unwrap_or_else(|| "DRY-RUN".into());
                let current = if current == "TUI" {
                    "DRY-RUN"
                } else {
                    current.as_str()
                };
                let next = match current {
                    "LIVE" => "MOCK",
                    "MOCK" => "DRY-RUN",
                    _ => "LIVE",
                };
                let _ = tx.send(StrategyCommand::SetMode(next.to_string()));
            }
        }
        Action::StrategyStart => {
            if let Some(ref tx) = app.strategy_cmd_tx {
                let _ = tx.send(StrategyCommand::Start);
            }
        }
        Action::StrategyStop => {
            if let Some(ref tx) = app.strategy_cmd_tx {
                let _ = tx.send(StrategyCommand::Stop);
            }
        }
        Action::StrategyCancelAll => {
            if let Some(ref tx) = app.strategy_cmd_tx {
                let _ = tx.send(StrategyCommand::CancelAll);
            }
        }
        Action::ForceSnapshot => {
            if let Some(ref tx) = app.strategy_cmd_tx {
                let _ = tx.send(StrategyCommand::PublishSnapshot);
            }
        }
        Action::SplitPaneToggle => {
            app.split_pane = !app.split_pane;
            app.positions_scroll = 0;
        }
        Action::NoOp => {}
    }
}
