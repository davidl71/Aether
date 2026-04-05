//! Central keyboard dispatch pipeline (`docs/TUI_PANE_MODEL.md` §6).
//!
//! Stages run **in order**; the first match wins. Precedence aligns with
//! [`crate::app::App::input_mode`] (overlays and palette before tab chrome).

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, InputMode};
use crate::input::Action;
use crate::input_loans::loan_form_key_action;
use crate::input_shell::{global_key_action, shell_key_action};
use crate::input_tabs::tab_key_action;

/// Map a key press to an [`Action`] if this layer handles it.
pub fn dispatch_key_event(app: &App, key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    // macOS: Cmd+Shift+P opens command palette (before plain Super handling).
    if key
        .modifiers
        .contains(KeyModifiers::SUPER | KeyModifiers::SHIFT)
    {
        if matches!(key.code, KeyCode::Char('p') | KeyCode::Char('P')) {
            return Some(Action::CommandPalette);
        }
    }

    if key.modifiers.contains(KeyModifiers::SUPER) {
        return handle_macos_cmd_key(key.code);
    }

    if app.command_palette.visible {
        return handle_command_palette_input(app, key.code);
    }

    let input_mode = app.input_mode();

    if let Some(action) = global_key_action(input_mode, key.code) {
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

fn handle_command_palette_input(app: &App, key: KeyCode) -> Option<Action> {
    let palette = &app.command_palette;
    let mode = app.app_mode;
    let tab = app.active_tab;

    #[cfg(feature = "tui-interact")]
    match key {
        KeyCode::Tab => return Some(Action::CommandPaletteFocusNext),
        KeyCode::BackTab => return Some(Action::CommandPaletteFocusPrev),
        _ => {}
    }

    match key {
        KeyCode::Esc => Some(Action::CommandPalette),
        KeyCode::Enter => {
            if let Some(cmd) = palette.selected_command_if_available(mode, tab) {
                Some(cmd.action)
            } else {
                Some(Action::NoOp)
            }
        }
        KeyCode::Up => Some(Action::CommandPalettePrev),
        KeyCode::Down => Some(Action::CommandPaletteNext),
        KeyCode::Backspace => {
            #[cfg(feature = "tui-interact")]
            if !app.command_palette_interact.allows_field_edit() {
                return Some(Action::NoOp);
            }
            Some(Action::CommandPaletteBackspace)
        }
        KeyCode::Char(c) => {
            #[cfg(feature = "tui-interact")]
            if !app.command_palette_interact.allows_field_edit() {
                return Some(Action::NoOp);
            }
            Some(Action::CommandPaletteChar(c))
        }
        _ => Some(Action::NoOp),
    }
}
