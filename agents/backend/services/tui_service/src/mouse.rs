//! Mouse input handling for TUI.
//!
//! Provides mouse interaction support including:
//! - Clicking on tabs to switch
//! - Scrolling with mouse wheel
//! - Clicking on table rows to select
//! - Dragging for scrollable areas

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::App;
use crate::input::Action;
use crate::workspace::VisibleWorkspace;

/// Handle a mouse event and return an action if applicable.
pub fn handle_mouse_event(app: &App, mouse: MouseEvent, area: Rect) -> Option<Action> {
    let x = mouse.column;
    let y = mouse.row;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => handle_mouse_click(app, x, y, area),
        MouseEventKind::ScrollUp => mouse_scroll_action(app, x, y, area, true),
        MouseEventKind::ScrollDown => mouse_scroll_action(app, x, y, area, false),
        _ => None,
    }
}

fn mouse_scroll_action(app: &App, x: u16, y: u16, area: Rect, up: bool) -> Option<Action> {
    // Route mouse wheel to the pane under the cursor when in a workspace layout.
    // This matters most in the Market workspace where 4 panes are visible at once.
    if app.visible_workspace() == VisibleWorkspace::Market {
        if let Some(tab) = market_workspace_tab_at_point(app, x, y, area) {
            return Some(if up {
                Action::MouseScrollUpIn(tab)
            } else {
                Action::MouseScrollDownIn(tab)
            });
        }
    }
    Some(if up {
        Action::MouseScrollUp
    } else {
        Action::MouseScrollDown
    })
}

fn market_workspace_tab_at_point(app: &App, x: u16, y: u16, area: Rect) -> Option<crate::app::Tab> {
    // Mirror the high-level layout in `ui::render` → `render_main` → `render_market_workspace`.
    //
    // Vertical shell: [tab bar=3, main, hint=1, status=1]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);
    let main = chunks.get(1).copied()?;

    // Market workspace adds a 1-row banner inside main.
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(main);
    let body = outer.get(1).copied()?;

    // 2x2 grid with asymmetric splits.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(rows[1]);

    let p = (x, y);
    if contains_point(top[0], p) {
        return Some(crate::app::Tab::Dashboard);
    }
    if contains_point(top[1], p) {
        return Some(crate::app::Tab::Positions);
    }
    if contains_point(bottom[0], p) {
        return Some(crate::app::Tab::Orders);
    }
    if contains_point(bottom[1], p) {
        return Some(crate::app::Tab::Yield);
    }

    // If the mouse is within main but not any pane (borders/gaps), fall back.
    let _ = app; // keep signature stable for future workspace-specific rules
    None
}

fn contains_point(r: Rect, (x, y): (u16, u16)) -> bool {
    x >= r.x && x < r.x.saturating_add(r.width) && y >= r.y && y < r.y.saturating_add(r.height)
}

/// Handle a mouse click at the given coordinates.
fn handle_mouse_click(app: &App, x: u16, y: u16, area: Rect) -> Option<Action> {
    // Check if click is within the main area
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    // Check if click is on tab bar (typically at y=1 or y=2)
    if y <= area.y + 1 {
        return handle_tab_click(app, x, y);
    }

    // Check if click is in main content area
    if y > area.y + 2 && y < area.y + area.height - 2 {
        return handle_content_click(app, x, y, area);
    }

    None
}

/// Handle a click on the tab bar.
fn handle_tab_click(app: &App, x: u16, y: u16) -> Option<Action> {
    // Route via regions recorded during the most recent render pass.
    // See `ui::render_tab_bar` for region computation.
    let p = (x, y);
    for (tab, rect) in app.tab_bar_regions.borrow().iter() {
        if contains_point(*rect, p) {
            let idx = crate::app::Tab::ALL.iter().position(|t| t == tab)?;
            return Some(Action::JumpToTab((idx + 1) as u8));
        }
    }
    None
}

/// Handle a click in the main content area.
fn handle_content_click(app: &App, x: u16, y: u16, area: Rect) -> Option<Action> {
    use crate::app::Tab;

    let (_x, _y, _area) = (x, y, area);
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
