//! Settings tab input: navigation, edit/add flows, and credential entry helpers.
//!
//! This module translates key presses into [`Action`]s and applies settings-specific
//! mutations that don't belong in `app.rs`.

use crossterm::event::KeyCode;

use api::credentials;

use crate::app::App;
use crate::input::Action;
use crate::ui::settings::{
    alpaca_credential_key_for_row, credential_key_for_sources_row, ALPACA_CREDENTIAL_ROW_COUNT,
    SOURCES_TABLE_ROW_COUNT,
};
use crate::ui::ToastLevel;
use crate::workspace::{SettingsHealthFocus, SettingsSection};

pub(crate) fn settings_key_action(app: &App, key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Left => Some(Action::SettingsSectionPrev),
        KeyCode::Right => Some(Action::SettingsSectionNext),
        KeyCode::Up => Some(Action::SettingsScrollUp),
        KeyCode::Down => Some(Action::SettingsScrollDown),
        KeyCode::Char('a') | KeyCode::Char('A')
            if app.settings_section == SettingsSection::Symbols =>
        {
            Some(Action::SettingsAddSymbol)
        }
        KeyCode::Char('d') | KeyCode::Char('D')
            if app.settings_section == SettingsSection::Symbols =>
        {
            Some(Action::SettingsDelete)
        }
        KeyCode::Char('d') | KeyCode::Char('D')
            if app.settings_section == SettingsSection::Alpaca =>
        {
            Some(Action::SettingsDelete)
        }
        KeyCode::Char('d') | KeyCode::Char('D')
            if app.settings_section == SettingsSection::Sources
                && credential_key_for_sources_row(app.settings_sources_row).is_some() =>
        {
            Some(Action::SettingsDelete)
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter
            if app.settings_section == SettingsSection::Config =>
        {
            Some(Action::SettingsEditConfig)
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter
            if app.settings_section == SettingsSection::Alpaca =>
        {
            Some(Action::SettingsEditCredential)
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter
            if app.settings_section == SettingsSection::Sources
                && credential_key_for_sources_row(app.settings_sources_row).is_some() =>
        {
            Some(Action::SettingsEditCredential)
        }
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::SettingsReset),
        KeyCode::Delete => match app.settings_section {
            SettingsSection::Symbols | SettingsSection::Alpaca => Some(Action::SettingsDelete),
            SettingsSection::Sources
                if credential_key_for_sources_row(app.settings_sources_row).is_some() =>
            {
                Some(Action::SettingsDelete)
            }
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn apply_settings_action(app: &mut App, action: Action) -> bool {
    let sources_last = SOURCES_TABLE_ROW_COUNT.saturating_sub(1);
    match action {
        Action::SettingsScrollUp => {
            if app.settings_section == SettingsSection::Health {
                app.settings_health_focus = app.settings_health_focus.prev();
            } else if app.settings_section == SettingsSection::Symbols {
                if app.settings_symbol_index > 0 {
                    app.settings_symbol_index = app.settings_symbol_index.saturating_sub(1);
                } else {
                    app.settings_section = SettingsSection::Config;
                }
            } else if app.settings_section == SettingsSection::Config {
                if app.settings_config_key_index > 0 {
                    app.settings_config_key_index = app.settings_config_key_index.saturating_sub(1);
                } else {
                    app.settings_section = SettingsSection::Health;
                }
            } else if app.settings_section == SettingsSection::Sources {
                if app.settings_sources_row > 0 {
                    app.settings_sources_row -= 1;
                } else {
                    app.settings_section = SettingsSection::Symbols;
                }
            } else if app.settings_section == SettingsSection::Alpaca {
                if app.settings_alpaca_row > 0 {
                    app.settings_alpaca_row -= 1;
                } else {
                    app.settings_section = SettingsSection::Sources;
                    app.settings_sources_row = sources_last;
                }
            } else {
                app.settings_section = app.settings_section.prev();
            }
        }
        Action::SettingsScrollDown => {
            if app.settings_section == SettingsSection::Health {
                if app.settings_health_focus == SettingsHealthFocus::Transport {
                    app.settings_health_focus = SettingsHealthFocus::Services;
                } else {
                    app.settings_section = SettingsSection::Config;
                }
            } else if app.settings_section == SettingsSection::Symbols {
                let len = app.watchlist().len();
                if len > 0 && app.settings_symbol_index + 1 < len {
                    app.settings_symbol_index =
                        (app.settings_symbol_index + 1).min(len.saturating_sub(1));
                } else {
                    app.settings_section = SettingsSection::Sources;
                    app.settings_sources_row = 0;
                }
            } else if app.settings_section == SettingsSection::Config {
                let last = app.config_key_count().saturating_sub(1);
                if app.settings_config_key_index < last {
                    app.settings_config_key_index += 1;
                } else {
                    app.settings_section = SettingsSection::Symbols;
                }
            } else if app.settings_section == SettingsSection::Sources {
                if app.settings_sources_row < sources_last {
                    app.settings_sources_row += 1;
                } else {
                    app.settings_section = SettingsSection::Alpaca;
                    app.settings_alpaca_row = 0;
                }
            } else if app.settings_section == SettingsSection::Alpaca {
                let last = ALPACA_CREDENTIAL_ROW_COUNT.saturating_sub(1);
                if app.settings_alpaca_row < last {
                    app.settings_alpaca_row += 1;
                } else {
                    app.settings_section = SettingsSection::Health;
                    app.settings_health_focus = SettingsHealthFocus::Transport;
                }
            } else {
                app.settings_section = app.settings_section.next();
            }
        }
        Action::SettingsAddSymbol => {
            if app.settings_section != SettingsSection::Symbols {
                return true;
            }
            app.settings_add_symbol_input = Some(String::new());
            app.command_success("settings", "Add symbol mode active.");
        }
        Action::SettingsSectionPrev => {
            app.settings_section = app.settings_section.prev();
            if app.settings_section == SettingsSection::Health {
                app.settings_health_focus = SettingsHealthFocus::Transport;
            }
        }
        Action::SettingsSectionNext => {
            app.settings_section = app.settings_section.next();
            if app.settings_section == SettingsSection::Health {
                app.settings_health_focus = SettingsHealthFocus::Transport;
            }
        }
        Action::SettingsEditCredential => {
            let ck = match app.settings_section {
                SettingsSection::Alpaca => alpaca_credential_key_for_row(app.settings_alpaca_row),
                SettingsSection::Sources => {
                    credential_key_for_sources_row(app.settings_sources_row)
                }
                _ => None,
            };
            if let Some(ck) = ck {
                let existing = credentials::get_credential(ck).unwrap_or_default();
                app.settings_credential_edit_key = Some(ck);
                app.settings_credential_buffer = Some(existing);
                app.command_success("settings", "Credential edit (Enter save, Esc cancel).");
            }
            return true;
        }
        Action::SettingsDelete => {
            match app.settings_section {
                SettingsSection::Alpaca => {
                    if let Some(ck) = alpaca_credential_key_for_row(app.settings_alpaca_row) {
                        clear_credential_toast(app, ck);
                    }
                    return true;
                }
                SettingsSection::Sources => {
                    if let Some(ck) = credential_key_for_sources_row(app.settings_sources_row) {
                        clear_credential_toast(app, ck);
                    }
                    return true;
                }
                // Symbols: watchlist removal lives in `apply_loan_action`.
                SettingsSection::Symbols => return false,
                _ => return false,
            }
        }
        _ => return false,
    }
    true
}

fn clear_credential_toast(app: &mut App, ck: credentials::CredentialKey) {
    match credentials::delete_credential(ck) {
        Ok(()) => {
            app.push_toast(format!("Cleared {}.", ck.display_name()), ToastLevel::Info);
        }
        Err(e) => app.push_toast(
            format!("Could not clear credential: {e}"),
            ToastLevel::Warning,
        ),
    }
}
