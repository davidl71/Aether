//! Input handling: key events to application actions.
//!
//! Converts crossterm key events into typed actions, keeping input parsing
//! separate from application state mutation. Legacy strategy/scenario “execute”
//! actions surface as disabled no-ops in data-exploration mode
//! (`docs/DATA_EXPLORATION_MODE.md`).

use api::RuntimePositionDto;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, DetailPopupContent, InputMode};
use crate::input_loans::{apply_loan_action, loan_form_key_action};
use crate::input_settings::apply_settings_action;
use crate::input_shell::{apply_shell_action, global_key_action, shell_key_action};
use crate::input_tabs::tab_key_action;
use crate::input_views::apply_view_action;

/// Actions that can result from key events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Quit,
    ShowHelp,
    ToggleLogPanel,
    ToggleTreePanel,
    TabNext,
    TabPrev,
    JumpToTab(u8),
    YieldSymbolPrev,
    YieldSymbolNext,
    YieldCurveScrollUp,
    YieldCurveScrollDown,
    YieldCurveDetail,
    YieldRefresh,
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
    LoansNewLoan,
    LoansBulkImportFocus,
    LoansImportPathChar(char),
    LoansImportPathBackspace,
    LoansImportPathEnter,
    LoansImportPathEscape,
    DiscountBankScrollUp,
    DiscountBankScrollDown,
    DiscountBankScrollPageUp,
    DiscountBankScrollPageDown,
    DiscountBankRefresh,
    LoansInputChar(char),
    LoansInputBackspace,
    LoansInputEscape,
    LoansInputEnter,
    LoansInputNavUp,
    LoansInputNavDown,
    AlertsScrollUp,
    AlertsScrollDown,
    AlertsScrollPageUp,
    AlertsScrollPageDown,
    DashboardScrollUp,
    DashboardScrollDown,
    DashboardNavigateToChart,
    ScenariosScrollUp,
    ScenariosScrollDown,
    ScenariosScrollPageUp,
    ScenariosScrollPageDown,
    ScenariosDetail,
    ScenariosDteContract,
    ScenariosDteExpand,
    ScenariosCycleStrikeWidth,
    /// Deprecated scenario “execute” binding; exploration UI only (no submission).
    ScenariosExecute,
    ChartSearchFocus,
    ChartSearchChar(char),
    ChartSearchBackspace,
    ChartSearchUp,
    ChartSearchDown,
    ChartSearchEnter,
    ChartSearchEscape,
    ChartSearchFirst,
    ChartSearchLast,
    ChartPillLeft,
    ChartPillRight,
    ChartPillUp,
    ChartPillDown,
    ChartPillSelect,
    SettingsScrollUp,
    SettingsScrollDown,
    SettingsAddSymbol,
    SettingsEditConfig,
    SettingsEditCredential,
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
    TreeUp,
    TreeDown,
    TreeLeft,
    TreeRight,
    TreeToggle,
    TreeEscape,
    ModeCycle,
    CommandPalette,
    CommandPalettePrev,
    CommandPaletteNext,
    CommandPaletteBackspace,
    CommandPaletteChar(char),
    CloseDetail,
    MouseScrollUp,
    MouseScrollDown,
    StrategyStart,
    StrategyStop,
    StrategyCancelAll,
    OrdersCancel,
    ForceSnapshot,
    SplitPaneToggle,
    WorkspaceFocusPrev,
    WorkspaceFocusNext,
    SettingsSectionPrev,
    SettingsSectionNext,
    FmpDetail,
    NoOp,
}

