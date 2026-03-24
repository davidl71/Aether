//! Alerts tab: scrollable alerts list.

use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use api::AlertLevel;

use ratatui::layout::Rect;

use crate::app::App;

pub fn render_alerts(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = if let Some(ref snap) = app.snapshot() {
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
    };

    let len = lines.len();
    let visible_height = area.height.saturating_sub(2) as usize; // inner height minus borders

    // Only scroll if content exceeds available height; otherwise show all lines from top.
    let scroll_row = if len <= visible_height {
        0
    } else {
        app.alerts_scroll.min(len.saturating_sub(1))
    };

    let title = if len > visible_height {
        format!("Alerts ({}/{})  [↑↓ PgUp/PgDn]:scroll", scroll_row + 1, len)
    } else {
        format!("Alerts ({})", len)
    };

    let widget = Paragraph::new(lines).scroll((scroll_row as u16, 0)).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL),
    );

    f.render_widget(widget, area);
}
