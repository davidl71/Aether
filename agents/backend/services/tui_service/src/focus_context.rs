//! Compact focus snapshot for routing, logging, and future central input dispatch.
//!
//! See `docs/TUI_PANE_MODEL.md`. Built from [`crate::app::App::focus_context`].

use crate::app::{InputMode, Tab};
use crate::workspace::{SecondaryFocus, VisibleWorkspace};

/// Current input surface + tab + Settings sub-focus (if any).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusContext {
    pub input_mode: InputMode,
    pub active_tab: Tab,
    pub secondary_focus: SecondaryFocus,
    /// Which workspace chrome is visible (full vs split); see `docs/TUI_PANE_MODEL.md`.
    pub visible_workspace: VisibleWorkspace,
}
