use crossterm::event::KeyCode;

use crate::app::{App, CommandStatusView};
use crate::input::Action;
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
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter
            if app.settings_section == SettingsSection::Config =>
        {
            Some(Action::SettingsEditConfig)
        }
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::SettingsReset),
        KeyCode::Delete => Some(Action::SettingsDelete),
        _ => None,
    }
}

pub(crate) fn apply_settings_action(app: &mut App, action: Action) -> bool {
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
                app.settings_section = SettingsSection::Symbols;
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
                }
            } else if app.settings_section == SettingsSection::Config {
                let last = app.config_key_count().saturating_sub(1);
                if app.settings_config_key_index < last {
                    app.settings_config_key_index += 1;
                } else {
                    app.settings_section = SettingsSection::Symbols;
                }
            } else if app.settings_section == SettingsSection::Health {
                app.settings_section = SettingsSection::Config;
            } else {
                app.settings_section = app.settings_section.next();
            }
        }
        Action::SettingsAddSymbol => {
            if app.settings_section != SettingsSection::Symbols {
                return true;
            }
            app.settings_add_symbol_input = Some(String::new());
            app.set_command_status(CommandStatusView::success(
                "settings",
                "Add symbol mode active.",
            ));
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
        _ => return false,
    }
    true
}
