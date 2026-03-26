//! Mouse input handling for TUI.
//!
//! Provides mouse interaction support including:
//! - Clicking on tabs to switch
//! - Scrolling with mouse wheel
//! - Clicking on table rows to select
//! - Dragging for scrollable areas

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::app::App;
use crate::input::Action;

/// Handle a mouse event and return an action if applicable.
pub fn handle_mouse_event(app: &App, mouse: MouseEvent, area: Rect) -> Option<Action> {
    let x = mouse.column;
    let y = mouse.row;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => handle_mouse_click(app, x, y, area),
        MouseEventKind::ScrollUp => Some(Action::MouseScrollUp),
        MouseEventKind::ScrollDown => Some(Action::MouseScrollDown),
        _ => None,
    }
}

/// Handle a mouse click at the given coordinates.
fn handle_mouse_click(app: &App, x: u16, y: u16, area: Rect) -> Option<Action> {
    // Check if click is within the main area
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    // Check if click is on tab bar (typically at y=1 or y=2)
    if y <= area.y + 1 {
        return handle_tab_click(app, x, area);
    }

    // Check if click is in main content area
    if y > area.y + 2 && y < area.y + area.height - 2 {
        return handle_content_click(app, x, y, area);
    }

    None
}

/// Handle a click on the tab bar.
fn handle_tab_click(app: &App, x: u16, area: Rect) -> Option<Action> {
    // Tab bar layout estimation:
    // Each tab is roughly "[N] Name  " format
    // Dashboard Positions Charts Orders Alerts Yield Loans Scenarios Settings

    let tabs = vec![
        ("Dashboard", 0u16, 10u16),
        ("Positions", 11, 10),
        ("Charts", 22, 7),
        ("Orders", 30, 7),
        ("Alerts", 38, 7),
        ("Yield", 46, 6),
        ("Loans", 53, 6),
        ("Scenarios", 60, 10),
        ("Settings", 71, 9),
    ];

    let rel_x = x.saturating_sub(area.x);

    for (idx, (_, start, width)) in tabs.iter().enumerate() {
        if rel_x >= *start && rel_x < *start + *width {
            return Some(Action::JumpToTab((idx + 1) as u8));
        }
    }

    None
}

/// Handle a click in the main content area.
fn handle_content_click(app: &App, x: u16, y: u16, area: Rect) -> Option<Action> {
    use crate::app::Tab;

    match app.active_tab {
        Tab::Positions => Some(Action::PositionsDetail),
        Tab::Orders => Some(Action::OrdersDetail),
        Tab::Dashboard => Some(Action::DashboardNavigateToChart),
        Tab::Scenarios => Some(Action::ScenariosDetail),
        _ => None,
    }
}

/// Enable mouse capture in the terminal.
pub fn enable_mouse_capture() -> std::io::Result<()> {
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)
}

/// Disable mouse capture in the terminal.
pub fn disable_mouse_capture() -> std::io::Result<()> {
    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)
}
