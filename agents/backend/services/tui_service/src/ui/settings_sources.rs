use api::credentials::CredentialKey;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::app::App;
use crate::config::credential_scope;
use crate::workspace::SettingsSection;

use super::{section_active, section_block};

/// Rows in [`settings_source_rows`] / [`credential_key_for_sources_row`] (yahoo → alpaca_live).
pub(crate) const SOURCES_TABLE_ROW_COUNT: usize = 9;

/// Credential editable from Settings → Sources (`e` / `d` on a row with a key).
pub(crate) fn credential_key_for_sources_row(row: usize) -> Option<CredentialKey> {
    match row {
        1 => Some(CredentialKey::FmpApiKey),
        3 => Some(CredentialKey::PolygonApiKey),
        4 => Some(CredentialKey::TaseApiKey),
        5 => Some(CredentialKey::FredApiKey),
        _ => None,
    }
}

pub(crate) fn render_settings_sources_section(f: &mut Frame, app: &App, area: Rect) {
    let sources_block = section_block(
        "Data sources (e edit / d clear on FMP Polygon TASE FRED rows)",
        section_active(app, SettingsSection::Sources),
    );
    let source_rows = settings_source_rows(app);
    let sources_table = Table::new(
        source_rows,
        [
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(28),
            Constraint::Length(10),
            Constraint::Min(10),
        ],
    )
    .header(
        Row::new(["Provider", "Pri", "Credential", "Status", "Purpose"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(sources_block);
    f.render_widget(sources_table, area);
}

fn settings_source_rows(app: &App) -> Vec<Row<'static>> {
    let live_source = app
        .live_market_data_source
        .as_ref()
        .map(|m| m.source.to_lowercase());

    struct SourceDef {
        name: &'static str,
        priority: &'static str,
        cred_key: &'static str,
        note: &'static str,
    }
    let sources = [
        SourceDef {
            name: "yahoo",
            priority: "50",
            cred_key: "(free)",
            note: "market quotes",
        },
        SourceDef {
            name: "fmp",
            priority: "60",
            cred_key: "FMP_API_KEY",
            note: "market + fundamentals",
        },
        SourceDef {
            name: "mock",
            priority: "0",
            cred_key: "(fixture/demo)",
            note: "deterministic provider",
        },
        SourceDef {
            name: "polygon",
            priority: "70",
            cred_key: "POLYGON_API_KEY",
            note: "market quotes (WebSocket)",
        },
        SourceDef {
            name: "tase",
            priority: "—",
            cred_key: "TASE_API_KEY",
            note: "Israeli exchange",
        },
        SourceDef {
            name: "fred",
            priority: "—",
            cred_key: "FRED_API_KEY",
            note: "yield benchmarks (SOFR/Treasury)",
        },
        SourceDef {
            name: "tws",
            priority: "100",
            cred_key: "(TWS connection)",
            note: "IB broker push (highest priority)",
        },
        SourceDef {
            name: "alpaca_paper",
            priority: "55",
            cred_key: "APCA_API_KEY_ID/SECRET",
            note: "paper — env or keyring (alpaca_paper_*)",
        },
        SourceDef {
            name: "alpaca_live",
            priority: "75",
            cred_key: "APCA_API_KEY_ID/SECRET",
            note: "live — same env names; keyring alpaca_live_*",
        },
    ];

    let configured_provider = app
        .snapshot()
        .as_ref()
        .and_then(|s| s.inner.market_data_source.as_deref().map(str::to_lowercase))
        .unwrap_or_default();

    sources
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let scope = credential_scope(s.name);
            let has_cred = match s.name {
                "yahoo" | "tws" | "mock" => true,
                name => *app.credential_status.get(name).unwrap_or(&false),
            };
            let credential_label = match s.name {
                "yahoo" | "tws" | "mock" => format!("{} [{}]", s.cred_key, scope.label()),
                name => match app.credential_source.get(name) {
                    Some(source) => format!("{} [{}] [{}]", s.cred_key, source, scope.label()),
                    None => format!("{} [{}]", s.cred_key, scope.label()),
                },
            };
            let is_live = live_source.as_deref() == Some(s.name);
            let is_configured = configured_provider == s.name || configured_provider == "all";

            let (status_label, status_color) = if is_live {
                ("● LIVE", Color::Green)
            } else if !has_cred && s.name != "yahoo" && s.name != "tws" && s.name != "mock" {
                ("✗ no key", Color::Red)
            } else if is_configured || s.name == "yahoo" {
                ("idle", Color::DarkGray)
            } else {
                ("disabled", Color::DarkGray)
            };

            let cred_color = if has_cred || s.name == "yahoo" || s.name == "tws" || s.name == "mock"
            {
                Color::Green
            } else {
                Color::Red
            };

            let row_active =
                section_active(app, SettingsSection::Sources) && app.settings_sources_row == idx;
            let name_style = if row_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new([
                Cell::from(s.name).style(name_style),
                Cell::from(s.priority),
                Cell::from(credential_label).style(Style::default().fg(cred_color)),
                Cell::from(status_label).style(Style::default().fg(status_color)),
                Cell::from(s.note).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect()
}
