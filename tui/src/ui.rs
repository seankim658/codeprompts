use crate::prelude::{ActivePanel, FileTree, OptionsPanel, Panel, TemplatesPanel};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const BUTTON_LABELS: [&str; 4] = ["Run", "Print", "Reset", "Exit"];

pub fn draw(
    frame: &mut Frame,
    command: &str,
    options: &mut OptionsPanel,
    file_tree: &mut FileTree,
    templates: &mut TemplatesPanel,
    active_panel: ActivePanel,
    focused_button: usize,
    user_config_found: bool,
) {
    // Calculate height required for command preview
    let max_line_width = frame.area().width as usize - 4;
    let command_height = calculate_wrapped_height(command, max_line_width);
    let command_height = command_height.min(10) as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(command_height + 2),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_command_preview(frame, command, chunks[0], user_config_found);
    draw_main_content(
        frame,
        file_tree,
        options,
        templates,
        active_panel,
        chunks[1],
    );
    draw_button_row(frame, focused_button, active_panel, chunks[2]);
}

fn draw_command_preview(frame: &mut Frame, command: &str, area: Rect, user_config_found: bool) {
    let config_status = if user_config_found {
        " • Config found"
    } else {
        " • No config found"
    };

    let preview = Paragraph::new(command)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Command Preview{}", config_status)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(preview, area);
}

fn draw_main_content(
    frame: &mut Frame,
    file_tree: &mut FileTree,
    options: &mut OptionsPanel,
    templates: &mut TemplatesPanel,
    active_panel: ActivePanel,
    area: Rect,
) {
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(area);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(horizontal_chunks[1]);

    file_tree.draw(
        frame,
        horizontal_chunks[0],
        active_panel == ActivePanel::FileTree,
    );
    options.draw(frame, right_chunks[0], active_panel == ActivePanel::Options);
    templates.draw(
        frame,
        right_chunks[1],
        active_panel == ActivePanel::Templates,
    );
}

fn draw_button_row(
    frame: &mut Frame,
    focused_button: usize,
    active_panel: ActivePanel,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .margin(1)
        .split(area);

    for (i, &label) in BUTTON_LABELS.iter().enumerate() {
        let is_focused = active_panel == ActivePanel::Buttons && focused_button == i;

        let style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let button = Paragraph::new(Line::from(label))
            .style(style)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(button, chunks[i]);
    }
}

fn calculate_wrapped_height(text: &str, max_width: usize) -> usize {
    let mut height = 1;
    let mut current_line_width = 0;

    for word in text.split_whitespace() {
        if current_line_width + word.len() + 1 > max_width {
            height += 1;
            current_line_width = word.len();
        } else {
            if current_line_width > 0 {
                current_line_width += 1;
            }
            current_line_width += word.len();
        }
    }

    height
}
