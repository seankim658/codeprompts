use crate::handler::{MOVE_DOWN, MOVE_UP};
use crate::prelude::{Config, Panel};
use anyhow::Result;
use codeprompt_core::is_ignored;
use crossterm::event::{KeyCode, KeyEvent};
use ignore::WalkBuilder;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use std::collections::{BTreeMap, HashMap};
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

/// A single entry in the cached file tree structure.
///
/// This is the in-memory representation of the repository layout. It is built once
/// per walk and reused across frames, so that marking a file for include/exclude only
/// rebuilds the rendered items, never the filesystem walk.
#[derive(Debug, Clone)]
pub struct FileNode {
    /// Path relative to the tree root.
    pub rel_path: PathBuf,
    /// Display name (the final path component).
    pub name: String,
    /// Whether this entry is a directory.
    pub is_dir: bool,
    /// Child entries. Empty for files.
    pub children: Vec<FileNode>,
}

/// The result of walking the filesystem once.
pub struct WalkResult {
    /// The nested tree of entries, sorted with directories first, then files.
    pub nodes: Vec<FileNode>,
    /// Flat lookup from a relative path to whether it is a directory. Used to record
    /// `is_dir` at mark time without re-hitting the filesystem.
    pub is_dir_index: BTreeMap<PathBuf, bool>,
}

/// Walks the repository once, honoring `.gitignore` and the ignore list.
///
/// ### Arguments
///
/// - `root`: The root directory to walk.
/// - `gitignore`: Whether to respect the `.gitignore`.
/// - `ignore`: Directory names to skip, matched on the final path component.
///
/// ### Returns
///
/// - `Result<WalkResult>`: The nested node tree and a flat `is_dir` lookup index.
fn build_walk(root: &Path, gitignore: bool, ignore: &[String]) -> Result<WalkResult> {
    let mut nodes: Vec<FileNode> = Vec::new();
    let mut is_dir_index: BTreeMap<PathBuf, bool> = BTreeMap::new();

    let ignore_names = ignore.to_vec();

    let walker = WalkBuilder::new(root)
        .standard_filters(false)
        .git_ignore(gitignore)
        .filter_entry(move |entry| !is_ignored(entry.path(), &ignore_names))
        .build();

    for entry in walker.filter_map(|entry| entry.ok()) {
        let path = entry.path();

        // Skip the root directory itself; we only want its contents.
        let rel_path = match path.strip_prefix(root) {
            Ok(rel) if !rel.as_os_str().is_empty() => rel,
            _ => continue,
        };

        let entry_is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        is_dir_index.insert(rel_path.to_path_buf(), entry_is_dir);

        insert_node(&mut nodes, rel_path, entry_is_dir);
    }

    sort_nodes(&mut nodes);
    Ok(WalkResult {
        nodes,
        is_dir_index,
    })
}

/// Inserts a relative path into the nested node tree, creating intermediate
/// directory nodes as needed.
fn insert_node(nodes: &mut Vec<FileNode>, rel_path: &Path, entry_is_dir: bool) {
    let components: Vec<_> = rel_path.components().collect();
    let mut current = nodes;
    let mut cumulative = PathBuf::new();

    for (idx, component) in components.iter().enumerate() {
        cumulative.push(component);
        let name = component.as_os_str().to_string_lossy().into_owned();
        let is_last = idx == components.len() - 1;

        let position = current.iter().position(|node| node.name == name);
        let index = match position {
            Some(existing) => existing,
            None => {
                // Intermediate components are always directories; the final component
                // uses the entry's own type.
                current.push(FileNode {
                    rel_path: cumulative.clone(),
                    name,
                    is_dir: if is_last { entry_is_dir } else { true },
                    children: Vec::new(),
                });
                current.len() - 1
            }
        };

        // If a directory was first created as an intermediate and is now the entry
        // itself, make sure its type is accurate.
        if is_last {
            current[index].is_dir = entry_is_dir;
        }

        current = &mut current[index].children;
    }
}

/// Sorts nodes in place: directories first, then files, each alphabetically
/// (case-insensitive). Recurses into children.
fn sort_nodes(nodes: &mut [FileNode]) {
    nodes.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    for node in nodes.iter_mut() {
        sort_nodes(&mut node.children);
    }
}

