//! Application mode system for TUI.
//!
//! Implements a modal interface similar to Vim, where different modes provide
//! different key bindings and behaviors:
//! - Navigation: Tab switching, scrolling, selection (default)
//! - Edit: Form input, text editing
//! - View: Detail views, charts (read-only)

use ratatui::style::{Color, Style};

/// High-level application mode controlling available actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Navigation mode - tab switching, scrolling, selection (default)
    #[default]
    Navigation,
    /// Edit mode - form input, text editing
    Edit,
    /// View mode - detail views, charts (read-only)
    View,
}

impl AppMode {
    /// Get the display label for this mode.
    pub fn label(&self) -> &'static str {
        match self {
            AppMode::Navigation => "NAV",
            AppMode::Edit => "EDIT",
            AppMode::View => "VIEW",
        }
    }

    /// Get the color associated with this mode.
    pub fn color(&self) -> Color {
        match self {
            AppMode::Navigation => Color::Green,
            AppMode::Edit => Color::Yellow,
            AppMode::View => Color::Cyan,
        }
    }

    /// Get the style for displaying this mode.
    pub fn style(&self) -> Style {
        Style::default().fg(self.color())
    }

    /// Cycle to the next mode (Navigation -> Edit -> View -> Navigation).
    pub fn cycle_next(&self) -> Self {
        match self {
            AppMode::Navigation => AppMode::Edit,
            AppMode::Edit => AppMode::View,
            AppMode::View => AppMode::Navigation,
        }
    }

    /// Check if this mode allows navigation actions (tab switching, scrolling).
    pub fn allows_navigation(&self) -> bool {
        matches!(self, AppMode::Navigation)
    }

    /// Check if this mode allows editing (form input).
    pub fn allows_editing(&self) -> bool {
        matches!(self, AppMode::Edit)
    }

    /// Check if this mode is read-only.
    pub fn is_read_only(&self) -> bool {
        matches!(self, AppMode::View)
    }
}

/// Context for determining the appropriate AppMode based on current state.
#[derive(Debug)]
pub struct ModeContext {
    pub has_detail_popup: bool,
    pub is_settings_editing: bool,
    pub is_loan_form: bool,
    pub is_chart_search: bool,
    pub is_orders_filter: bool,
    pub is_log_panel: bool,
}

impl ModeContext {
    /// Determine the appropriate AppMode based on current UI state.
    pub fn determine_mode(&self) -> AppMode {
        if self.is_settings_editing
            || self.is_loan_form
            || self.is_chart_search
            || self.is_orders_filter
        {
            AppMode::Edit
        } else if self.has_detail_popup || self.is_log_panel {
            AppMode::View
        } else {
            AppMode::Navigation
        }
    }
}

impl Default for ModeContext {
    fn default() -> Self {
        Self {
            has_detail_popup: false,
            is_settings_editing: false,
            is_loan_form: false,
            is_chart_search: false,
            is_orders_filter: false,
            is_log_panel: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_cycle() {
        assert_eq!(AppMode::Navigation.cycle_next(), AppMode::Edit);
        assert_eq!(AppMode::Edit.cycle_next(), AppMode::View);
        assert_eq!(AppMode::View.cycle_next(), AppMode::Navigation);
    }

    #[test]
    fn test_mode_permissions() {
        assert!(AppMode::Navigation.allows_navigation());
        assert!(!AppMode::Navigation.allows_editing());
        assert!(!AppMode::Navigation.is_read_only());

        assert!(!AppMode::Edit.allows_navigation());
        assert!(AppMode::Edit.allows_editing());
        assert!(!AppMode::Edit.is_read_only());

        assert!(!AppMode::View.allows_navigation());
        assert!(!AppMode::View.allows_editing());
        assert!(AppMode::View.is_read_only());
    }

    #[test]
    fn test_mode_context() {
        let ctx = ModeContext {
            is_settings_editing: true,
            ..Default::default()
        };
        assert_eq!(ctx.determine_mode(), AppMode::Edit);

        let ctx = ModeContext {
            has_detail_popup: true,
            ..Default::default()
        };
        assert_eq!(ctx.determine_mode(), AppMode::View);

        let ctx = ModeContext::default();
        assert_eq!(ctx.determine_mode(), AppMode::Navigation);
    }
}
