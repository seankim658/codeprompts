use crate::prelude::{Config, Panel};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
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
        if let Some(template_dir) = &config.template_dir {
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

    fn next(&mut self) {
        let i = match self.interaction_state.selected() {
            Some(i) => {
                // +1 for "None" option at the start
                if i >= self.templates.len() {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.interaction_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.interaction_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.templates.len()
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.interaction_state.select(Some(i));
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
        match key.code {
            KeyCode::Char('j') => self.next(),
            KeyCode::Char('k') => self.previous(),
            KeyCode::Enter => self.select_template(),
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect, is_active: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Templates")
            .border_style(if is_active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        // Start with "None" option
        let mut items = vec![ListItem::new(format!(
            "[{}] None (default template)",
            if self.selected_template.is_none() {
                "x"
            } else {
                " "
            }
        ))];

        // Add available templates
        items.extend(self.templates.iter().map(|path| {
            let is_selected = self
                .selected_template
                .as_ref()
                .map_or(false, |selected| selected == path);

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Invalid path")
                .to_string();

            ListItem::new(format!(
                "[{}] {}",
                if is_selected { "x" } else { " " },
                name
            ))
        }));

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_stateful_widget(list, area, &mut self.interaction_state);
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
