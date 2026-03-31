//! Dirty flags for selective re-rendering optimization.
//!
//! Tracks which UI regions need redrawing so [`crate::ui::render`] can skip
//! widgets whose backing cells are unchanged (ratatui diffs buffers between draws).
//!
//! If no bit is set, [`crate::ui::render`] paints the full frame (tests and paths
//! that only set [`crate::app::App::needs_redraw`]).

/// Flags indicating which UI regions need redrawing.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DirtyFlags {
    /// Tab bar needs redraw (tab switch, active tab change)
    pub tabs: bool,
    /// Main content area needs redraw (data changes, scroll)
    pub content: bool,
    /// Status bar needs redraw (connection status, toasts)
    pub status_bar: bool,
    /// Hint bar needs redraw (command status, shortcuts)
    pub hint_bar: bool,
    /// Overlay needs redraw (help, detail popup, command palette)
    pub overlay: bool,
}

impl DirtyFlags {
    /// Create new dirty flags with all regions marked clean.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark all regions as dirty (force full redraw).
    pub fn mark_all(&mut self) {
        self.tabs = true;
        self.content = true;
        self.status_bar = true;
        self.hint_bar = true;
        self.overlay = true;
    }

    /// Mark all regions as clean after render.
    pub fn clear_all(&mut self) {
        *self = Self::default();
    }

    /// Check if any region is dirty.
    pub fn is_dirty(&self) -> bool {
        self.tabs || self.content || self.status_bar || self.hint_bar || self.overlay
    }

    /// Check if tabs region needs redraw.
    pub fn tabs_dirty(&self) -> bool {
        self.tabs
    }

    /// Check if content region needs redraw.
    pub fn content_dirty(&self) -> bool {
        self.content
    }

    /// Check if status bar needs redraw.
    pub fn status_bar_dirty(&self) -> bool {
        self.status_bar
    }

    /// Check if hint bar needs redraw.
    pub fn hint_bar_dirty(&self) -> bool {
        self.hint_bar
    }

    /// Check if overlay needs redraw.
    pub fn overlay_dirty(&self) -> bool {
        self.overlay
    }

    /// Mark tabs as dirty.
    pub fn mark_tabs(&mut self) {
        self.tabs = true;
    }

    /// Mark content as dirty.
    pub fn mark_content(&mut self) {
        self.content = true;
    }

    /// Mark status bar as dirty.
    pub fn mark_status_bar(&mut self) {
        self.status_bar = true;
    }

    /// Mark hint bar as dirty.
    pub fn mark_hint_bar(&mut self) {
        self.hint_bar = true;
    }

    /// Mark overlay as dirty.
    pub fn mark_overlay(&mut self) {
        self.overlay = true;
    }

    /// Clear tabs flag.
    pub fn clear_tabs(&mut self) {
        self.tabs = false;
    }

    /// Clear content flag.
    pub fn clear_content(&mut self) {
        self.content = false;
    }

    /// Clear status bar flag.
    pub fn clear_status_bar(&mut self) {
        self.status_bar = false;
    }

    /// Clear hint bar flag.
    pub fn clear_hint_bar(&mut self) {
        self.hint_bar = false;
    }

    /// Clear overlay flag.
    pub fn clear_overlay(&mut self) {
        self.overlay = false;
    }
}

/// Macro to mark dirty flags based on action type.
#[macro_export]
macro_rules! mark_dirty_for_action {
    ($flags:expr, TabNext | TabPrev | JumpToTab(_)) => {
        $flags.mark_tabs();
        $flags.mark_content()
    };
    ($flags:expr, PositionsScrollUp | PositionsScrollDown | PositionsScrollPageUp | PositionsScrollPageDown) => {
        $flags.mark_content()
    };
    ($flags:expr, OrdersScrollUp | OrdersScrollDown | OrdersScrollPageUp | OrdersScrollPageDown) => {
        $flags.mark_content()
    };
    ($flags:expr, YieldSymbolPrev | YieldSymbolNext | YieldCurveScrollUp | YieldCurveScrollDown | YieldRefresh) => {
        $flags.mark_content()
    };
    ($flags:expr, LoansScrollUp | LoansScrollDown | LoansScrollPageUp | LoansScrollPageDown | DiscountBankScrollUp | DiscountBankScrollDown | DiscountBankScrollPageUp | DiscountBankScrollPageDown | LedgerScrollUp | LedgerScrollDown | LedgerScrollPageUp | LedgerScrollPageDown | LedgerRefresh | AlertsScrollUp | AlertsScrollDown | AlertsScrollPageUp | AlertsScrollPageDown | DashboardScrollUp | DashboardScrollDown | ScenariosScrollUp | ScenariosScrollDown | ScenariosScrollPageUp | ScenariosScrollPageDown) => {
        $flags.mark_content()
    };
    ($flags:expr, MouseScrollUp | MouseScrollDown | MouseScrollUpIn(_) | MouseScrollDownIn(_)) => {
        $flags.mark_content()
    };
    ($flags:expr, PositionsToggleCombo | PositionsCycleSort | OrdersFilterFocus | OrdersFilterChar(_) | OrdersFilterBackspace | OrdersFilterClear) => {
        $flags.mark_content();
        $flags.mark_hint_bar()
    };
    ($flags:expr, ToggleLogPanel | ToggleTreePanel) => {
        $flags.mark_overlay()
    };
    ($flags:expr, ShowHelp | DetailPopup | CommandPalette) => {
        $flags.mark_overlay()
    };
    ($flags:expr, $_:expr) => {
        // Default: mark everything dirty for unknown actions
        $flags.mark_all()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_flags_new() {
        let flags = DirtyFlags::new();
        assert!(!flags.is_dirty());
    }

    #[test]
    fn test_dirty_flags_mark_all() {
        let mut flags = DirtyFlags::new();
        flags.mark_all();
        assert!(flags.is_dirty());
        assert!(flags.tabs_dirty());
        assert!(flags.content_dirty());
        assert!(flags.status_bar_dirty());
    }

    #[test]
    fn test_dirty_flags_clear_all() {
        let mut flags = DirtyFlags::new();
        flags.mark_all();
        flags.clear_all();
        assert!(!flags.is_dirty());
    }

    #[test]
    fn test_dirty_flags_individual() {
        let mut flags = DirtyFlags::new();
        flags.mark_tabs();
        assert!(flags.tabs_dirty());
        assert!(!flags.content_dirty());

        flags.clear_tabs();
        assert!(!flags.is_dirty());
    }
}
