//! Terminal feedback primitives: toasts, progress bars, status indicators
//!
//! This module provides visual feedback mechanisms for the TUI:
//! - Toast notifications with stacking and auto-dismiss
//! - Progress bars for long-running operations
//! - Status indicators for connection and operation state

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Maximum number of toasts to display simultaneously
const MAX_VISIBLE_TOASTS: usize = 3;
/// Default toast display duration
const DEFAULT_TOAST_DURATION: Duration = Duration::from_secs(4);
/// Extended duration for error toasts
const ERROR_TOAST_DURATION: Duration = Duration::from_secs(6);
/// Cap on queued toasts (including expired until next tick); prevents unbounded memory
const MAX_QUEUED_TOASTS: usize = 32;

/// Severity level for toast notifications (`Info` is lowest; `Error` is highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    /// Get the color associated with this toast level
    pub fn color(&self) -> Color {
        match self {
            ToastLevel::Info => Color::Cyan,
            ToastLevel::Success => Color::Green,
            ToastLevel::Warning => Color::Yellow,
            ToastLevel::Error => Color::Red,
        }
    }

    /// Get the icon/symbol for this toast level
    pub fn icon(&self) -> &'static str {
        match self {
            ToastLevel::Info => "ℹ",
            ToastLevel::Success => "✓",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error => "✗",
        }
    }

    /// Get the duration for this toast level
    pub fn duration(&self) -> Duration {
        match self {
            ToastLevel::Error => ERROR_TOAST_DURATION,
            _ => DEFAULT_TOAST_DURATION,
        }
    }
}

/// Normalize message for dedupe: trim runs of whitespace so near-identical strings match.
fn normalize_toast_message(msg: &str) -> String {
    msg.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A single toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    /// Unique identifier for this toast
    pub id: u64,
    /// The message to display
    pub message: String,
    /// Severity level
    pub level: ToastLevel,
    /// When this toast was created
    pub created_at: Instant,
    /// When this toast should expire
    pub expires_at: Instant,
}

/// Toast notification manager
#[derive(Debug)]
pub struct ToastManager {
    /// Queue of active toasts
    toasts: VecDeque<Toast>,
    /// Counter for generating unique IDs
    next_id: u64,
    /// Maximum number of toasts to keep in history
    max_history: usize,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new() -> Self {
        Self {
            toasts: VecDeque::new(),
            next_id: 1,
            max_history: MAX_QUEUED_TOASTS,
        }
    }

    /// Push a new toast notification.
    ///
    /// Policy: **dedupe** — an active toast with the same normalized message is refreshed
    /// (TTL reset, severity raised if the new level is higher, wording updated) and moved to the
    /// back of the queue instead of adding a duplicate row. Resize/refocus paths that replay the
    /// same notice therefore do not spam the stack.
    pub fn push(&mut self, message: impl Into<String>, level: ToastLevel) {
        let now = Instant::now();
        let message = message.into();
        let norm = normalize_toast_message(&message);

        if !norm.is_empty() {
            if let Some(pos) = self
                .toasts
                .iter()
                .position(|t| t.expires_at > now && normalize_toast_message(&t.message) == norm)
            {
                let mut t = self
                    .toasts
                    .remove(pos)
                    .unwrap_or_else(|| panic!("ToastManager::push: invalid index {pos}"));
                t.message = message;
                t.level = t.level.max(level);
                t.expires_at = now + t.level.duration();
                self.toasts.push_back(t);
                self.trim_queue();
                return;
            }
        }

        let toast = Toast {
            id: self.next_id,
            message,
            level,
            created_at: now,
            expires_at: now + level.duration(),
        };
        self.next_id += 1;
        self.toasts.push_back(toast);
        self.trim_queue();
    }

    fn trim_queue(&mut self) {
        while self.toasts.len() > self.max_history {
            self.toasts.pop_front();
        }
    }

    /// Push an info toast
    pub fn info(&mut self, message: impl Into<String>) {
        self.push(message, ToastLevel::Info);
    }

    /// Push a success toast
    pub fn success(&mut self, message: impl Into<String>) {
        self.push(message, ToastLevel::Success);
    }

    /// Push a warning toast
    pub fn warning(&mut self, message: impl Into<String>) {
        self.push(message, ToastLevel::Warning);
    }

    /// Push an error toast
    pub fn error(&mut self, message: impl Into<String>) {
        self.push(message, ToastLevel::Error);
    }

    /// Remove expired toasts and return whether any were removed
    pub fn cleanup_expired(&mut self) -> bool {
        let now = Instant::now();
        let before_len = self.toasts.len();
        self.toasts.retain(|t| t.expires_at > now);
        self.toasts.len() != before_len
    }

