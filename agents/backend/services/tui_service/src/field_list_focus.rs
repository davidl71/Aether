//! Shared field-vs-list sub-focus for modal overlays (`ratatui-interact` / `FocusManager`).
//!
//! Used when an overlay has a text field plus a scrollable list: Tab / Shift+Tab cycles
//! regions; while focus is on the list, character keys are ignored so arrows move selection.

use ratatui_interact::state::FocusManager;

/// Two focusable bands: editable field (filter, search, path) and result/command list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum FieldListRegion {
    Field,
    List,
}

/// Tab order between [`FieldListRegion::Field`] and [`FieldListRegion::List`].
#[derive(Debug)]
pub(crate) struct FieldListFocus {
    focus: FocusManager<FieldListRegion>,
}

impl FieldListFocus {
    pub(crate) fn new() -> Self {
        Self {
            focus: FocusManager::new(),
        }
    }

    /// Register both regions and start on the field.
    pub(crate) fn on_open(&mut self) {
        self.focus.clear();
        self.focus.register(FieldListRegion::Field);
        self.focus.register(FieldListRegion::List);
    }

    pub(crate) fn on_close(&mut self) {
        self.focus.clear();
    }

    pub(crate) fn tab_next(&mut self) {
        self.focus.next();
    }

    pub(crate) fn tab_prev(&mut self) {
        self.focus.prev();
    }

    /// When false, typed characters and Backspace should not edit the field (`NoOp`).
    pub(crate) fn allows_field_edit(&self) -> bool {
        self.focus
            .current()
            .is_some_and(|r| *r == FieldListRegion::Field)
    }

    /// Active band when the overlay is open (`on_open`); [`None`] after `on_close` or before register.
    pub(crate) fn focused_region(&self) -> Option<FieldListRegion> {
        self.focus.current().copied()
    }
}

// Integration with real widgets / terminal input is exercised manually; these tests only pin
// the `FocusManager` wrapper contract (open/close, Tab order, `allows_field_edit`).
#[cfg(all(test, feature = "tui-interact"))]
mod tests {
    use super::*;

    #[test]
    fn opened_starts_on_field_and_tabs_cycle() {
        let mut s = FieldListFocus::new();
        s.on_open();
        assert!(s.allows_field_edit());
        s.tab_next();
        assert!(!s.allows_field_edit());
        s.tab_next();
        assert!(s.allows_field_edit());
        s.tab_prev();
        assert!(!s.allows_field_edit());
    }

    #[test]
    fn closed_clears_focus() {
        let mut s = FieldListFocus::new();
        s.on_open();
        s.tab_next();
        s.on_close();
        assert!(!s.allows_field_edit());
        assert!(s.focused_region().is_none());
    }

    #[test]
    fn focused_region_tracks_tab_target() {
        let mut s = FieldListFocus::new();
        s.on_open();
        assert_eq!(s.focused_region(), Some(FieldListRegion::Field));
        s.tab_next();
        assert_eq!(s.focused_region(), Some(FieldListRegion::List));
    }

    #[test]
    fn before_on_open_no_region_and_field_edit_disabled() {
        let s = FieldListFocus::new();
        assert_eq!(s.focused_region(), None);
        assert!(!s.allows_field_edit());
    }

    #[test]
    fn tab_next_prev_before_on_open_is_safe_noop() {
        let mut s = FieldListFocus::new();
        s.tab_next();
        s.tab_prev();
        assert_eq!(s.focused_region(), None);
        assert!(!s.allows_field_edit());
    }

    #[test]
    fn prev_from_field_wraps_to_list() {
        let mut s = FieldListFocus::new();
        s.on_open();
        assert_eq!(s.focused_region(), Some(FieldListRegion::Field));
        s.tab_prev();
        assert_eq!(s.focused_region(), Some(FieldListRegion::List));
        assert!(!s.allows_field_edit());
    }

    #[test]
    fn on_open_resets_to_field_after_list_focus() {
        let mut s = FieldListFocus::new();
        s.on_open();
        s.tab_next();
        assert_eq!(s.focused_region(), Some(FieldListRegion::List));
        s.on_open();
        assert_eq!(s.focused_region(), Some(FieldListRegion::Field));
        assert!(s.allows_field_edit());
    }

    #[test]
    fn reopen_after_close_starts_on_field() {
        let mut s = FieldListFocus::new();
        s.on_open();
        s.tab_next();
        s.on_close();
        s.on_open();
        assert_eq!(s.focused_region(), Some(FieldListRegion::Field));
        assert!(s.allows_field_edit());
    }
}
