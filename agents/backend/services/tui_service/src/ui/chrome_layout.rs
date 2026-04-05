//! Fixed vertical chrome regions (tab bar, main, hint, status).
//!
//! See `docs/TUI_PANE_MODEL.md` — keep heights in sync with `render` expectations.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Tab bar height (includes borders/padding as allocated today).
pub const TAB_BAR_HEIGHT: u16 = 3;
/// Bottom hint bar (single row).
pub const HINT_BAR_HEIGHT: u16 = 1;
/// Status bar (single row).
pub const STATUS_BAR_HEIGHT: u16 = 1;

/// Split `area` into `[tab_bar, main, hint_bar, status_bar]`.
pub fn split_vertical_chrome(area: Rect) -> [Rect; 4] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TAB_BAR_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(HINT_BAR_HEIGHT),
            Constraint::Length(STATUS_BAR_HEIGHT),
        ])
        .split(area);
    [chunks[0], chunks[1], chunks[2], chunks[3]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn split_four_regions_stacked_vertically() {
        let area = Rect::new(0, 0, 100, 30);
        let [tab, main, hint, status] = split_vertical_chrome(area);
        assert_eq!(tab.height, TAB_BAR_HEIGHT);
        assert_eq!(hint.height, HINT_BAR_HEIGHT);
        assert_eq!(status.height, STATUS_BAR_HEIGHT);
        assert_eq!(tab.y + tab.height, main.y);
        assert_eq!(main.y + main.height, hint.y);
        assert_eq!(hint.y + hint.height, status.y);
        assert_eq!(status.y + status.height, area.y + area.height);
    }
}
