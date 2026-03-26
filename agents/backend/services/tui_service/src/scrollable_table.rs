//! Scrollable table state for large tables.
//!
//! Manages scroll position and selection for tables with many rows,
//! allowing efficient navigation without rendering all rows.

/// State for a scrollable table with selection.
#[derive(Debug, Clone)]
pub struct ScrollableTableState {
    /// First visible row offset
    pub offset: usize,
    /// Currently selected row index (absolute, not relative to offset)
    pub selected: usize,
    /// Total number of rows in the table
    pub total_rows: usize,
    /// Number of rows visible at once
    pub visible_rows: usize,
}

impl Default for ScrollableTableState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollableTableState {
    /// Create a new scrollable table state.
    pub fn new() -> Self {
        Self {
            offset: 0,
            selected: 0,
            total_rows: 0,
            visible_rows: 10,
        }
    }

    /// Create with total rows and visible rows.
    pub fn with_rows(total_rows: usize, visible_rows: usize) -> Self {
        Self {
            offset: 0,
            selected: 0,
            total_rows,
            visible_rows,
        }
    }

    /// Update total rows (e.g., when data changes).
    pub fn set_total_rows(&mut self, total: usize) {
        self.total_rows = total;
        self.clamp_selection();
    }

    /// Update visible rows (e.g., when terminal resizes).
    pub fn set_visible_rows(&mut self, visible: usize) {
        self.visible_rows = visible;
        self.adjust_offset();
    }

    /// Move selection up by one row.
    pub fn scroll_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.adjust_offset();
    }

    /// Move selection down by one row.
    pub fn scroll_down(&mut self) {
        if self.total_rows > 0 {
            self.selected = (self.selected + 1).min(self.total_rows - 1);
            self.adjust_offset();
        }
    }

    /// Move selection up by page size.
    pub fn page_up(&mut self) {
        self.selected = self.selected.saturating_sub(self.visible_rows);
        self.adjust_offset();
    }

    /// Move selection down by page size.
    pub fn page_down(&mut self) {
        if self.total_rows > 0 {
            self.selected = (self.selected + self.visible_rows).min(self.total_rows - 1);
            self.adjust_offset();
        }
    }

    /// Move to first row.
    pub fn go_to_top(&mut self) {
        self.selected = 0;
        self.adjust_offset();
    }

    /// Move to last row.
    pub fn go_to_bottom(&mut self) {
        if self.total_rows > 0 {
            self.selected = self.total_rows - 1;
            self.adjust_offset();
        }
    }

    /// Get the visible range (start, end) for rendering.
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.visible_rows).min(self.total_rows);
        (start, end)
    }

    /// Check if a row index is currently visible.
    pub fn is_visible(&self, index: usize) -> bool {
        let (start, end) = self.visible_range();
        index >= start && index < end
    }

    /// Get relative index for rendering (0 to visible_rows-1).
    pub fn relative_index(&self, absolute: usize) -> Option<usize> {
        if self.is_visible(absolute) {
            Some(absolute - self.offset)
        } else {
            None
        }
    }

    /// Clamp selection to valid bounds.
    fn clamp_selection(&mut self) {
        if self.total_rows == 0 {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.total_rows - 1);
        }
    }

    /// Adjust offset to ensure selection is visible.
    fn adjust_offset(&mut self) {
        // Ensure offset doesn't go past selected
        if self.offset > self.selected {
            self.offset = self.selected;
        }
        // Ensure selection is visible
        if self.selected >= self.offset + self.visible_rows {
            self.offset = self.selected.saturating_sub(self.visible_rows - 1);
        }
        // Clamp offset to valid range
        if self.total_rows > self.visible_rows {
            let max_offset = self.total_rows - self.visible_rows;
            self.offset = self.offset.min(max_offset);
        } else {
            self.offset = 0;
        }
    }
}

/// Helper to render a scrollable table with selection highlighting.
pub struct ScrollableTable<'a> {
    rows: Vec<Vec<String>>,
    headers: Vec<String>,
    state: &'a ScrollableTableState,
}

impl<'a> ScrollableTable<'a> {
    /// Create a new scrollable table.
    pub fn new(
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        state: &'a ScrollableTableState,
    ) -> Self {
        Self {
            rows,
            headers,
            state,
        }
    }

    /// Get visible rows with their absolute indices.
    pub fn visible_rows(&self) -> impl Iterator<Item = (usize, &Vec<String>)> {
        let (start, end) = self.state.visible_range();
        self.rows.iter().enumerate().skip(start).take(end - start)
    }

    /// Get header row.
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Check if row is selected.
    pub fn is_selected(&self, absolute_index: usize) -> bool {
        self.state.selected == absolute_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollable_table_state() {
        let mut state = ScrollableTableState::with_rows(100, 10);
        assert_eq!(state.visible_range(), (0, 10));

        state.scroll_down();
        assert_eq!(state.selected, 1);
        assert_eq!(state.offset, 0);

        // Scroll to bottom of visible area
        for _ in 0..10 {
            state.scroll_down();
        }
        assert_eq!(state.selected, 11);
        assert_eq!(state.offset, 2); // Adjusted to keep selection visible
    }

    #[test]
    fn test_page_navigation() {
        let mut state = ScrollableTableState::with_rows(100, 10);

        state.page_down();
        assert_eq!(state.selected, 10);
        assert_eq!(state.offset, 10);

        state.page_up();
        assert_eq!(state.selected, 0);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn test_go_to_top_bottom() {
        let mut state = ScrollableTableState::with_rows(50, 10);

        state.go_to_bottom();
        assert_eq!(state.selected, 49);

        state.go_to_top();
        assert_eq!(state.selected, 0);
    }
}