/// Converts a key event to an action, or None if the key is not handled.
pub fn key_to_action(app: &App, key: KeyEvent) -> Option<Action> {
    use crossterm::event::KeyModifiers;

    // Only handle Press events (crossterm 0.27+)
    if key.kind != crossterm::event::KeyEventKind::Press {
        return None;
    }

    // macOS: Cmd+Shift+P opens command palette (VS Code–style); must run before plain Cmd handler.
    if key
        .modifiers
        .contains(KeyModifiers::SUPER | KeyModifiers::SHIFT)
    {
        if matches!(key.code, KeyCode::Char('p') | KeyCode::Char('P')) {
            return Some(Action::CommandPalette);
        }
    }

    // Handle macOS Cmd (Super) key shortcuts
    if key.modifiers.contains(KeyModifiers::SUPER) {
        return handle_macos_cmd_key(key.code);
    }

    // Handle command palette input first
    if app.command_palette.visible {
        return handle_command_palette_input(&app.command_palette, key.code);
    }

    let input_mode = app.input_mode();

    if let Some(action) = global_key_action(input_mode.clone(), key.code) {
        return Some(action);
    }

    match input_mode {
        InputMode::Help | InputMode::DetailPopup => return Some(Action::NoOp),
        InputMode::SettingsEditConfig
        | InputMode::SettingsAddSymbol
        | InputMode::SettingsCredentialEntry => return Some(Action::NoOp),
        InputMode::LoanForm => return loan_form_key_action(key.code),
        _ => {}
    }

    if let Some(action) = tab_key_action(app, key.code, input_mode) {
        return Some(action);
    }

    shell_key_action(app, key.code)
}

fn handle_macos_cmd_key(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Quit),
        KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::CloseDetail),
        // macOS idiom: Preferences / Settings
        KeyCode::Char(',') => Some(Action::JumpToTab(0)),
        KeyCode::Char('/') => Some(Action::ShowHelp),
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
        KeyCode::Char('p') | KeyCode::Char('P') => Some(Action::SplitPaneToggle),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::ForceSnapshot),
        _ => None,
    }
}

fn handle_command_palette_input(
    palette: &crate::discoverability::CommandPalette,
    key: KeyCode,
) -> Option<Action> {
    match key {
        KeyCode::Esc => Some(Action::CommandPalette),
        KeyCode::Enter => {
            // Confirm: run the selected palette action
            if let Some(cmd) = palette.selected_command() {
                Some(cmd.action)
            } else {
                Some(Action::NoOp)
            }
        }
        KeyCode::Up => Some(Action::CommandPalettePrev),
        KeyCode::Down => Some(Action::CommandPaletteNext),
        KeyCode::Backspace => Some(Action::CommandPaletteBackspace),
        KeyCode::Char(c) => Some(Action::CommandPaletteChar(c)),
        _ => Some(Action::NoOp),
    }
}

/// True while the user is typing or navigating inside the command palette.
fn command_palette_input_action(action: &Action) -> bool {
    matches!(
        action,
        Action::CommandPalette
            | Action::CommandPalettePrev
            | Action::CommandPaletteNext
            | Action::CommandPaletteBackspace
            | Action::CommandPaletteChar(_)
            | Action::NoOp
    )
}

