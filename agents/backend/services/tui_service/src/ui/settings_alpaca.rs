//! Settings → Alpaca: store API key ID and secret (paper + live endpoints) in keyring or credential file for read-only quotes and account views.

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table},
    Frame,
};

use api::credentials::{credential_source, CredentialKey};

use crate::app::App;
use crate::workspace::SettingsSection;

use super::{section_active, section_block};

/// Rows: paper id, paper secret, live id, live secret.
pub const ALPACA_CREDENTIAL_ROW_COUNT: usize = 4;

pub(crate) fn alpaca_credential_key_for_row(row: usize) -> Option<CredentialKey> {
    match row {
        0 => Some(CredentialKey::AlpacaPaperApiKeyId),
        1 => Some(CredentialKey::AlpacaPaperSecretKey),
        2 => Some(CredentialKey::AlpacaLiveApiKeyId),
        3 => Some(CredentialKey::AlpacaLiveSecretKey),
        _ => None,
    }
}

fn row_label(row: usize) -> &'static str {
    match row {
        0 => "Paper  API key ID",
        1 => "Paper  secret",
        2 => "Live   API key ID",
        3 => "Live   secret",
        _ => "?",
    }
}

fn mask_status(key: CredentialKey) -> (String, Color) {
    let id_ok = credential_source(key).is_some();
    if id_ok {
        ("set".to_string(), Color::Green)
    } else {
        ("missing".to_string(), Color::Red)
    }
}

pub(crate) fn render_settings_alpaca_section(f: &mut Frame, app: &App, area: Rect) {
    let block = section_block(
        "Alpaca credentials (keyring/file; env APCA_* still overrides)",
        section_active(app, SettingsSection::Alpaca),
    );

    let rows: Vec<Row> = (0..ALPACA_CREDENTIAL_ROW_COUNT)
        .filter_map(|row| {
            let ck = alpaca_credential_key_for_row(row)?;
            let active =
                section_active(app, SettingsSection::Alpaca) && app.settings_alpaca_row == row;
            let (st, col) = mask_status(ck);
            let src = credential_source(ck)
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| "—".to_string());
            let row_style = if active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Some(Row::new([
                Cell::from(row_label(row)).style(row_style),
                Cell::from(st).style(Style::default().fg(col)),
                Cell::from(src).style(Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(22),
            Constraint::Length(8),
            Constraint::Min(6),
        ],
    )
    .header(
        Row::new(["Field", "Status", "Source"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(block);

    if app.settings_alpaca_edit_key.is_some() && app.settings_alpaca_buffer.is_some() {
        let key = app
            .settings_alpaca_edit_key
            .map(|k| k.display_name())
            .unwrap_or("Alpaca");
        let buf = app.settings_alpaca_buffer.as_deref().unwrap_or("");
        let edit = ratatui::widgets::Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Edit ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    key,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{buf}_"),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "[Enter] save to keyring/file  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(section_block(
            "Alpaca — edit",
            section_active(app, SettingsSection::Alpaca),
        ));
        f.render_widget(edit, area);
    } else {
        f.render_widget(table, area);
    }
}
