use crate::gutter::{self, GutterMode};
use crate::prelude::{Config, OptionState, Panel};
use crate::theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
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
            options: config.tui.defaults.clone(),
        }
    }

    fn toggle_selected(&mut self) {
        if let Some(i) = self.interaction_state.selected() {
            if let Some((_, value)) = self.options.options_iter_mut().get_mut(i) {
                **value = !**value
            }
        }
    }

    pub fn gitignore(&self) -> bool {
        self.options.gitignore
    }
}

impl Panel for OptionsPanel {
    fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Enter {
            self.toggle_selected();
        }
        Ok(())
    }

    fn move_down(&mut self, count: usize) {
        let len = self.options.num();
        if len == 0 {
            return;
        }
        let current = self.interaction_state.selected().unwrap_or(0);
        self.interaction_state
            .select(Some((current + count).min(len - 1)));
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
        let len = self.options.num();
        if len > 0 {
            self.interaction_state.select(Some(len - 1));
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect, is_active: bool, mode: GutterMode) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(theme::BORDER_TYPE)
            .border_style(theme::border(is_active))
            .title(" Options ")
            .title_style(theme::title(is_active));

        let cursor = self.interaction_state.selected().unwrap_or(0);
        let width = gutter::number_width(self.options.num());

        let items = self
            .options
            .options_iter_mut()
            .into_iter()
            .enumerate()
            .map(|(index, (name, value))| {
                let content = format!("[{}] {}", if *value { "x" } else { " " }, name);
                let gutter_cell = gutter::cell(mode, index, cursor, width);
                let gutter_style = if index == cursor {
                    theme::gutter_current()
                } else {
                    theme::gutter()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(gutter_cell, gutter_style),
                    Span::raw(content),
                ]))
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(block)
            .highlight_style(theme::selection(is_active));

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
        if self.options.literal_brackets {
            args.push("--literal-brackets".to_owned());
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
        if self.options.absolute_paths {
            args.push("--absolute-paths".to_owned());
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
