//! Loans tab: list from NATS api.loans.list.

use api::loans::{LoanStatus, LoanType};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

fn status_label(s: &LoanStatus) -> &'static str {
    match s {
        LoanStatus::Active => "Active",
        LoanStatus::PaidOff => "Paid off",
        LoanStatus::Defaulted => "Defaulted",
    }
}

fn type_label(t: &LoanType) -> &'static str {
    match t {
        LoanType::ShirBased => "SHIR",
        LoanType::CpiLinked => "CPI",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

pub fn render_loans(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Loans (api.loans.list) ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.loans_fetch_pending && app.loans_list.is_none() {
        let p = Paragraph::new("Requesting api.loans.list…")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        return;
    }

    if let None = app.loans_list {
        let p = Paragraph::new("No data yet. Switch away and back or wait for auto-refresh.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        return;
    }

    if let Some(Err(e)) = &app.loans_list {
        let p = Paragraph::new(e.as_str()).style(Style::default().fg(Color::Red));
        f.render_widget(p, inner);
        return;
    }

    let list = match &app.loans_list {
        Some(Ok(l)) => l,
        _ => return,
    };

    if list.is_empty() {
        let p = Paragraph::new("No loans returned.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        return;
    }

    let header = Row::new([
        Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Bank").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Principal").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Rate %").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Maturity").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);
    let all_rows: Vec<Row> = list
        .iter()
        .map(|l| {
            Row::new([
                Cell::from(truncate(&l.loan_id, 12)),
                Cell::from(truncate(&l.bank_name, 14)),
                Cell::from(type_label(&l.loan_type)),
                Cell::from(format!("{:.0}", l.principal)),
                Cell::from(format!("{:.2}", l.interest_rate)),
                Cell::from(status_label(&l.status)),
                Cell::from(truncate(&l.maturity_date, 10)),
            ])
        })
        .collect();
    let len = all_rows.len();
    let visible_height = (inner.height as usize).saturating_sub(2).max(1);
    let scroll = if len <= 1 {
        0
    } else {
        app.loans_scroll.min(len.saturating_sub(1))
    };
    let window: Vec<Row> = all_rows
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();
    let table = Table::new(
        window,
        [
            ratatui::layout::Constraint::Length(12),
            ratatui::layout::Constraint::Length(14),
            ratatui::layout::Constraint::Length(5),
            ratatui::layout::Constraint::Length(10),
            ratatui::layout::Constraint::Length(7),
            ratatui::layout::Constraint::Length(10),
            ratatui::layout::Constraint::Length(12),
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(table, inner);
}
