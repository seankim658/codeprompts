use ratatui::style::{Color, Style, Stylize};

/// UI color schemes and styles
pub struct StyleConfig {
    pub normal: Style,
    pub selected: Style,
    pub header: Style,
    pub included: Style,
    pub excluded: Style,
    pub button: Style,
    pub button_selected: Style,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            normal: Style::default().fg(Color::White),
            selected: Style::default().fg(Color::Yellow),
            header: Style::default().fg(Color::Cyan).bold(),
            included: Style::default().fg(Color::Green),
            excluded: Style::default().fg(Color::Red),
            button: Style::default().fg(Color::White),
            button_selected: Style::default().fg(Color::Black).bg(Color::White),
        }
    }
}
