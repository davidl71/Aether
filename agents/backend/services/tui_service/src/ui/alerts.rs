//! Alerts tab: scrollable alerts list.

use api::AlertLevel;
use ratatui::{
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub(crate) struct AlertsView {
    pub title: String,
    pub lines: Vec<Line<'static>>,
    pub scroll_row: u16,
}

pub(crate) fn build_alerts_view(app: &App, area: Rect) -> AlertsView {
    let lines = alert_lines(app);
    let len = lines.len();
    let visible_height = area.height.saturating_sub(2) as usize; // inner height minus borders
    let max_scroll_row = len.saturating_sub(visible_height.max(1));

    // Only scroll if content exceeds available height; otherwise show all lines from top.
    let scroll_row = if len <= visible_height {
        0
    } else {
        app.alerts_scroll.min(max_scroll_row)
    } as u16;

    let title = if len > visible_height {
        format!("Alerts ({}/{})  [↑↓ PgUp/PgDn]:scroll", scroll_row + 1, len)
    } else {
        format!("Alerts ({})", len)
    };

    AlertsView {
        title,
        lines,
        scroll_row,
    }
}

pub(crate) fn render_alerts_panel(f: &mut Frame, area: Rect, view: AlertsView) {
    let widget = Paragraph::new(view.lines)
        .scroll((view.scroll_row, 0))
        .block(Block::default().title(view.title).borders(Borders::ALL));

    f.render_widget(widget, area);
}

pub fn render_alerts(f: &mut Frame, app: &App, area: Rect) {
    render_alerts_panel(f, area, build_alerts_view(app, area));
}

fn alert_lines(app: &App) -> Vec<Line<'static>> {
    if let Some(ref snap) = app.snapshot() {
        snap.dto()
            .alerts
            .iter()
            .rev()
            .map(|a| {
                let color = match a.level {
                    AlertLevel::Info => Color::Cyan,
                    AlertLevel::Warning => Color::Yellow,
                    AlertLevel::Error => Color::Red,
                };
                Line::from(Span::styled(
                    format!("[{}] {}", a.timestamp.format("%H:%M:%S"), a.message),
                    ratatui::style::Style::default().fg(color),
                ))
            })
            .collect()
    } else {
        vec![Line::from("No alerts")]
    }
}
