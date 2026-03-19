//! Logs tab: tui-logger widget.

use ratatui::{layout::Rect, style::Color, style::Style, widgets::Block, widgets::Borders, Frame};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use crate::app::App;

pub fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let widget = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title("Logs  [+/-]:level  [↑↓ PgUp/Dn]:scroll  [h]:hide  [Esc]:reset")
                .borders(Borders::ALL),
        )
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
        .state(&app.log_state);
    f.render_widget(widget, area);
}
