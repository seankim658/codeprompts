use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

const HELP_TEXT: &[(&str, &str)] = &[
    ("Panels:", ""),
    ("ctrl-h", "Focus panel left"),
    ("ctrl-l", "Focus panel right"),
    ("ctrl-j", "Focus panel below"),
    ("ctrl-k", "Focus panel above"),
    ("", ""),
    ("Move (any panel):", ""),
    ("j / k", "Down / up one row"),
    ("Nj / Nk", "Down / up N rows"),
    ("gg", "Jump to top"),
    ("G", "Jump to bottom"),
    ("", ""),
    ("File Tree:", ""),
    ("enter", "Cycle include/exclude/none"),
    ("space", "Expand/collapse directory"),
    ("+", "Include file/directory"),
    ("-", "Exclude file/directory"),
    ("c", "Collapse all open nodes"),
    ("", ""),
    ("Options & Templates:", ""),
    ("enter", "Toggle selection"),
    ("", ""),
    ("Buttons:", ""),
    ("r", "Run command"),
    ("p", "Print command"),
    ("t", "Reset selections"),
    ("q", "Quit application"),
    ("", ""),
    ("General:", ""),
    ("?", "Toggle help"),
    ("", ""),
    ("Search", ""),
    ("/", "Open fuzzy finder"),
    ("enter", "Cycle mark (any mode)"),
    ("+ / -", "Include / exclude (normal)"),
    ("tab", "Reveal in tree & close"),
    ("esc / q", "Close (normal mode)"),
    ("(config)", "escape_sequence exits insert mode"),
];

pub fn draw_help(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(area, 60, 80);

    frame.render_widget(Clear, popup_area);

    let mut text = Vec::new();
    for &(key, desc) in HELP_TEXT {
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
pub(crate) fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
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