/// Applies an action to the app state.
pub fn apply_action(app: &mut App, action: Action) {
    if app.command_palette.visible && !command_palette_input_action(&action) {
        app.command_palette.hide();
    }
    if apply_settings_action(app, action) {
        return;
    }
    if apply_shell_action(app, action) {
        return;
    }
    if apply_loan_action(app, action) {
        return;
    }
    if apply_view_action(app, action) {
        return;
    }
    match action {
        Action::YieldRefresh => {
            let watchlist = app.watchlist();
            let symbol = watchlist
                .get(
                    app.yield_symbol_index
                        .min(watchlist.len().saturating_sub(1)),
                )
                .cloned()
                .unwrap_or_default();
            app.request_yield_fetch(&symbol);
        }
        Action::YieldSymbolPrev => {
            let len = app.watchlist().len();
            if len > 0 {
                app.yield_symbol_index = (app.yield_symbol_index + len - 1) % len;
                app.yield_curve_table.reset();
                app.sync_yield_curve_from_cache();
            }
        }
        Action::YieldSymbolNext => {
            let len = app.watchlist().len();
            if len > 0 {
                app.yield_symbol_index = (app.yield_symbol_index + 1) % len;
                app.yield_curve_table.reset();
                app.sync_yield_curve_from_cache();
            }
        }
        Action::YieldCurveScrollUp => {
            app.yield_curve_table.move_up();
        }
        Action::YieldCurveScrollDown => {
            if let Some(ref curve) = app.yield_curve {
                app.yield_curve_table.move_down(curve.point_count);
            }
        }
        Action::YieldCurveDetail => {
            if let Some(ref curve) = app.yield_curve {
                if let Some(point) = curve.points.get(app.yield_curve_table.selected()) {
                    app.detail_popup =
                        Some(crate::app::DetailPopupContent::YieldPoint(point.clone()));
                }
            }
        }
        Action::PositionsToggleCombo => {
            app.positions_combo_view = !app.positions_combo_view;
            app.positions_table.reset();
        }
        Action::MouseScrollUp => match app.active_tab {
            crate::app::Tab::Positions => {
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
                app.positions_table.shift_selected(-3, len);
            }
            crate::app::Tab::Orders => {
                app.orders_table
                    .shift_selected(-3, app.filtered_orders_len());
            }
            crate::app::Tab::Dashboard => {
                let len = app
                    .snapshot()
                    .as_ref()
                    .map(|s| s.inner.symbols.len())
                    .unwrap_or(0);
                app.dashboard_table.shift_selected(-3, len);
            }
            _ => {}
        },
        Action::MouseScrollDown => match app.active_tab {
            crate::app::Tab::Positions => {
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
                app.positions_table.shift_selected(3, len);
            }
            crate::app::Tab::Orders => {
                let len = app.filtered_orders_len();
                app.orders_table.shift_selected(3, len);
            }
            crate::app::Tab::Dashboard => {
                let len = app
                    .snapshot()
                    .as_ref()
                    .map(|s| s.inner.symbols.len())
                    .unwrap_or(0);
                app.dashboard_table.shift_selected(3, len);
            }
            _ => {}
        },
        Action::PositionsScrollUp => {
            app.positions_table.move_up();
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
            app.positions_table.move_down(len);
        }
        Action::PositionsScrollPageUp => {
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
            app.positions_table.shift_selected(-10, len);
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
            app.positions_table.shift_selected(10, len);
        }
        Action::PositionsDetail => {
            // Collect what we need while borrow is live, then drop it before mutating app.
            enum PosDetailAction {
                ToggleCombo((String, String, String)),
                ShowPosition(RuntimePositionDto),
            }
            let detail_action: Option<PosDetailAction> = app.snapshot().as_ref().and_then(|snap| {
                let (_display_len, index_map, combo_key_per_row) =
                    crate::ui::positions_display_info(
                        &snap.dto().positions,
                        app.positions_combo_view,
                        &app.positions_expanded_combos,
                    );
                if let Some(Some(combo_key)) = combo_key_per_row.get(app.positions_table.selected())
                {
                    Some(PosDetailAction::ToggleCombo(combo_key.clone()))
                } else if let Some(Some(pos_idx)) = index_map.get(app.positions_table.selected()) {
                    snap.dto()
                        .positions
                        .get(*pos_idx)
                        .map(|pos| PosDetailAction::ShowPosition(pos.clone()))
                } else {
                    None
                }
            });
            match detail_action {
                Some(PosDetailAction::ToggleCombo(combo_key)) => {
                    if app.positions_expanded_combos.contains(&combo_key) {
                        app.positions_expanded_combos.remove(&combo_key);
                    } else {
                        app.positions_expanded_combos.insert(combo_key);
                    }
                }
                Some(PosDetailAction::ShowPosition(pos)) => {
                    let is_opt = pos
                        .position_type
                        .as_deref()
                        .map(|t| t == "OPT" || t == "OPTION")
                        .unwrap_or(false);
                    app.detail_popup = Some(DetailPopupContent::Position(pos.clone(), None));
                    if is_opt {
                        app.fetch_greeks_for_position(&pos);
                    }
                }
                None => {}
            }
        }
        Action::SettingsEditConfig => {
            if let Some((key, value)) = app.config_key_value_at(app.settings_config_key_index) {
                app.settings_edit_config_key = Some(key);
                app.settings_add_symbol_input = Some(value);
                app.set_command_status(crate::app::CommandStatusView::success(
                    "settings",
                    "Config edit mode active.",
                ));
            }
        }
        Action::SettingsScrollUp
        | Action::SettingsScrollDown
        | Action::SettingsAddSymbol
        | Action::SettingsSectionPrev
        | Action::SettingsSectionNext
        | Action::NoOp => {}
        _ => {}
    }
}
