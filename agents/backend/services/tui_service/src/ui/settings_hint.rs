use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::ui::text_trunc;
use crate::workspace::{SettingsHealthFocus, SettingsSection};

pub(crate) fn render_settings_hint_section(f: &mut Frame, app: &App, area: Rect) {
    let compact = area.width < 100;
    let hint_text: String = match app.settings_section {
        SettingsSection::Config if compact => {
            "0 Settings  ↑↓  e edit  · Config".to_string()
        }
        SettingsSection::Config => {
            " 0 = Settings  ↑↓ key  e/Enter edit override  Active section: Config (editable) "
                .to_string()
        }
        SettingsSection::Symbols if compact => {
            "0 Settings  ↑↓  a add  Del del  · Symbols".to_string()
        }
        SettingsSection::Symbols => {
            " 0 = Settings  ↑↓ symbol  a add symbol  Del remove  Active section: Symbols "
                .to_string()
        }
        SettingsSection::Sources if compact => {
            "0 Settings  ↑↓  e key  d clear  · Sources".to_string()
        }
        SettingsSection::Sources => {
            " 0 = Settings  ↑↓ row  e/Enter edit key  d/Del clear  Active section: Data Sources (origin: env/keyring/file) "
                .to_string()
        }
        SettingsSection::Alpaca if compact => {
            "0 Settings  ↑↓  e edit  d clear  · Alpaca".to_string()
        }
        SettingsSection::Alpaca => {
            " 0 = Settings  ↑↓ field  e/Enter edit  d/Del clear  Active section: Alpaca credentials "
                .to_string()
        }
        SettingsSection::Health if compact => match app.settings_health_focus {
            SettingsHealthFocus::Transport => "0 Settings  ↑↓ T/S  · Health/Transport".to_string(),
            SettingsHealthFocus::Services => "0 Settings  ↑↓ T/S  · Health/Services".to_string(),
        },
        SettingsSection::Health => match app.settings_health_focus {
            SettingsHealthFocus::Transport => {
                " 0 = Settings  ↑↓ transport/services  Active section: Health / Transport ".to_string()
            }
            SettingsHealthFocus::Services => {
                " 0 = Settings  ↑↓ transport/services  Active section: Health / Services ".to_string()
            }
        },
    };
    let hint_text = if compact && area.width > 0 {
        let max = area.width as usize;
        text_trunc::truncate_chars(&hint_text, max)
    } else {
        hint_text
    };
    let hint = Line::from(Span::styled(
        hint_text,
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(Paragraph::new(hint), area);
}