/// File tree panel state
pub struct FileTree {
    /// Root path being displayed
    root: PathBuf,
    /// State for UI interaction
    state: TreeState<String>,
    /// State for tracking current include/exclude selections
    statuses: HashMap<PathBuf, EntryStatus>,
    /// Whether the cached walk honors the `.gitignore`
    gitignore: bool,
    /// Directory names skipped during the walk, resolved from `[global].ignore`
    ignore: Vec<String>,
    /// Cached filesystem walk
    walk: Option<WalkResult>,
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
    pub fn new(config: &Config) -> Result<Self> {
        // Start in current directory by default
        let root = std::env::current_dir()?;

        Ok(Self {
            root,
            state: TreeState::default(),
            statuses: HashMap::new(),
            gitignore: config.tui.defaults.gitignore,
            ignore: config.global.effective_ignore(),
            walk: None,
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
        self.invalidate_walk();
    }

    /// Invalidate the rendered items only
    fn invalidate_cache(&mut self) {
        self.cached_items = None;
    }

    /// Invalidate the filesystem walk and the rendered items
    fn invalidate_walk(&mut self) {
        self.walk = None;
        self.cached_items = None;
    }

    /// Keep the walk's gitignore behavior in sync with the options panel, only rebuilds
    /// the walk if the settings are actually changed
    pub fn set_gitignore(&mut self, gitignore: bool) {
        if self.gitignore != gitignore {
            self.gitignore = gitignore;
            self.invalidate_walk();
        }
    }

    /// Ensure the filesystem walk is cached, building if necessary
    fn ensure_walk(&mut self) {
        if self.walk.is_none() {
            self.walk = Some(
                build_walk(&self.root, self.gitignore, &self.ignore).unwrap_or_else(|e| {
                    WalkResult {
                        nodes: vec![FileNode {
                            rel_path: PathBuf::from("error"),
                            name: format!("Error: {}", e),
                            is_dir: false,
                            children: Vec::new(),
                        }],
                        is_dir_index: BTreeMap::new(),
                    }
                }),
            );
        }
    }

    /// Ensure the rendered items are cached
    fn ensure_items(&mut self) {
        self.ensure_walk();
        if self.cached_items.is_none() {
            let nodes = &self.walk.as_ref().unwrap().nodes;
            self.cached_items = Some(Self::build_tree_items(nodes, &self.statuses));
        }
    }

    /// Build the readable tree items from the cached file nodes.
    ///
    /// This does not touch the filesystem, it walks the in-memory
    /// `FileNode` structure and applied the current include/exclude
    /// styling.
    fn build_tree_items(
        nodes: &[FileNode],
        statuses: &HashMap<PathBuf, EntryStatus>,
    ) -> Vec<TreeItem<'static, String>> {
        let mut items = Vec::new();

        for node in nodes {
            let identifier = node.rel_path.to_string_lossy().into_owned();
            let (style, prefix) = Self::entry_style(&node.rel_path, statuses, false);
            let display_name = format!("{}{}", prefix, node.name);

            let item = if node.is_dir {
                let children = Self::build_tree_items(&node.children, statuses);
                TreeItem::new(identifier, Text::styled(display_name, style), children)
                    .expect("file node identifiers are unique relative paths")
            } else {
                TreeItem::new_leaf(identifier, Text::styled(display_name, style))
            };
            items.push(item);
        }

        items
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
        self.ensure_items();
        self.state.select_last();
    }

    /// Close all open nodes in the tree
    fn close_all_nodes(&mut self) {
        self.state.close_all();
        self.invalidate_cache();
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
            KeyCode::Char('c') => {
                self.close_all_nodes();
                self.last_key_press = None;
            }
            KeyCode::Char(MOVE_DOWN) => {
                self.state.key_down();
                self.last_key_press = None;
            }
            KeyCode::Char(MOVE_UP) => {
                self.state.key_up();
                self.last_key_press = None;
            }
            KeyCode::Enter => {
                self.state.toggle_selected();
                self.last_key_press = None;
            }
            KeyCode::Char(INCLUDE_KEY) => {
                self.toggle_include();
                self.last_key_press = None;
            }
            KeyCode::Char(EXCLUDE_KEY) => {
                self.toggle_exclude();
                self.last_key_press = None;
            }
            _ => {
                self.last_key_press = None;
            }
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

        self.ensure_items();

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
