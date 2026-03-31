//! Ledger tab: read-only ledger journal (transactions table) via NATS.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", t)
    }
}

pub(crate) fn render_ledger(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Ledger journal (api.ledger.journal) ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let render_hint = |f: &mut Frame| {
        if inner.height > 1 {
            let hint_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " ↑↓ PgUp/PgDn scroll  r = refresh ",
                    Style::default().fg(Color::DarkGray),
                ))),
                hint_area,
            );
        }
    };

    if app.ledger_fetch_pending && app.ledger_journal.is_none() {
        let p = Paragraph::new("Requesting api.ledger.journal…")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        render_hint(f);
        return;
    }

    if app.ledger_journal.is_none() {
        let p = Paragraph::new("No ledger data. Press 'r' to refresh.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        render_hint(f);
        return;
    }

    match &app.ledger_journal {
        Some(Err(e)) => {
            let lines = vec![
                Line::from(Span::styled(
                    "Ledger fetch error:",
                    Style::default().fg(Color::Red),
                )),
                Line::from(Span::styled(e.as_str(), Style::default().fg(Color::Red))),
            ];
            f.render_widget(Paragraph::new(lines), inner);
            render_hint(f);
        }
        Some(Ok(journal)) => {
            if journal.entries.is_empty() {
                let p = Paragraph::new("No transactions returned.")
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(p, inner);
                render_hint(f);
                return;
            }

            let header = Row::new([
                Cell::from("Date").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("✓").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Description").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Accounts").style(Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let all_rows: Vec<Row> = journal
                .entries
                .iter()
                .map(|e| {
                    let cleared = if e.cleared { "Y" } else { "" };
                    Row::new([
                        Cell::from(truncate(&e.date, 10)),
                        Cell::from(cleared).style(Style::default().fg(Color::Green)),
                        Cell::from(truncate(&e.description, 44)),
                        Cell::from(truncate(&e.account_paths, 44)).style(
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ])
                })
                .collect();

            let len = all_rows.len();
            let visible_height = (inner.height as usize).saturating_sub(2).max(1);
            let scroll = if len <= 1 {
                0
            } else {
                app.ledger_table.selected().min(len.saturating_sub(1))
            };
            let window: Vec<Row> = all_rows
                .into_iter()
                .skip(scroll)
                .take(visible_height)
                .collect();

            let table = Table::new(
                window,
                [
                    ratatui::layout::Constraint::Length(10),
                    ratatui::layout::Constraint::Length(2),
                    ratatui::layout::Constraint::Min(30),
                    ratatui::layout::Constraint::Min(24),
                ],
            )
            .header(header)
            .row_highlight_style(Style::default().fg(Color::Yellow));

            f.render_widget(table, inner);
            render_hint(f);
        }
        None => {}
    }
}

