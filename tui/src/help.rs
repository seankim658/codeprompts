use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

const HELP_TEXT: [(&str, &str); 19] = [
    ("Navigation:", ""),
    ("ctrl-h", "Move to left panel"),
    ("ctrl-l", "Move to right panel"),
    ("ctrl-j", "Move to panel below"),
    ("ctrl-k", "Move to panel above"),
    ("", ""),
    ("File Tree:", ""),
    ("j/k", "Move up/down"),
    ("enter", "Expand/collapse directory"),
    ("i", "Include file/directory"),
    ("x", "Exclude file/directory"),
    ("", ""),
    ("Options & Templates:", ""),
    ("j/k", "Move up/down"),
    ("enter", "Toggle selection"),
    ("", ""),
    ("General", ""),
    ("?", "Toggle help"),
    ("q", "Quit application"),
];

pub fn draw_help(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(area, 60, 80);

    frame.render_widget(Clear, popup_area);

    let mut text = Vec::new();
    for (key, desc) in HELP_TEXT {
        if desc.is_empty() {
            text.push(Line::from(vec![Span::styled(
                key,
                Style::default().fg(Color::Yellow),
            )]));
        } else if key.is_empty() {
            text.push(Line::from(""));
        } else {
            text.push(Line::from(vec![
                Span::styled(format!("{:10}", key), Style::default().fg(Color::Green)),
                Span::raw(desc),
            ]));
        }
    }

    // Create help popup
    let help = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(help, popup_area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect
fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
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
