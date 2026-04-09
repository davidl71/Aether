//! Settings tab: backend health, editable config overrides, and watchlist management.

#[path = "settings_alpaca.rs"]
mod alpaca_section;
#[path = "settings_config.rs"]
mod config_section;
#[path = "settings_health.rs"]
mod health_section;
#[path = "settings_hint.rs"]
mod hint_section;
#[path = "settings_sources.rs"]
mod sources_section;
#[path = "settings_symbols.rs"]
mod symbols_section;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::workspace::SettingsSection;

pub(crate) use alpaca_section::{
    alpaca_credential_key_for_row, render_settings_alpaca_section, ALPACA_CREDENTIAL_ROW_COUNT,
};
pub(crate) use config_section::render_settings_config_section;
pub(crate) use health_section::render_settings_health_section;
pub(crate) use hint_section::render_settings_hint_section;
pub(crate) use sources_section::{
    credential_key_for_sources_row, render_settings_sources_section, SOURCES_TABLE_ROW_COUNT,
};
pub(crate) use symbols_section::render_settings_symbols_section;

#[derive(Clone, Copy)]
pub(crate) struct SettingsLayout {
    pub health: Rect,
    pub config: Rect,
    pub symbols: Rect,
    pub sources: Rect,
    pub alpaca: Rect,
    pub hint: Rect,
}

/// Renders settings sub-panels only (no credential modal). Use from embedded
/// layouts (e.g. Operations workspace); full Settings tab also runs the modal.
pub(crate) fn render_settings_sections(f: &mut Frame, app: &App, layout: SettingsLayout) {
    render_settings_health_section(f, app, layout.health);
    render_settings_config_section(f, app, layout.config);
    render_settings_symbols_section(f, app, layout.symbols);
    render_settings_sources_section(f, app, layout.sources);
    render_settings_alpaca_section(f, app, layout.alpaca);
    render_settings_hint_section(f, app, layout.hint);
}

pub(crate) fn settings_layout(area: Rect) -> SettingsLayout {
    settings_layout_with_min_width(area, 110)
}

/// Like `settings_layout`, but tuned for narrower panes (e.g. the Operations workspace
/// settings column) so we still get the 2-column layout in typical terminals.
pub(crate) fn settings_layout_embedded(area: Rect) -> SettingsLayout {
    settings_layout_with_min_width(area, 92)
}

/// Distribute `total` rows across the four stacked Settings sections (config, symbols,
/// sources, alpaca). Lengths sum to `total` (some may be 0 when `total` is very small).
fn settings_stacked_mid_heights(total: u16) -> [u16; 4] {
    let t = total;
    let base = t / 4;
    let rem = t % 4;
    let mut h = [base; 4];
    for i in 0..usize::from(rem) {
        h[i] = h[i].saturating_add(1);
    }
    h
}

fn settings_layout_with_min_width(area: Rect, wide_min_width: u16) -> SettingsLayout {
    // "Wide" layout is used both for the full Settings tab and for embedded
    // Settings panes (e.g. Operations workspace right column). Keep the
    // threshold low enough that the embedded pane can still use the 2-column
    // layout in a reasonably wide terminal.
    let wide_layout = area.width >= wide_min_width && area.height >= 18;
    if wide_layout {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(7),
                Constraint::Length(1),
            ])
            .split(area);
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(rows[0]);
        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
            .split(rows[1]);
        let sources_alpaca = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(bottom[1]);
        SettingsLayout {
            health: top[0],
            config: top[1],
            symbols: bottom[0],
            sources: sources_alpaca[0],
            alpaca: sources_alpaca[1],
            hint: rows[2],
        }
    } else {
        // Responsive stacked layout: keep a dedicated hint row, then allocate
        // a compact but non-zero health header. Short terminals should still
        // show all panels without forcing the main sections to collapse to 0.
        let hint_h = 1u16;
        let mut health_h = 9u16;
        if area.height < 18 {
            health_h = 7;
        }
        if area.height < 14 {
            health_h = 5;
        }
        if area.height < 11 {
            health_h = 3;
        }
        // Ensure we always leave at least 1 row for the mid stack when possible.
        let max_health = area.height.saturating_sub(hint_h).saturating_sub(1);
        health_h = health_h.min(max_health.max(1));

        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(health_h),
                Constraint::Min(0),
                Constraint::Length(hint_h),
            ])
            .split(area);
        let mid_heights = settings_stacked_mid_heights(outer[1].height);
        let mid = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(mid_heights[0]),
                Constraint::Length(mid_heights[1]),
                Constraint::Length(mid_heights[2]),
                Constraint::Length(mid_heights[3]),
            ])
            .split(outer[1]);
        SettingsLayout {
            health: outer[0],
            config: mid[0],
            symbols: mid[1],
            sources: mid[2],
            alpaca: mid[3],
            hint: outer[2],
        }
    }
}

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let layout = settings_layout(area);
    render_settings_sections(f, app, layout);
    render_settings_credential_modal_if_any(f, app, area);
}

fn settings_modal_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let w = (r.width * percent_x) / 100;
    let h = (r.height * percent_y) / 100;
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

/// Centered overlay for any API key edit (Alpaca rows or Data sources rows).
fn render_settings_credential_modal_if_any(f: &mut Frame, app: &App, area: Rect) {
    if app.settings_credential_buffer.is_none() {
        return;
    }
    let key = app
        .settings_credential_edit_key
        .map(|k| k.display_name())
        .unwrap_or("API key");
    let buf = app.settings_credential_buffer.as_deref().unwrap_or("");
    let modal_area = settings_modal_rect(85, 40, area);
    let edit = Paragraph::new(vec![
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
    .block(
        Block::default()
            .title(format!(" {key} — edit "))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(Clear, modal_area);
    f.render_widget(edit, modal_area);
}

pub(super) fn section_active(app: &App, section: SettingsSection) -> bool {
    app.settings_section == section
}

pub(super) fn section_block(title: impl Into<String>, active: bool) -> Block<'static> {
    let title = title.into();
    let border_style = if active {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = if active {
        format!(" ▶ {} ", title)
    } else {
        format!(" {} ", title)
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    super::text_trunc::truncate_chars(s, max)
}

#[cfg(test)]
mod settings_layout_tests {
    use super::settings_stacked_mid_heights;

    #[test]
    fn stacked_mid_heights_sum_to_total() {
        for t in 0u16..=48 {
            let h = settings_stacked_mid_heights(t);
            let sum: u16 = h.iter().sum();
            assert_eq!(sum, t, "t={t} -> {h:?}");
        }
    }
}
