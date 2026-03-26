//! Discoverability layer: command palette, help system, context hints
//!
//! Provides user interface elements to help users discover and learn key bindings:
//! - Command palette (Cmd+Shift+P style): searchable list of all actions
//! - Enhanced help overlay: organized by mode with search
//! - Context hints: dynamic hints based on current mode/tab

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Tab};
use crate::input::Action;
use crate::mode::AppMode;

/// A command that can be executed from the command palette
#[derive(Debug, Clone)]
pub struct Command {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description of what the command does
    pub description: String,
    /// Key binding(s) that trigger this command
    pub keys: Vec<String>,
    /// Which modes this command is available in
    pub available_in: Vec<AppMode>,
    /// Which tabs this command is available in (empty = all)
    pub tabs: Vec<Tab>,
    /// The action to execute
    pub action: Action,
}

impl Command {
    /// Create a new command
    pub fn new(id: impl Into<String>, name: impl Into<String>, action: Action) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            keys: Vec::new(),
            available_in: vec![AppMode::Navigation, AppMode::Edit, AppMode::View],
            tabs: Vec::new(),
            action,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set key bindings
    pub fn keys(mut self, keys: Vec<String>) -> Self {
        self.keys = keys;
        self
    }

    /// Set available modes
    pub fn available_in(mut self, modes: Vec<AppMode>) -> Self {
        self.available_in = modes;
        self
    }

    /// Set available tabs
    pub fn tabs(mut self, tabs: Vec<Tab>) -> Self {
        self.tabs = tabs;
        self
    }
}

/// Command palette state
#[derive(Debug, Default)]
pub struct CommandPalette {
    /// Whether the palette is visible
    pub visible: bool,
    /// Search input buffer
    pub search: String,
    /// Selected command index
    pub selected: usize,
    /// All available commands
    pub commands: Vec<Command>,
    /// Filtered commands (matching search)
    pub filtered: Vec<usize>, // indices into commands
}

impl CommandPalette {
    /// Create a new command palette with default commands
    pub fn new() -> Self {
        let commands = build_command_registry();
        let filtered: Vec<usize> = (0..commands.len()).collect();

        Self {
            visible: false,
            search: String::new(),
            selected: 0,
            commands,
            filtered,
        }
    }

    /// Show the command palette
    pub fn show(&mut self) {
        self.visible = true;
        self.search.clear();
        self.selected = 0;
        self.update_filtered();
    }

    /// Hide the command palette
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Add a character to the search
    pub fn push_char(&mut self, c: char) {
        self.search.push(c);
        self.update_filtered();
        self.selected = 0;
    }

    /// Remove last character from search
    pub fn backspace(&mut self) {
        self.search.pop();
        self.update_filtered();
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    /// Get the currently selected command
    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.commands.get(idx))
    }

    /// Update filtered list based on search
    fn update_filtered(&mut self) {
        let search_lower = self.search.to_lowercase();
        self.filtered = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| {
                let matches_search = search_lower.is_empty()
                    || cmd.name.to_lowercase().contains(&search_lower)
                    || cmd.description.to_lowercase().contains(&search_lower)
                    || cmd
                        .keys
                        .iter()
                        .any(|k| k.to_lowercase().contains(&search_lower));
                matches_search
            })
            .map(|(idx, _)| idx)
            .collect();
    }

    /// Filter commands by current mode and tab
    pub fn available_commands(&self, mode: AppMode, tab: Tab) -> Vec<&Command> {
        self.commands
            .iter()
            .filter(|cmd| {
                cmd.available_in.contains(&mode) && (cmd.tabs.is_empty() || cmd.tabs.contains(&tab))
            })
            .collect()
    }
}

