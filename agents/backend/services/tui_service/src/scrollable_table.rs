//! Scrollable table selection and viewport offset (`docs/TUI_ARCHITECTURE.md` Phase 2).
//!
//! Most tabs use [`ScrollableTableState::selected`] as the highlighted row index; the optional
//! [`ScrollableTableState::scroll`] field is the first visible row when implementing a viewport.
//! Alerts use [`ScrollableTableState::shift_scroll`] on `scroll` only (paragraph-style list).
//!
//! # Single owner on [`crate::app::App`]
//!
//! Each pane keeps **one** [`ScrollableTableState`] on [`crate::app::App`] (for example
//! `positions_table`, `orders_table`, `dashboard_table`). That struct is the source of truth for
//! both cursor and viewport offset for that pane. **Do not** duplicate `selected` / `scroll` in
//! render-only or input-local state; read and update the `App` field (or pass a `&mut` to it) so
//! navigation, mouse, and draw agree on the same indices.
//!
//! # [`ScrollableTableState::clamp_to_len`] vs [`ScrollableTableState::adjust_scroll`]
//!
//! - **`clamp_to_len(len)`** — Run when the **logical row count** changes: new snapshot, filter
//!   tightening, combo expand/collapse, or any event that changes how many rows exist for the
//!   pane. It clamps `selected` to `0..len` (or zeros both when `len == 0`) and then ensures
//!   `scroll <= selected` so the viewport anchor stays consistent. It does **not** grow the
//!   visible window; it only repairs indices after the list shrinks or is replaced.
//!
//! - **`adjust_scroll(visible_height)`** — Run when the **viewport height in rows** changes
//!   (terminal resize → recompute `visible_height` from layout) or after moving `selected` while
//!   using a scrolling window. It only moves `scroll` so `selected` lies inside the visible range
//!   `[scroll, scroll + visible_height)` (no-op if `visible_height == 0`). It assumes indices
//!   are already valid for the current list length; if the list might have shrunk, call
//!   `clamp_to_len` **first**, then `adjust_scroll`.
//!
//! **Invariant:** after the underlying row list shrinks or is replaced, call `clamp_to_len` so
//! `selected` / `scroll` stay valid before the next render or key handler.
//!
//! # Resize behavior
//!
//! On terminal resize, layout code typically produces a new `visible_height`. If the row count
//! for that pane may have changed in the same tick (e.g. snapshot update), apply **`clamp_to_len`
//! then `adjust_scroll`**. If only geometry changed, **`adjust_scroll` alone** is enough. Order
//! matters: clamping after adjusting scroll can leave a stale `selected` past the new `len`.

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
