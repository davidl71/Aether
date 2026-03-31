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
}

#[cfg(test)]
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
    }
}