/// Build the default command registry
fn build_command_registry() -> Vec<Command> {
    vec![
        Command::new("quit", "Quit", Action::Quit)
            .description("Exit the application")
            .keys(vec!["q".into()]),
        Command::new("help", "Show Help", Action::ShowHelp)
            .description("Show help overlay with key bindings")
            .keys(vec!["?".into()]),
        Command::new("tab_next", "Next Tab", Action::TabNext)
            .description("Switch to next tab")
            .keys(vec!["Tab".into()]),
        Command::new("tab_prev", "Previous Tab", Action::TabPrev)
            .description("Switch to previous tab")
            .keys(vec!["Shift+Tab".into()]),
        Command::new("jump_dashboard", "Jump to Dashboard", Action::JumpToTab(1))
            .description("Jump to Dashboard tab")
            .keys(vec!["1".into()]),
        Command::new("jump_positions", "Jump to Positions", Action::JumpToTab(2))
            .description("Jump to Positions tab")
            .keys(vec!["2".into()]),
        Command::new("jump_charts", "Jump to Charts", Action::JumpToTab(3))
            .description("Jump to Charts tab")
            .keys(vec!["3".into()]),
        Command::new("jump_orders", "Jump to Orders", Action::JumpToTab(4))
            .description("Jump to Orders tab")
            .keys(vec!["4".into()]),
        Command::new("jump_alerts", "Jump to Alerts", Action::JumpToTab(5))
            .description("Jump to Alerts tab")
            .keys(vec!["5".into()]),
        Command::new("jump_yield", "Jump to Yield", Action::JumpToTab(6))
            .description("Jump to Yield Curve tab")
            .keys(vec!["6".into()]),
        Command::new("jump_loans", "Jump to Loans", Action::JumpToTab(7))
            .description("Jump to Loans tab")
            .keys(vec!["7".into()]),
        Command::new("jump_scenarios", "Jump to Scenarios", Action::JumpToTab(9))
            .description("Jump to Scenarios tab")
            .keys(vec!["9".into()]),
        Command::new("jump_settings", "Jump to Settings", Action::JumpToTab(0))
            .description("Jump to Settings tab")
            .keys(vec!["0".into()]),
        Command::new("split_pane", "Toggle Split Pane", Action::SplitPaneToggle)
            .description("Toggle split pane view")
            .keys(vec!["p".into()]),
        Command::new("log_panel", "Toggle Log Panel", Action::ToggleLogPanel)
            .description("Toggle log panel overlay")
            .keys(vec!["`".into(), "~".into()]),
        Command::new("mode_cycle", "Cycle Mode", Action::ModeCycle)
            .description("Cycle through application modes")
            .keys(vec!["m".into()]),
    ]
}

/// Render the command palette
pub fn render_command_palette(f: &mut Frame, palette: &CommandPalette, area: Rect) {
    if !palette.visible {
        return;
    }

    let area = centered_rect(60, 70, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Command Palette ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title_style(Style::default().add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    // Search input
    let search_style = Style::default().fg(Color::Yellow);
    let search_text = format!("> {}", palette.search);
    let search = Paragraph::new(search_text).style(search_style).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(search, chunks[0]);

    // Command list
    let items: Vec<ListItem> = palette
        .filtered
        .iter()
        .enumerate()
        .map(|(i, &cmd_idx)| {
            let cmd = &palette.commands[cmd_idx];
            let keys = cmd.keys.join(", ");
            let content = format!("{}  {}", keys, cmd.name);

            let style = if i == palette.selected {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(keys, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::raw(&cmd.name),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .highlight_spacing(HighlightSpacing::Always)
        .block(Block::default());

    f.render_widget(list, chunks[1]);
}

/// Render context-sensitive hints at the bottom of the screen
pub fn render_context_hints(f: &mut Frame, app: &App, area: Rect) {
    let hints = get_context_hints(app);
    if hints.is_empty() {
        return;
    }

    let spans: Vec<Span> = hints
        .into_iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(key, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(":{}  ", desc)),
            ]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Get context-sensitive hints based on current state
fn get_context_hints(app: &App) -> Vec<(String, String)> {
    let mut hints = vec![];

    match app.app_mode {
        AppMode::Navigation => {
            hints.push(("Tab".into(), "next".into()));
            hints.push(("?".into(), "help".into()));
            hints.push((":".into(), "palette".into()));

            match app.active_tab {
                Tab::Positions => {
                    hints.push(("↑↓".into(), "scroll".into()));
                    hints.push(("Enter".into(), "detail".into()));
                }
                Tab::Orders => {
                    hints.push(("/".into(), "filter".into()));
                }
                Tab::Charts => {
                    hints.push(("/".into(), "search".into()));
                }
                _ => {}
            }
        }
        AppMode::Edit => {
            hints.push(("Esc".into(), "cancel".into()));
            hints.push(("Enter".into(), "confirm".into()));
        }
        AppMode::View => {
            hints.push(("Esc".into(), "close".into()));
            hints.push(("↑↓".into(), "scroll".into()));
        }
    }

    hints
}

/// Calculate a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_filter() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Should show all commands initially
        assert!(!palette.filtered.is_empty());

        // Filter for "quit"
        palette.push_char('q');
        palette.push_char('u');
        palette.push_char('i');

        // Should find quit command
        assert!(palette.filtered.len() <= 5);

        // Clear filter
        palette.backspace();
        palette.backspace();
        palette.backspace();

        // Should show all commands again
        assert!(!palette.filtered.is_empty());
    }

    #[test]
    fn test_context_hints_navigation() {
        // Just verify it doesn't panic
        let app = App::default();
        let hints = get_context_hints(&app);
        assert!(!hints.is_empty());
    }
}
