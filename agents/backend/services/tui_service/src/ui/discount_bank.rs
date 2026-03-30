//! Discount Bank tab: balance summary + scrollable transactions from NATS.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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

pub fn render_discount_bank(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Discount Bank (api.discount_bank.*) ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Hint bar anchored to bottom of inner area
    let render_hint = |f: &mut Frame| {
        if inner.height > 1 {
            let hint_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " \u{2191}\u{2193} scroll  r = refresh ",
                    Style::default().fg(Color::DarkGray),
                ))),
                hint_area,
            );
        }
    };

    // Loading state
    if app.discount_bank_fetch_pending
        && app.discount_bank_balance.is_none()
        && app.discount_bank_transactions.is_none()
    {
        let p = Paragraph::new("Requesting api.discount_bank.balance and transactions\u{2026}")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        render_hint(f);
        return;
    }

    // No data yet
    if app.discount_bank_balance.is_none() && app.discount_bank_transactions.is_none() {
        let p = Paragraph::new("No data. Press 'r' to refresh.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, inner);
        render_hint(f);
        return;
    }

    // Split inner area: top = balance summary (5 lines), bottom = transactions table
    let summary_height = 5u16.min(inner.height.saturating_sub(3));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(summary_height),
            Constraint::Min(0),
            Constraint::Length(1), // hint bar
        ])
        .split(inner);

    render_balance_summary(f, app, chunks[0]);
    render_transactions_table(f, app, chunks[1]);

    // Hint bar
    let hint = Line::from(Span::styled(
        " \u{2191}\u{2193} scroll  r = refresh ",
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(Paragraph::new(hint), chunks[2]);
}

fn render_balance_summary(f: &mut Frame, app: &App, area: Rect) {
    match &app.discount_bank_balance {
        None => {
            let p = Paragraph::new("Balance: —").style(Style::default().fg(Color::DarkGray));
            f.render_widget(p, area);
        }
        Some(Err(e)) => {
            let lines = vec![
                Line::from(Span::styled(
                    "Balance fetch error:",
                    Style::default().fg(Color::Red),
                )),
                Line::from(Span::styled(e.as_str(), Style::default().fg(Color::Red))),
            ];
            f.render_widget(Paragraph::new(lines), area);
        }
        Some(Ok(bal)) => {
            let acct = &bal.account;
            let lines = vec![
                Line::from(vec![
                    Span::styled("Account: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!(
                            "{} ({}-{}-{})",
                            acct.id, acct.branch_number, acct.section_number, acct.account_number
                        ),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Balance: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} {}", bal.balance, bal.currency),
                        Style::default().fg(if bal.balance >= 0.0 {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                    ),
                    Span::styled(
                        format!("  (as of {})", bal.balance_date),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Credit rate: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2}%", bal.credit_rate * 100.0),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled("   Debit rate: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2}%", bal.debit_rate * 100.0),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
            ];
            f.render_widget(Paragraph::new(lines), area);
        }
    }
}

fn render_transactions_table(f: &mut Frame, app: &App, area: Rect) {
    match &app.discount_bank_transactions {
        None => {
            let p = Paragraph::new("Transactions: —").style(Style::default().fg(Color::DarkGray));
            f.render_widget(p, area);
        }
        Some(Err(e)) => {
            let lines = vec![
                Line::from(Span::styled(
                    "Transactions fetch error:",
                    Style::default().fg(Color::Red),
                )),
                Line::from(Span::styled(e.as_str(), Style::default().fg(Color::Red))),
            ];
            f.render_widget(Paragraph::new(lines), area);
        }
        Some(Ok(list)) => {
            if list.transactions.is_empty() {
                let p = Paragraph::new("No transactions returned.")
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(p, area);
                return;
            }

            let header = Row::new([
                Cell::from("Date").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Amount").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Reference").style(Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let all_rows: Vec<Row> = list
                .transactions
                .iter()
                .map(|tx| {
                    let type_label = if tx.is_debit { "Debit" } else { "Credit" };
                    let type_color = if tx.is_debit {
                        Color::Red
                    } else {
                        Color::Green
                    };
                    Row::new([
                        Cell::from(truncate(&tx.value_date, 10)),
                        Cell::from(format!("{:.2}", tx.amount)).style(Style::default().fg(
                            if tx.is_debit {
                                Color::Red
                            } else {
                                Color::Green
                            },
                        )),
                        Cell::from(type_label).style(Style::default().fg(type_color)),
                        Cell::from(truncate(&tx.reference, 40)),
                    ])
                })
                .collect();

            let len = all_rows.len();
            let visible_height = (area.height as usize).saturating_sub(2).max(1);
            let scroll = if len <= 1 {
                0
            } else {
                app.discount_bank_table
                    .selected()
                    .min(len.saturating_sub(1))
            };
            let window: Vec<Row> = all_rows
                .into_iter()
                .skip(scroll)
                .take(visible_height)
                .collect();

            let table = Table::new(
                window,
                [
                    Constraint::Length(10),
                    Constraint::Length(14),
                    Constraint::Length(7),
                    Constraint::Min(20),
                ],
            )
            .header(header)
            .row_highlight_style(Style::default().fg(Color::Yellow));

            f.render_widget(table, area);
        }
    }
}
