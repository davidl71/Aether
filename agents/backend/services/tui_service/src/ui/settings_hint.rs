use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::workspace::SettingsSection;

pub(crate) fn render_settings_hint_section(f: &mut Frame, app: &App, area: Rect) {
    let hint_text = match app.settings_section {
        SettingsSection::Config => {
            " 0 = Settings  ↑↓ key  e/Enter edit override  Active section: Config (editable) "
        }
        SettingsSection::Symbols => {
            " 0 = Settings  ↑↓ symbol  a add symbol  Del remove  Active section: Symbols "
        }
        SettingsSection::Sources => {
            " 0 = Settings  ↑↓ section  Active section: Data Sources (credential origin: env/keyring/file/built-in) "
        }
        SettingsSection::Health => {
            " 0 = Settings  ↑↓ section  Enter inspect  Active section: Backends "
        }
    };
    let hint = Line::from(Span::styled(
        hint_text,
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(Paragraph::new(hint), area);
}
