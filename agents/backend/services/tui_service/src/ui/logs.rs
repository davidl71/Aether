//! Logs tab: tui-logger widget.

use ratatui::{layout::Rect, style::Color, style::Style, widgets::Block, widgets::Borders, Frame};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use crate::app::App;

fn level_name(level: log::LevelFilter) -> &'static str {
    match level {
        log::LevelFilter::Error => "ERROR",
        log::LevelFilter::Warn => "WARN",
        log::LevelFilter::Info => "INFO",
        log::LevelFilter::Debug => "DEBUG",
        log::LevelFilter::Trace => "TRACE",
        _ => "ALL",
    }
}

pub(crate) fn logs_title(app: &App) -> String {
    let lvl = level_name(app.log_display_level);
    format!("Logs [{lvl}]  [+/- e/w/i/d]:level  [↑↓ PgUp/Dn]:scroll  [h]:hide  [Esc]:reset")
}

pub(crate) fn build_logs_widget<'a>(app: &'a App, title: String) -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::Cyan))
        .style_debug(Style::default().fg(Color::White))
        .style_trace(Style::default().fg(Color::DarkGray))
        .output_separator(' ')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&app.log_state)
}

pub fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let widget = build_logs_widget(app, logs_title(app));
    f.render_widget(widget, area);
}
