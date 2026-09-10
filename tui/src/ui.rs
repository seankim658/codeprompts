use crate::gutter::GutterMode;
use crate::prelude::{ActivePanel, FileTree, OptionsPanel, Panel, TemplatesPanel};
use crate::theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
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
    show_help: bool,
    gutter: GutterMode,
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
        gutter,
        chunks[1],
    );
    draw_button_row(frame, focused_button, active_panel, chunks[2]);

    if show_help {
        crate::help::draw_help(frame, frame.area());
    }
}

fn draw_command_preview(frame: &mut Frame, command: &str, area: Rect, user_config_found: bool) {
    let config_status = if user_config_found {
        "Config found"
    } else {
        "No config found"
    };

    let title = Line::from(vec![
        Span::styled(
            " Command Preview ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("• {} • Press ? for help ", config_status),
            theme::muted(),
        ),
    ]);

    let preview = Paragraph::new(command)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(theme::BORDER_TYPE)
                .border_style(Style::default().fg(theme::MUTED))
                .title(title),
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
    gutter: GutterMode,
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
        gutter,
    );
    options.draw(
        frame,
        right_chunks[0],
        active_panel == ActivePanel::Options,
        gutter,
    );
    templates.draw(
        frame,
        right_chunks[1],
        active_panel == ActivePanel::Templates,
        gutter,
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
        .constraints([Constraint::Ratio(1, 4); 4])
        .margin(1)
        .split(area);

    for (i, &label) in BUTTON_LABELS.iter().enumerate() {
        let is_focused = active_panel == ActivePanel::Buttons && focused_button == i;

        // Size a pill to hug the label, then center it in its cell, so only the
        // focused button shows a solid highlight.
        let pill_area = centered_pill(chunks[i], label.len() as u16 + 4);

        let button = Paragraph::new(label)
            .style(theme::button(is_focused))
            .alignment(Alignment::Center);

        frame.render_widget(button, pill_area);
    }
}

/// Returns a `width`-wide rect centered horizontally within `cell`, clamped to
/// the cell so it never overflows on a narrow terminal.
fn centered_pill(cell: Rect, width: u16) -> Rect {
    let width = width.min(cell.width);
    let x = cell.x + cell.width.saturating_sub(width) / 2;
    Rect {
        x,
        y: cell.y,
        width,
        height: cell.height,
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