    /// Get all currently active (non-expired) toasts
    pub fn active_toasts(&self) -> impl Iterator<Item = &Toast> {
        let now = Instant::now();
        self.toasts.iter().filter(move |t| t.expires_at > now)
    }

    /// Get the number of active toasts
    pub fn active_count(&self) -> usize {
        self.active_toasts().count()
    }

    /// Check if there are any active toasts
    pub fn has_active(&self) -> bool {
        self.active_count() > 0
    }

    /// Dismiss a specific toast by ID
    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Dismiss all active toasts
    pub fn dismiss_all(&mut self) {
        self.toasts.clear();
    }

    /// Most prominent active toast for one-line surfaces (hint bar): highest severity first,
    /// then newest among ties. Aligns with operator attention to errors over info noise.
    pub fn latest_active_toast(&self) -> Option<&Toast> {
        let now = Instant::now();
        self.toasts
            .iter()
            .filter(|t| t.expires_at > now)
            .max_by_key(|t| (t.level, t.id))
    }

    /// Up to [`MAX_VISIBLE_TOASTS`] active toasts: **severity-first** selection (errors likely
    /// visible even when many infos follow), then **newest** within the same rank. Returned in
    /// **chronological id order** (oldest first in the batch) so stacked bottom-right rendering
    /// keeps the usual oldest-low / newest-high layout.
    pub fn visible_toasts(&self) -> Vec<&Toast> {
        let now = Instant::now();
        let mut active: Vec<&Toast> = self.toasts.iter().filter(|t| t.expires_at > now).collect();
        if active.len() <= MAX_VISIBLE_TOASTS {
            active.sort_by_key(|t| t.id);
            return active;
        }
        active.sort_by_key(|t| (std::cmp::Reverse(t.level), std::cmp::Reverse(t.id)));
        active.truncate(MAX_VISIBLE_TOASTS);
        active.sort_by_key(|t| t.id);
        active
    }
}

/// Progress bar for long-running operations
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// Operation label
    pub label: String,
    /// Current progress (0.0 to 1.0)
    pub progress: f64,
    /// Optional detail message
    pub detail: Option<String>,
    /// When the operation started
    pub started_at: Instant,
    /// Whether the operation is indeterminate
    pub indeterminate: bool,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            progress: 0.0,
            detail: None,
            started_at: Instant::now(),
            indeterminate: false,
        }
    }

    /// Create a new indeterminate progress bar
    pub fn indeterminate(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            progress: 0.0,
            detail: None,
            started_at: Instant::now(),
            indeterminate: true,
        }
    }

    /// Update progress (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
        self.indeterminate = false;
    }

    /// Set detail message
    pub fn set_detail(&mut self, detail: impl Into<String>) {
        self.detail = Some(detail.into());
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Check if operation is complete
    pub fn is_complete(&self) -> bool {
        !self.indeterminate && self.progress >= 1.0
    }
}

/// Status indicator for connections or operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIndicator {
    /// Operation/connection is idle
    Idle,
    /// Operation/connection is active/working
    Active,
    /// Operation completed successfully
    Success,
    /// Warning state
    Warning,
    /// Error state
    Error,
    /// Operation in progress
    Loading,
}

impl StatusIndicator {
    /// Get the color for this status
    pub fn color(&self) -> Color {
        match self {
            StatusIndicator::Idle => Color::Gray,
            StatusIndicator::Active => Color::Cyan,
            StatusIndicator::Success => Color::Green,
            StatusIndicator::Warning => Color::Yellow,
            StatusIndicator::Error => Color::Red,
            StatusIndicator::Loading => Color::Blue,
        }
    }

    /// Get the symbol/emoji for this status (ASCII-compatible)
    pub fn symbol(&self) -> &'static str {
        match self {
            StatusIndicator::Idle => "○",
            StatusIndicator::Active => "●",
            StatusIndicator::Success => "✓",
            StatusIndicator::Warning => "!",
            StatusIndicator::Error => "✗",
            StatusIndicator::Loading => "◐",
        }
    }

    /// Get a spinner frame for loading state
    pub fn spinner(frame: usize) -> &'static str {
        const FRAMES: &[&str] = &["◐", "◓", "◑", "◒"];
        FRAMES[frame % FRAMES.len()]
    }
}

