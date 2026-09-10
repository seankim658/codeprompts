use crate::gutter::{self, GutterMode};
use crate::prelude::{Config, Panel};
use crate::theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use std::path::PathBuf;

pub struct TemplatesPanel {
    /// State for UI interaction
    interaction_state: ListState,
    /// Available template files
    templates: Vec<PathBuf>,
    /// Currently selected template
    selected_template: Option<PathBuf>,
}

impl TemplatesPanel {
    pub fn new(config: &Config) -> Result<Self> {
        let mut templates = Vec::new();

        // If template directory is configured, scan for .hbs file
        if let Some(template_dir) = &config.tui.template_dir {
            // Expand `~` if present
            let expanded_path = if template_dir.starts_with("~/") {
                dirs::home_dir()
                    .map(|mut home| {
                        let remainder = template_dir.strip_prefix("~/").unwrap();
                        home.push(remainder);
                        home
                    })
                    .unwrap_or_else(|| template_dir.into())
            } else {
                template_dir.into()
            };

            if expanded_path.exists() {
                for entry in std::fs::read_dir(expanded_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
                        templates.push(path);
                    }
                }
            }
        }

        let mut interaction_state = ListState::default();
        // Always start with "None" selected (index 0)
        interaction_state.select(Some(0));

        Ok(Self {
            interaction_state,
            templates,
            selected_template: None,
        })
    }

    fn select_template(&mut self) {
        if let Some(i) = self.interaction_state.selected() {
            // Index 0 is "None", so subtract 1 from index when selecting template
            self.selected_template = if i == 0 {
                None
            } else {
                Some(self.templates[i - 1].clone())
            };
        }
    }
}

impl Panel for TemplatesPanel {
    fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Enter {
            self.select_template();
        }
        Ok(())
    }

    fn move_down(&mut self, count: usize) {
        let last = self.templates.len();
        let current = self.interaction_state.selected().unwrap_or(0);
        self.interaction_state
            .select(Some((current + count).min(last)));
    }

    fn move_up(&mut self, count: usize) {
        let current = self.interaction_state.selected().unwrap_or(0);
        self.interaction_state
            .select(Some(current.saturating_sub(count)));
    }

    fn jump_to_top(&mut self) {
        self.interaction_state.select(Some(0));
    }

    fn jump_to_bottom(&mut self) {
        self.interaction_state.select(Some(self.templates.len()));
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect, is_active: bool, mode: GutterMode) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(theme::BORDER_TYPE)
            .border_style(theme::border(is_active))
            .title(" Templates ")
            .title_style(theme::title(is_active));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let none_marker = if self.selected_template.is_none() {
            "x"
        } else {
            " "
        };
        let mut contents = vec![format!("[{}] None (default template)", none_marker)];

        contents.extend(self.templates.iter().map(|path| {
            let is_selected = self
                .selected_template
                .as_ref()
                .map_or(false, |selected| selected == path);

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Invalid path")
                .to_string();

            format!("[{}] {}", if is_selected { "x" } else { " " }, name)
        }));

        let total = contents.len();
        let items: Vec<ListItem> = contents
            .into_iter()
            .map(|content| ListItem::new(content))
            .collect();

        let list = List::new(items).highlight_style(theme::selection(is_active));

        if !mode.is_visible() {
            frame.render_stateful_widget(list, inner, &mut self.interaction_state);
            return;
        }

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(gutter::column_width(total) as u16),
                Constraint::Min(0),
            ])
            .split(inner);

        frame.render_stateful_widget(list, columns[1], &mut self.interaction_state);

        gutter::GutterColumn {
            mode,
            offset: self.interaction_state.offset(),
            cursor: self.interaction_state.selected().unwrap_or(0),
            total,
        }
        .draw(frame, columns[0]);
    }

    fn get_command_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(template_path) = &self.selected_template {
            args.push("--template".to_owned());
            args.push(template_path.display().to_string());
        }

        args
    }
}
