//! Tree panel overlay: tui-tree-widget spike for hierarchy navigation.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem};

use crate::app::App;

pub(crate) type TreeId = &'static str;

pub(crate) fn build_spike_items() -> Vec<TreeItem<'static, TreeId>> {
    let leaf_a = TreeItem::new_leaf("leaf_a", Text::from("Leaf A"));
    let leaf_b = TreeItem::new_leaf("leaf_b", Text::from("Leaf B"));

    let child_1 = TreeItem::new("child_1", Text::from("Child 1"), vec![leaf_a, leaf_b])
        .expect("child identifiers unique");

    let child_2 = TreeItem::new_leaf("child_2", Text::from("Child 2"));

    vec![
        TreeItem::new("root", Text::from("Root"), vec![child_1, child_2])
            .expect("root identifiers unique"),
        TreeItem::new_leaf("solo", Text::from("Solo")),
    ]
}

pub(crate) fn ensure_initialized(app: &mut App) {
    if !app.tree_items.is_empty() {
        return;
    }

    app.tree_items = build_spike_items();
    app.tree_state.borrow_mut().select_first();
}

pub(crate) fn render_tree_panel_overlay(f: &mut Frame, app: &App, area: Rect) {
    if app.tree_items.is_empty() {
        return;
    }

    let area = centered_rect(70, 70, area);
    f.render_widget(Clear, area);

    let outer = Block::default()
        .title(Line::from(vec![
            Span::styled(
                " Tree (Spike) ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  [↑↓ ←→ Enter]:navigate  [Esc]:close"),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let tree = Tree::new(&app.tree_items)
        .expect("all sibling identifiers are unique")
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(tree, rows[0], &mut *app.tree_state.borrow_mut());

    let hint = Paragraph::new("Spike data only. Next: map accounts/strategies into TreeItem.")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, rows[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
