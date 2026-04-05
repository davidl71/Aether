//! Scrollable table selection and viewport offset (`docs/TUI_ARCHITECTURE.md` Phase 2).
//!
//! Most tabs use [`ScrollableTableState::selected`] as the highlighted row index; the optional
//! [`ScrollableTableState::scroll`] field is the first visible row when implementing a viewport.
//! Alerts use [`ScrollableTableState::shift_scroll`] on `scroll` only (paragraph-style list).
//!
//! **Invariant:** after the underlying row list shrinks or is replaced, call [`ScrollableTableState::clamp_to_len`]
//! so `selected` / `scroll` stay valid before the next render or key handler.

/// Clamp a possibly-out-of-range selected index to a list length.
///
/// Returns 0 when `len == 0`, otherwise `min(index, len - 1)`.
pub fn clamp_index(index: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        index.min(len - 1)
    }
}

/// Compute a viewport start offset that keeps `cursor` roughly centered.
///
/// `cursor` should already be clamped to `len`.
pub fn centered_viewport_start(cursor: usize, len: usize, visible_height: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let visible = visible_height.max(1).min(len);
    if len <= visible {
        0
    } else {
        cursor
            .saturating_sub(visible / 2)
            .min(len.saturating_sub(visible))
    }
}

/// Selection index and optional viewport offset for tabular / list panes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScrollableTableState {
    selected: usize,
    scroll: usize,
}

impl ScrollableTableState {
    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Move selection down; `max` is row count (valid indices are `0..max`).
    pub fn move_down(&mut self, max: usize) {
        if max == 0 {
            self.selected = 0;
        } else {
            self.selected = (self.selected + 1).min(max - 1);
        }
    }

    /// Keep [`Self::selected`] within the visible window starting at [`Self::scroll`].
    pub fn adjust_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
        let vis = visible_height.max(1);
        let last_visible = self.scroll.saturating_add(vis.saturating_sub(1));
        if self.selected > last_visible {
            self.scroll = self.selected.saturating_sub(vis.saturating_sub(1));
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Reset selection and viewport (e.g. toggle combo view, change symbol).
    pub fn reset(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }

    /// Clamp when row count shrinks; keeps `scroll` consistent with `selected`.
    pub fn clamp_to_len(&mut self, len: usize) {
        if len == 0 {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        self.selected = self.selected.min(len - 1);
        self.scroll = self.scroll.min(self.selected);
    }

    /// Move selection by `delta` rows, clamped to `[0, max - 1]` when `max > 0`.
    pub fn shift_selected(&mut self, delta: isize, max: usize) {
        if max == 0 {
            self.selected = 0;
            return;
        }
        let max_i = (max - 1) as isize;
        let s = self.selected as isize + delta;
        self.selected = s.clamp(0, max_i) as usize;
    }

    /// Move viewport offset only (alerts paragraph scroll). Clamps to `[0, max_offset]`.
    pub fn shift_scroll(&mut self, delta: isize, max_offset: usize) {
        let s = self.scroll as isize + delta;
        self.scroll = s.clamp(0, max_offset as isize) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_up_down_respects_max() {
        let mut s = ScrollableTableState::default();
        s.move_down(3);
        assert_eq!(s.selected(), 1);
        s.move_down(3);
        assert_eq!(s.selected(), 2);
        s.move_down(3);
        assert_eq!(s.selected(), 2);
        s.move_up();
        assert_eq!(s.selected(), 1);
        s.move_up();
        s.move_up();
        assert_eq!(s.selected(), 0);
    }

    #[test]
    fn move_down_zero_max_zeros_selection() {
        let mut s = ScrollableTableState::default();
        s.shift_selected(5, 10);
        s.move_down(0);
        assert_eq!(s.selected(), 0);
    }

    #[test]
    fn adjust_scroll_keeps_selection_visible() {
        let mut s = ScrollableTableState::default();
        s.shift_selected(15, 20);
        s.adjust_scroll(5);
        assert_eq!(s.scroll(), 11);
        s.shift_selected(-10, 20);
        s.adjust_scroll(5);
        assert_eq!(s.selected(), 5);
        assert_eq!(s.scroll(), 5);
    }

    #[test]
    fn shift_selected_clamps() {
        let mut s = ScrollableTableState::default();
        s.shift_selected(50, 10);
        assert_eq!(s.selected(), 9);
        s.shift_selected(-100, 10);
        assert_eq!(s.selected(), 0);
    }

    #[test]
    fn shift_scroll_clamps() {
        let mut s = ScrollableTableState::default();
        s.shift_scroll(5, 3);
        assert_eq!(s.scroll(), 3);
        s.shift_scroll(-10, 3);
        assert_eq!(s.scroll(), 0);
    }
}
