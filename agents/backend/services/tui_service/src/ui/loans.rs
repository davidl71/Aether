//! Loans tab: list from NATS api.loans.list.

use api::loans::{LoanStatus, LoanType};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, LoanEntryState, LoanType as AppLoanType};

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
    // If loan entry form is open, render it instead of the list
    if let Some(ref entry) = app.loan_entry {
        render_loan_entry_form(f, app, area, entry);
        return;
    }

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
        let p = Paragraph::new("No loans returned. Press 'n' to add a new loan.")
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
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(table, inner);

    // Hint line
    let hint = Line::from(Span::styled(
        " ↑↓ scroll  n = new loan  Del = delete  [Enter] = details ",
        Style::default().fg(Color::DarkGray),
    ));
    let hint_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
    f.render_widget(Paragraph::new(hint), hint_area);
}

fn render_loan_entry_form(f: &mut Frame, app: &App, area: Rect, entry: &LoanEntryState) {
    let block = Block::default()
        .title(" New Loan (Enter=submit, Esc=cancel, ↑↓=nav) ")
        .borders(Borders::ALL);
    f.render_widget(block, area);

    let inner = Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 4);

    let fields = [
        ("Bank Name", entry.bank_name.as_str()),
        ("Account #", entry.account_number.as_str()),
        (
            "Type",
            if entry.loan_type == AppLoanType::ShirBased {
                "SHIR"
            } else {
                "CPI"
            },
        ),
        ("Principal", entry.principal.as_str()),
        ("Interest %", entry.interest_rate.as_str()),
        ("Spread %", entry.spread.as_str()),
        ("Start Date", entry.origination_date.as_str()),
        ("First Pmt", entry.first_payment_date.as_str()),
        ("# Payments", entry.num_payments.as_str()),
        ("Monthly $", entry.monthly_payment.as_str()),
        ("Maturity", entry.maturity_date.as_str()),
    ];

    let max_field_len = 40;
    let field_rows: Vec<Line> = fields
        .iter()
        .enumerate()
        .map(|(idx, (label, value))| {
            let is_selected = idx == entry.current_field;
            let prefix = if is_selected { "> " } else { "  " };
            let label_str = format!("{}{}:", prefix, label);
            let value_display = if value.is_empty() { "—" } else { value };
            let value_str = format!(
                " {:width$}",
                value_display,
                width = max_field_len - label_str.len()
            );

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(vec![Span::raw(label_str), Span::styled(value_str, style)])
        })
        .collect();

    f.render_widget(
        Paragraph::new(field_rows).style(Style::default().fg(Color::White)),
        inner,
    );

    // Instructions at bottom
    let instructions = Line::from(Span::styled(
        " Type values  [Tab] toggle SHIR/CPI  [Enter] calculate/submit  [Esc] cancel ",
        Style::default().fg(Color::DarkGray),
    ));
    let instr_area = Rect::new(area.x, area.y + area.height - 3, area.width, 1);
    f.render_widget(Paragraph::new(instructions), instr_area);
}