/// Render a toast notification popup
pub fn render_toast_area(frame: &mut Frame, toasts: &ToastManager, area: Rect) {
    let visible = toasts.visible_toasts();
    if visible.is_empty() {
        return;
    }

    // Calculate total height needed
    let toast_height = 3;
    let total_height = (visible.len() as u16 * toast_height).min(area.height);

    // Position in bottom-right corner
    let toast_area = Rect {
        x: area.x + area.width.saturating_sub(40),
        y: area.y + area.height.saturating_sub(total_height),
        width: 40.min(area.width),
        height: total_height,
    };

    // Clear the area
    frame.render_widget(Clear, toast_area);

    // Render each toast
    for (i, toast) in visible.iter().enumerate() {
        let y_offset = i as u16 * toast_height;
        let toast_rect = Rect {
            x: toast_area.x,
            y: toast_area.y + y_offset,
            width: toast_area.width,
            height: toast_height,
        };

        let color = toast.level.color();
        let icon = toast.level.icon();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(Color::Black));

        let msg = super::truncate_detail(
            &toast.message,
            (toast_rect.width.saturating_sub(4)) as usize,
        );
        let content = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(msg, Style::default().fg(Color::White)),
        ]))
        .wrap(Wrap { trim: true })
        .block(block);

        frame.render_widget(content, toast_rect);
    }
}

/// Render a progress bar
pub fn render_progress_bar(frame: &mut Frame, progress: &ProgressBar, area: Rect) {
    let label = if let Some(ref detail) = progress.detail {
        format!("{} - {}", progress.label, detail)
    } else {
        progress.label.clone()
    };

    let gauge = if progress.indeterminate {
        // Indeterminate progress - show a moving pattern
        let elapsed = progress.elapsed().as_millis() as u64;
        let position = (elapsed / 200) % 10;
        let ratio = position as f64 / 10.0;
        Gauge::default()
            .block(Block::default().title(label).borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(ratio)
            .label("Loading...")
    } else {
        let percent = (progress.progress * 100.0) as u16;
        let color = if progress.is_complete() {
            Color::Green
        } else {
            Color::Cyan
        };

        Gauge::default()
            .block(Block::default().title(label).borders(Borders::ALL))
            .gauge_style(Style::default().fg(color))
            .ratio(progress.progress)
            .label(format!("{}%", percent))
    };

    frame.render_widget(gauge, area);
}

/// Render a status indicator line
pub fn render_status_line(items: &[(String, StatusIndicator)]) -> Line<'_> {
    let mut spans = vec![];

    for (i, (label, status)) in items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }

        spans.push(Span::styled(
            status.symbol(),
            Style::default().fg(status.color()),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(label, Style::default().fg(Color::Gray)));
    }

    Line::from(spans)
}

/// Calculate the area for toast notifications (bottom-right)
pub fn toast_area(frame_size: Rect) -> Rect {
    Rect {
        x: frame_size.width.saturating_sub(42),
        y: frame_size.height.saturating_sub(10),
        width: 40.min(frame_size.width),
        height: 9.min(frame_size.height),
    }
}

/// Calculate the area for a centered progress popup
pub fn progress_area(frame_size: Rect) -> Rect {
    let width = 50.min(frame_size.width - 4);
    let height = 5.min(frame_size.height - 4);
    Rect {
        x: (frame_size.width - width) / 2,
        y: (frame_size.height - height) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_manager_push() {
        let mut manager = ToastManager::new();
        manager.info("Test message");
        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_toast_cleanup() {
        let mut manager = ToastManager::new();
        // Note: This would need time manipulation to test properly
        manager.info("Test");
        assert!(!manager.cleanup_expired()); // Should not clean up fresh toast
    }

    #[test]
    fn latest_active_toast_is_newest() {
        let mut m = ToastManager::new();
        m.info("first");
        m.success("second");
        assert_eq!(
            m.latest_active_toast().map(|t| t.message.as_str()),
            Some("second")
        );
    }

    #[test]
    fn visible_toasts_prefers_newest_when_over_cap() {
        let mut m = ToastManager::new();
        for i in 0..5 {
            m.info(format!("t{i}"));
        }
        let labels: Vec<_> = m
            .visible_toasts()
            .into_iter()
            .map(|t| t.message.clone())
            .collect();
        assert_eq!(labels.len(), MAX_VISIBLE_TOASTS);
        assert_eq!(labels, vec!["t2", "t3", "t4"]);
    }

    #[test]
    fn test_progress_bar() {
        let mut progress = ProgressBar::new("Test Operation");
        assert_eq!(progress.progress, 0.0);
        assert!(!progress.is_complete());

        progress.set_progress(0.5);
        assert_eq!(progress.progress, 0.5);

        progress.set_progress(1.0);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_status_indicator() {
        assert_eq!(StatusIndicator::Success.color(), Color::Green);
        assert_eq!(StatusIndicator::Error.symbol(), "✗");
    }
}
