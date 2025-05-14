use crate::handler::{MOVE_DOWN, MOVE_UP};
use crate::prelude::Panel;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tui_tree_widget::{Tree, TreeItem, TreeState};

const INCLUDE_KEY: char = 'i';
const EXCLUDE_KEY: char = 'x';

/// File tree entry status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryStatus {
    /// File/directory is included
    Included,
    /// File/directory is excluded
    Excluded,
    /// No explicit include/exclude status
    None,
}

/// File tree panel state
pub struct FileTree {
    /// Root path being displayed
    root: PathBuf,
    /// State for UI interaction
    state: TreeState<String>,
    /// State for tracking current include/exclude selections
    statuses: HashMap<PathBuf, EntryStatus>,
    /// Cache the file tree so its not redrawn on every frame
    cached_items: Option<Vec<TreeItem<'static, String>>>,
    /// Holds the last key press, used for double key keybinds
    last_key_press: Option<char>,
    /// Key press time, used for tracking double key keybinds
    key_press_time: Instant,
    /// Key timeout for double presses, used for tracking double key keybinds
    key_timeout: Duration,
}

impl FileTree {
    /// Create a new file tree panel
    pub fn new() -> Result<Self> {
        // Start in current directory by default
        let root = std::env::current_dir()?;

        Ok(Self {
            root: root.clone(),
            state: TreeState::default(),
            statuses: HashMap::new(),
            cached_items: None,
            last_key_press: None,
            key_press_time: Instant::now(),
            key_timeout: Duration::from_millis(500),
        })
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.state = TreeState::default();
        self.statuses.clear();
    }

    fn invalidate_cache(&mut self) {
        self.cached_items = None;
    }

    /// Build tree items for a directory
    fn build_tree_items(
        dir: &Path,
        root: &Path,
        statuses: &HashMap<PathBuf, EntryStatus>,
    ) -> Result<Vec<TreeItem<'static, String>>> {
        let mut items = Vec::new();

        for entry in dir.read_dir()? {
            let entry = entry?;
            let abs_path = entry.path();

            let rel_path = abs_path
                .strip_prefix(root)
                .unwrap_or(&abs_path)
                .to_path_buf();

            let identifier = rel_path.to_string_lossy().into_owned();

            let name = abs_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let (style, prefix) = Self::entry_style(&rel_path, statuses, false);
            let display_name = format!("{}{}", prefix, name);

            let item = if abs_path.is_dir() {
                // Recursively build items for subdirectories
                let children = Self::build_tree_items(&abs_path, root, statuses)?;
                TreeItem::new(identifier, Text::styled(display_name, style), children)?
            } else {
                TreeItem::new_leaf(identifier, Text::styled(display_name, style))
            };

            items.push(item);
        }

        Ok(items)
    }

    /// Get an entry's display style based on its status
    fn entry_style(
        path: &Path,
        statuses: &HashMap<PathBuf, EntryStatus>,
        is_selected: bool,
    ) -> (Style, String) {
        let base_style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let (style, prefix) = if let Some(status) = statuses.get(path) {
            match status {
                EntryStatus::Included => (base_style.fg(Color::Green), "[+] "),
                EntryStatus::Excluded => (base_style.fg(Color::Red), "[-] "),
                EntryStatus::None => (base_style, "[ ] "),
            }
        } else {
            (base_style, "[ ] ")
        };

        (style, prefix.to_owned())
    }

    /// Toggle include status for the selected entry
    fn toggle_include(&mut self) {
        if let Some(rel_path) = self.state.selected().last().cloned() {
            let path = PathBuf::from(rel_path);
            let status = self.statuses.entry(path).or_insert(EntryStatus::None);
            *status = match *status {
                EntryStatus::None | EntryStatus::Excluded => EntryStatus::Included,
                EntryStatus::Included => EntryStatus::None,
            };
            self.invalidate_cache();
        }
    }

    /// Toggle exclude status for the selected entry
    fn toggle_exclude(&mut self) {
        if let Some(rel_path) = self.state.selected().last().cloned() {
            let path = PathBuf::from(rel_path);
            let status = self.statuses.entry(path).or_insert(EntryStatus::None);
            *status = match *status {
                EntryStatus::None | EntryStatus::Included => EntryStatus::Excluded,
                EntryStatus::Excluded => EntryStatus::None,
            };
            self.invalidate_cache();
        }
    }

    /// Jump to the top of the tree
    fn jump_to_top(&mut self) {
        self.state.select_first();
    }

    /// Jump to the bottom of the tree
    fn jump_to_bottom(&mut self) {
        if self.cached_items.is_none() {
            self.cached_items = Some(
                Self::build_tree_items(&self.root, &self.root, &self.statuses).unwrap_or_else(
                    |e| {
                        vec![TreeItem::new_leaf(
                            "error".to_owned(),
                            format!("Error: {}", e),
                        )]
                    },
                ),
            )
        }
        self.state.select_last();
    }
}

impl Panel for FileTree {
    fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('g') => {
                let now = Instant::now();
                if let Some('g') = self.last_key_press {
                    if now.duration_since(self.key_press_time) < self.key_timeout {
                        self.jump_to_top();
                        self.last_key_press = None;
                        return Ok(());
                    }
                }
                self.last_key_press = Some('g');
                self.key_press_time = now;
            }
            KeyCode::Char('G') => {
                self.jump_to_bottom();
                self.last_key_press = None;
            }
            KeyCode::Char(MOVE_DOWN) => {
                self.state.key_down();
            }
            KeyCode::Char(MOVE_UP) => {
                self.state.key_up();
            }
            KeyCode::Enter => {
                self.state.toggle_selected();
            }
            KeyCode::Char(INCLUDE_KEY) => {
                self.toggle_include();
            }
            KeyCode::Char(EXCLUDE_KEY) => {
                self.toggle_exclude();
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, is_active: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("File Tree")
            .border_style(if is_active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        if self.cached_items.is_none() {
            self.cached_items = Some(
                Self::build_tree_items(&self.root, &self.root, &self.statuses).unwrap_or_else(
                    |e| {
                        vec![TreeItem::new_leaf(
                            "error".to_owned(),
                            format!("Error: {}", e),
                        )]
                    },
                ),
            );
        };

        let tree = Tree::new(self.cached_items.as_ref().unwrap())
            .expect("Tree items have unique identifiers")
            .block(block)
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_stateful_widget(tree, area, &mut self.state);
    }

    fn get_command_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        let includes: Vec<String> = self
            .statuses
            .iter()
            .filter(|(_, status)| **status == EntryStatus::Included)
            .map(|(path, _)| {
                if path.is_dir() {
                    // For directories, include everything
                    format!("./{}/*", path.display())
                } else {
                    format!("./{}", path.display())
                }
            })
            .collect();

        let excludes: Vec<String> = self
            .statuses
            .iter()
            .filter(|(_, status)| **status == EntryStatus::Excluded)
            .map(|(path, _)| {
                if path.is_dir() {
                    // For directories, exclude everything
                    format!("{}/*", path.display())
                } else {
                    path.display().to_string()
                }
            })
            .collect();

        if !includes.is_empty() {
            args.push("--include".to_owned());
            let include_arg = format!("{}", includes.join(", "));
            args.push(include_arg);
        }

        if !excludes.is_empty() {
            args.push("--exclude".to_owned());
            let exclude_args = format!("{}", excludes.join(", "));
            args.push(exclude_args);
        }

        args
    }
}
