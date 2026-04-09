//! Theme-derived colors for operator-facing chrome (TUI_THEME / runtime toggle).
//!
//! Keeps high-contrast vs default differences in one place; panes can opt in gradually.

use ratatui::style::Color;

use crate::config::TuiTheme;

/// Colors that vary with [`TuiTheme`].
#[derive(Debug, Clone, Copy)]
pub struct UiPalette {
    pub accent: Color,
    pub muted: Color,
    pub selection_fg: Color,
    pub selection_bg: Color,
    pub filter_border_active: Color,
    pub filter_text_active: Color,
    pub help_title: Color,
    pub palette_border: Color,
    pub palette_search_border: Color,
}

impl UiPalette {
    pub fn from_theme(theme: TuiTheme) -> Self {
        match theme {
            TuiTheme::Default => Self {
                accent: Color::Cyan,
                muted: Color::DarkGray,
                selection_fg: Color::Black,
                selection_bg: Color::Yellow,
                filter_border_active: Color::Yellow,
                filter_text_active: Color::Cyan,
                help_title: Color::Yellow,
                palette_border: Color::Cyan,
                palette_search_border: Color::DarkGray,
            },
            TuiTheme::HighContrast => Self {
                accent: Color::LightCyan,
                muted: Color::Gray,
                selection_fg: Color::Black,
                selection_bg: Color::LightYellow,
                filter_border_active: Color::LightYellow,
                filter_text_active: Color::White,
                help_title: Color::White,
                palette_border: Color::LightCyan,
                palette_search_border: Color::Gray,
            },
        }
    }
}
