use crossterm::event::KeyCode;

use crate::app::{App, CommandStatusView, InputMode, Tab};
use crate::input::Action;

pub(crate) fn global_key_action(input_mode: InputMode, key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Quit),
        KeyCode::Char('?') => Some(Action::ShowHelp),
        KeyCode::Char(':') => Some(Action::CommandPalette),
        KeyCode::Char('`') | KeyCode::Char('~') => Some(Action::ToggleLogPanel),
        KeyCode::Char('g') | KeyCode::Char('G') => Some(Action::ToggleTreePanel),
        KeyCode::Esc if matches!(input_mode, InputMode::Normal | InputMode::LogPanel) => {
            Some(Action::ToggleLogPanel)
        }
        KeyCode::Esc if matches!(input_mode, InputMode::TreePanel) => Some(Action::ToggleTreePanel),
        _ => None,
    }
}

pub(crate) fn shell_key_action(app: &App, key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Tab if workspace_focus_target(app, true).is_some() => {
            Some(Action::WorkspaceFocusNext)
        }
        KeyCode::BackTab if workspace_focus_target(app, false).is_some() => {
            Some(Action::WorkspaceFocusPrev)
        }
        KeyCode::Tab => Some(Action::TabNext),
        KeyCode::BackTab => Some(Action::TabPrev),
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
        KeyCode::Char('f') | KeyCode::Char('F')
            if matches!(app.active_tab, Tab::Dashboard | Tab::Positions) =>
        {
            Some(Action::FmpDetail)
        }
        KeyCode::Char('f') | KeyCode::Char('F') => Some(Action::ForceSnapshot),
        KeyCode::Char('p') | KeyCode::Char('P') => Some(Action::SplitPaneToggle),
        _ => None,
    }
}

pub(crate) fn apply_shell_action(app: &mut App, action: Action) -> bool {
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
        Action::ToggleTreePanel => {
            app.show_tree_panel = !app.show_tree_panel;
            if app.show_tree_panel {
                crate::ui::tree_panel::ensure_initialized(app);
            }
        }
        Action::CloseDetail => {
            app.detail_popup = None;
            app.show_help = false;
            app.show_log_panel = false;
            app.show_tree_panel = false;
        }
        Action::TabNext => {
            set_active_tab(app, app.active_tab.next());
        }
        Action::TabPrev => {
            set_active_tab(app, app.active_tab.prev());
        }
        Action::JumpToTab(n) => {
            let target = match n {
                1 => Tab::Dashboard,
                2 => Tab::Positions,
                3 => Tab::Charts,
                4 => Tab::Orders,
                5 => Tab::Alerts,
                6 => Tab::Yield,
                7 => Tab::Loans,
                8 => Tab::DiscountBank,
                9 => Tab::Scenarios,
                0 => Tab::Settings,
                _ => app.active_tab,
            };
            set_active_tab(app, target);
        }
        Action::ModeCycle => {
            app.push_toast(
                "NAV / EDIT / VIEW follow your focus (forms, search, overlays). Esc closes overlays. ? help  : palette",
                crate::ui::ToastLevel::Info,
            );
        }
        Action::CommandPalette => {
            app.command_palette.toggle();
        }
        Action::CommandPalettePrev => {
            app.command_palette.select_prev();
        }
        Action::CommandPaletteNext => {
            app.command_palette.select_next();
        }
        Action::CommandPaletteBackspace => {
            app.command_palette.backspace();
        }
        Action::CommandPaletteChar(c) => {
            app.command_palette.push_char(c);
        }
        Action::StrategyStart => {
            app.set_command_status(CommandStatusView::disabled("start"));
        }
        Action::StrategyStop => {
            app.set_command_status(CommandStatusView::disabled("stop"));
        }
        Action::StrategyCancelAll => {
            app.set_command_status(CommandStatusView::disabled("cancel_all"));
        }
        Action::ForceSnapshot => {
            app.set_command_status(CommandStatusView::disabled("publish_snapshot"));
        }
        Action::FmpDetail => {
            let symbol = if app.active_tab == Tab::Dashboard {
                app.snapshot().as_ref().and_then(|snap| {
                    snap.inner
                        .symbols
                        .get(app.dashboard_table.selected())
                        .map(|s| s.symbol.clone())
                })
            } else {
                app.snapshot().as_ref().and_then(|snap| {
                    let (_display_len, index_map, _combo_key_per_row) =
                        crate::ui::positions_display_info(
                            &snap.dto().positions,
                            app.positions_combo_view,
                            &app.positions_expanded_combos,
                        );
                    if let Some(Some(pos_idx)) = index_map.get(app.positions_table.selected()) {
                        snap.dto().positions.get(*pos_idx).map(|p| {
                            p.symbol
                                .split_whitespace()
                                .next()
                                .unwrap_or(&p.symbol)
                                .to_string()
                        })
                    } else {
                        None
                    }
                })
            };
            if let Some(sym) = symbol {
                app.fetch_fmp(sym);
            }
        }
        Action::SplitPaneToggle => {
            app.split_pane = !app.split_pane;
            app.positions_table.reset();
        }
        Action::WorkspaceFocusPrev => {
            if let Some(target) = workspace_focus_target(app, false) {
                set_active_tab(app, target);
            }
        }
        Action::WorkspaceFocusNext => {
            if let Some(target) = workspace_focus_target(app, true) {
                set_active_tab(app, target);
            }
        }
        _ => return false,
    }
    true
}

fn workspace_focus_target(app: &App, forward: bool) -> Option<Tab> {
    let tabs = app.visible_workspace_spec()?.tabs;
    let index = tabs.iter().position(|tab| *tab == app.active_tab)?;
    let next = if forward {
        (index + 1) % tabs.len()
    } else {
        (index + tabs.len() - 1) % tabs.len()
    };
    tabs.get(next).copied()
}

fn set_active_tab(app: &mut App, tab: Tab) {
    app.active_tab = tab;
    if app.active_tab != Tab::Loans {
        app.loan_import_path = None;
    }
    if app.active_tab == Tab::Yield {
        app.sync_yield_curve_from_cache();
    } else if app.active_tab == Tab::Loans {
        app.request_loans_fetch_if_uncached();
    } else if app.active_tab == Tab::DiscountBank {
        app.request_discount_bank_fetch();
    }
}
