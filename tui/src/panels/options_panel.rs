use crate::prelude::{Config, OptionState, Panel};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub struct OptionsPanel {
    /// State for UI interaction
    interaction_state: ListState,
    /// State of the currently selected options
    options: OptionState,
}

impl OptionsPanel {
    pub fn new(config: &Config) -> Self {
        let mut interaction_state = ListState::default();
        // Start with first item selected
        interaction_state.select(Some(0));

        Self {
            interaction_state,
            options: config.defaults.clone(),
        }
    }

    fn next(&mut self) {
        let options_len = self.options.num();
        let i = match self.interaction_state.selected() {
            Some(i) => {
                if i >= options_len - 1 {
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
        let options_len = self.options.num();
        let i = match self.interaction_state.selected() {
            Some(i) => {
                if i == 0 {
                    options_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.interaction_state.select(Some(i));
    }

    fn toggle_selected(&mut self) {
        if let Some(i) = self.interaction_state.selected() {
            if let Some((_, value)) = self.options.options_iter_mut().get_mut(i) {
                **value = !**value
            }
        }
    }
}

impl Panel for OptionsPanel {
    fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') => self.next(),
            KeyCode::Char('k') => self.previous(),
            KeyCode::Enter => self.toggle_selected(),
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect, is_active: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Options")
            .border_style(if is_active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let items = self
            .options
            .options_iter_mut()
            .into_iter()
            .map(|(name, value)| {
                ListItem::new(format!("[{}] {}", if *value { "x" } else { " " }, name))
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_stateful_widget(list, area, &mut self.interaction_state);
    }

    fn get_command_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.options.exclude_priority {
            args.push("--exclude-priority".to_owned());
        }
        if self.options.exclude_from_tree {
            args.push("--exclude-from-tree".to_owned());
        }
        if self.options.diff_staged {
            args.push("--diff-staged".to_owned());
        }
        if self.options.diff_unstaged {
            args.push("--diff-unstaged".to_owned());
        }
        if !self.options.gitignore {
            args.push("--gitignore".to_owned());
        }
        if self.options.no_tokens {
            args.push("--no-tokens".to_owned());
        }
        if self.options.no_line_numbers {
            args.push("--no-line-numbers".to_owned());
        }
        if self.options.no_codeblock {
            args.push("--no-codeblock".to_owned());
        }
        if self.options.relative_paths {
            args.push("--relative_paths".to_owned());
        }
        if self.options.no_clipboard {
            args.push("--no-clipboard".to_owned());
        }
        if self.options.no_spinner {
            args.push("--no-spinner".to_owned());
        }

        args
    }
}
