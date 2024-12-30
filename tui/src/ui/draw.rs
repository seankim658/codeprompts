use super::{style::StyleConfig, Focus};
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::Tree;

/// Renders the UI
pub fn draw(f: &mut Frame, app: &App) {
    let styles = StyleConfig::default();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Command preview
            Constraint::Min(5),    // Main content
            Constraint::Length(3), // Buttons
        ])
        .split(f.area());

    // Draw command preview
    let command_block = Block::default()
        .title("Command Preview")
        .borders(Borders::ALL);
    let command = Paragraph::new(app.command.clone())
        .block(command_block)
        .wrap(Wrap { trim: true });
    f.render_widget(command, chunks[0]);

    // Split main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Draw file tree
    draw_file_tree(f, app, main_chunks[0], &styles);

    // Draw options panel
    draw_options(f, app, main_chunks[1], &styles);

    // Draw buttons
    draw_buttons(f, app, chunks[2], &styles);

    // Draw command editor if active
    if app.show_command_editor {
        draw_command_editor(f, app);
    }
}

/// Draws the file tree panel
fn draw_file_tree(f: &mut Frame, app: &App, area: Rect, styles: &StyleConfig) {
    let tree_block = Block::default().title("File Tree").borders(Borders::ALL);

    // Convert FileNode to TreeItem
    let items = vec![app.file_tree.to_tree_item(app.options.include_priority)];
    // Create tree widget from items
    let tree = Tree::new(&items)
        .expect("Failed to create tree widget")
        .block(tree_block)
        .highlight_style(styles.selected);

    f.render_widget(tree, area);
}

/// Draws the options panel
fn draw_options(f: &mut Frame, app: &App, area: Rect, styles: &StyleConfig) {
    // Create list of options with their current values
    let option_lines = vec![
        format!(
            "[{}] Include Priority",
            if app.options.include_priority {
                "x"
            } else {
                " "
            }
        ),
        format!(
            "[{}] Exclude From Tree",
            if app.options.exclude_from_tree {
                "x"
            } else {
                " "
            }
        ),
        format!(
            "[{}] Respect Gitignore",
            if app.options.gitignore { "x" } else { " " }
        ),
        format!(
            "[{}] Diff Staged",
            if app.options.diff_staged { "x" } else { " " }
        ),
        format!(
            "[{}] Diff Unstaged",
            if app.options.diff_unstaged { "x" } else { " " }
        ),
        format!(
            "[{}] Show Tokens",
            if app.options.tokens { "x" } else { " " }
        ),
        format!(
            "[{}] Line Numbers",
            if app.options.line_numbers { "x" } else { " " }
        ),
        format!(
            "[{}] No Codeblock",
            if app.options.no_codeblock { "x" } else { " " }
        ),
        format!(
            "[{}] Relative Paths",
            if app.options.relative_paths { "x" } else { " " }
        ),
        format!(
            "[{}] No Clipboard",
            if app.options.no_clipboard { "x" } else { " " }
        ),
        format!(
            "[{}] Show Spinner",
            if app.options.spinner { "x" } else { " " }
        ),
        format!("[{}] JSON Output", if app.options.json { "x" } else { " " }),
        format!("[{}] Verbose", if app.options.verbose { "x" } else { " " }),
    ];

    // Template selection if available
    let template_section = if !app.available_templates.is_empty() {
        let current_template = app.options.template.as_deref().unwrap_or("default");
        vec![
            "".to_owned(),
            "Template".to_owned(),
            format!("Selected: {}", current_template),
        ]
    } else {
        vec![]
    };

    // Combine all lines
    let mut all_lines = option_lines;
    all_lines.extend(template_section);

    // Create spans from lines with appropriate styling
    let spans: Vec<Line> = all_lines
        .into_iter()
        .enumerate()
        .map(|(idx, line)| {
            let style = if app.focus == Focus::Options && Some(idx) == app.selected_option {
                styles.selected
            } else {
                styles.normal
            };
            Line::from(Span::styled(line, style))
        })
        .collect();

    let options_widget = Paragraph::new(spans)
        .block(Block::default().title("Options").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(options_widget, area);
}

/// Draws the button row
fn draw_buttons(f: &mut Frame, app: &App, area: Rect, styles: &StyleConfig) {
    let buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(16),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(0),
        ])
        .split(area);

    // Draw each button
    let button_labels = ["Submit", "Edit Command", "Reset", "Exit"];
    for (idx, &label) in button_labels.iter().enumerate() {
        let style = if app.focus == Focus::Buttons && app.selected_button == idx {
            styles.button_selected
        } else {
            styles.button
        };

        let button = Paragraph::new(label)
            .style(style)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(button, buttons[idx]);
    }
}

/// Draws the command editor popup when active
fn draw_command_editor(f: &mut Frame, app: &App) {
    // Create a centered box for the editor
    let area = centered_rect(60, 20, f.area());

    // Create a clear background
    f.render_widget(Clear, area);

    // Create the editor box with the command
    let editor = Paragraph::new(app.command.clone())
        .block(Block::default().title("Edit Command").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(editor, area);
}

/// Helper function to create a centered rect using up a certain percentage of the available rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Calculate size based on percentage
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
