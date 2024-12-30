use crate::types::{
    command::Command,
    config::{Config, Options},
    file_node::FileNode,
};
use crate::ui::Focus;
use anyhow::Result;
use std::path::PathBuf;
use tui_tree_widget::{TreeItem, TreeState};

/// Manages the application state.
pub struct App {
    pub file_tree: FileNode,
    pub tree_state: TreeState<PathBuf>,
    pub selected_path: Option<PathBuf>,
    pub options: Options,
    pub command: String,
    pub show_command_editor: bool,
    pub config: Option<Config>,
    pub available_templates: Vec<String>,
    pub focus: Focus,
    pub selected_button: usize,
    pub selected_option: Option<usize>,
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let config = Config::load()?;

        // Initialize tree state and open first level by default
        let mut tree_state = TreeState::default();
        tree_state.select_first();

        // Get available templates if config specifies template dir
        let available_templates = if let Some(cfg) = &config {
            cfg.get_available_templates()?
        } else {
            Vec::new()
        };

        // Use config defaults or built-in defaults
        let options = if let Some(cfg) = &config {
            cfg.defaults.clone()
        } else {
            Options::default()
        };

        let mut app = Self {
            file_tree: FileNode::from_path(path),
            tree_state,
            selected_path: None,
            options,
            command: String::new(),
            show_command_editor: false,
            config,
            available_templates,
            focus: Focus::FileTree,
            selected_button: 0,
            selected_option: None,
        };

        app.update_command();

        Ok(app)
    }

    /// Reset app state
    pub fn reset(&mut self) {
        self.selected_path = None;
        self.options = if let Some(cfg) = &self.config {
            cfg.defaults.clone()
        } else {
            Options::default()
        };
        self.show_command_editor = false;
        self.command = String::new();
        self.update_command();
    }

    // Toggle expansion of current directory
    pub fn tree_toggle_expand(&mut self) {
        if let Some(selected) = self.tree_state.selected().first() {
            if selected.is_dir() {
                self.tree_state.toggle(vec![selected.clone()]);
            }
        }
    }

    /// Navigate to next item in tree
    pub fn tree_next(&mut self) {
        if let Some(node_id) = self.tree_state.selected().first().cloned() {
            // Get all visible items as TreeItem vector
            let items = vec![self.file_tree.to_tree_item(self.options.include_priority)];
            // Find next visible item after current selection
            if let Some(next_id) = self.find_next_visible(&items, &node_id) {
                self.tree_state.select(vec![next_id]);
            }
        } else {
            // If nothing selected, select first item
            let root_path = self.file_tree.path.clone();
            self.tree_state.select(vec![root_path]);
        }
    }

    /// Navigate to previous item in tree
    pub fn tree_previous(&mut self) {
        if let Some(node_id) = self.tree_state.selected().first().cloned() {
            // Get all visible items as TreeItem vector
            let items = vec![self.file_tree.to_tree_item(self.options.include_priority)];
            // Find previous visible item before current selection
            if let Some(prev_id) = self.find_previous_visible(&items, &node_id) {
                self.tree_state.select(vec![prev_id]);
            }
        }
    }

    /// Helper to find next visible item in tree
    fn find_next_visible(&self, items: &[TreeItem<PathBuf>], current: &PathBuf) -> Option<PathBuf> {
        fn find_next(
            items: &[TreeItem<PathBuf>],
            current: &PathBuf,
            found: &mut bool,
        ) -> Option<PathBuf> {
            for item in items {
                if *found {
                    return Some(item.identifier().clone());
                }
                if item.identifier() == current {
                    *found = true;
                    if !item.children().is_empty() {
                        return Some(item.children()[0].identifier().clone());
                    }
                    continue;
                }
                if let Some(next) = find_next(item.children(), current, found) {
                    return Some(next);
                }
            }
            None
        }

        let mut found = false;
        find_next(items, current, &mut found)
    }

    /// Helper to find previous visible item in tree
    fn find_previous_visible(
        &self,
        items: &[TreeItem<PathBuf>],
        current: &PathBuf,
    ) -> Option<PathBuf> {
        fn find_prev(items: &[TreeItem<PathBuf>], current: &PathBuf, prev: &mut Option<PathBuf>) {
            for item in items {
                if item.identifier() == current {
                    return;
                }
                *prev = Some(item.identifier().clone());
                find_prev(item.children(), current, prev);
            }
        }

        let mut prev = None;
        find_prev(items, current, &mut prev);
        prev
    }

    /// Update command string based on current state
    pub fn update_command(&mut self) {
        let (includes, excludes) = self.file_tree.get_patterns();
        let mut cmd = String::from("codeprompt ");

        // Add root path
        if let Some(root_path) = self.file_tree.path.to_str() {
            cmd.push_str(&format!("\"{}\" ", root_path));
        }

        // Add include/exclude patterns
        if !includes.is_empty() {
            cmd.push_str(&format!("--include \"{}\" ", includes.join(",")));
        }
        if !excludes.is_empty() {
            cmd.push_str(&format!("--exclude \"{}\" ", excludes.join(",")));
        }

        // Add boolean flags (only add if different from default)
        if !self.options.include_priority {
            cmd.push_str("--include-priority ");
        }
        if self.options.exclude_from_tree {
            cmd.push_str("--exclude-from-tree ");
        }
        if !self.options.gitignore {
            cmd.push_str("--gitignore ");
        }
        if self.options.diff_staged {
            cmd.push_str("--diff-staged ");
        }
        if self.options.diff_unstaged {
            cmd.push_str("--diff-unstaged ");
        }
        if !self.options.tokens {
            cmd.push_str("--tokens ");
        }
        if !self.options.line_numbers {
            cmd.push_str("--line-numbers ");
        }
        if self.options.no_codeblock {
            cmd.push_str("--no-codeblock ");
        }
        if !self.options.relative_paths {
            cmd.push_str("--relative-paths ");
        }
        if self.options.no_clipboard {
            cmd.push_str("--no-clipboard ");
        }
        if !self.options.spinner {
            cmd.push_str("--spinner ");
        }
        if self.options.json {
            cmd.push_str("--json ");
        }
        if self.options.verbose {
            cmd.push_str("--verbose ");
        }

        // Add non-boolean options
        if let Some(template) = &self.options.template {
            if let Some(cfg) = &self.config {
                if let Some(template_dir) = &cfg.template_dir {
                    let template_path = template_dir.join(format!("{}.hbs", template));
                    cmd.push_str(&format!("--template \"{}\" ", template_path.display()));
                }
            }
        }

        if let Some(output) = &self.options.output {
            cmd.push_str(&format!("--output \"{}\" ", output));
        }

        if self.options.encoding != "cl100k" {
            cmd.push_str(&format!("--encoding {} ", self.options.encoding));
        }

        if let Some(issue) = self.options.issue {
            cmd.push_str(&format!("--issue {} ", issue));
        }

        self.command = cmd.trim().to_owned();
    }

    /// Execute the current command
    pub fn execute_command(&self) -> Result<()> {
        let args: Vec<&str> = self.command.split_whitespace().collect();
        if !args.is_empty() {
            Command::new(&args[0]).args(&args[1..]).spawn()?.wait()?;
        }
        Ok(())
    }

    /// Toggles the currently selected option in the options panel
    pub fn toggle_current_option(&mut self) {
        if let Some(idx) = self.selected_option {
            match idx {
                0 => self.options.include_priority = !self.options.include_priority,
                1 => self.options.exclude_from_tree = !self.options.exclude_from_tree,
                2 => self.options.gitignore = !self.options.gitignore,
                3 => self.options.diff_staged = !self.options.diff_staged,
                4 => self.options.diff_unstaged = !self.options.diff_unstaged,
                5 => self.options.tokens = !self.options.tokens,
                6 => self.options.line_numbers = !self.options.line_numbers,
                7 => self.options.no_codeblock = !self.options.no_codeblock,
                8 => self.options.relative_paths = !self.options.relative_paths,
                9 => self.options.no_clipboard = !self.options.no_clipboard,
                10 => self.options.spinner = !self.options.spinner,
                11 => self.options.json = !self.options.json,
                12 => self.options.verbose = !self.options.verbose,
                // Template selection (if available)
                n if n >= 13 && n < 13 + self.available_templates.len() => {
                    let template = self.available_templates[n - 13].clone();
                    self.options.template = Some(template);
                }
                _ => {}
            }
            self.update_command();
        }
    }

    /// Toggles expansion state of the currently selected directory
    pub fn toggle_expand(&mut self) {
        if let Some(path) = &self.selected_path {
            fn toggle_node(node: &mut FileNode, path: &PathBuf) -> bool {
                if node.path == *path {
                    if node.is_dir {
                        node.expanded = !node.expanded;
                        // Directly toggle the tree state
                        if node.expanded {
                        } else {
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    for child in &mut node.children {
                        if toggle_node(child, path) {
                            return true;
                        }
                    }
                    false
                }
            }

            toggle_node(&mut self.file_tree, path);
            self.update_command();
        }
    }

    /// Toggle node include/exclude state at a given path
    pub fn toggle_node_by_path(&mut self, path: PathBuf) {
        fn toggle_node(node: &mut FileNode, path: &PathBuf) {
            if node.path == *path {
                if node.included {
                    node.included = false;
                    node.excluded = true;
                } else if node.excluded {
                    node.included = false;
                    node.excluded = false;
                } else {
                    node.included = true;
                    node.excluded = false;
                }
                return;
            }

            for child in &mut node.children {
                toggle_node(child, path);
            }
        }

        toggle_node(&mut self.file_tree, &path);
    }

    /// Select next item in the file tree
    pub fn select_next(&mut self) {
        if self.selected_path.is_none() {
            // If nothing is selected, select the root
            self.selected_path = Some(self.file_tree.path.clone());
            return;
        }

        let current = self.selected_path.as_ref().unwrap();

        /// Helper function to find next node
        fn find_next(node: &FileNode, current: &PathBuf) -> Option<PathBuf> {
            // If we find the current node
            if &node.path == current {
                // If it has children, return first child
                if !node.children.is_empty() {
                    return Some(node.children[0].path.clone());
                }
                return None; // Will handle sibling case in parent
            }

            // Look through children
            for (idx, child) in node.children.iter().enumerate() {
                if &child.path == current {
                    // If there's a sibling, return it
                    if idx + 1 < node.children.len() {
                        return Some(node.children[idx + 1].path.clone());
                    }
                    return None; // Will handle next parent sibling in caller
                }

                // Recrusively check this child's subtree
                if let Some(next) = find_next(child, current) {
                    return Some(next);
                }
            }

            None
        }

        /// Helper function to find next sibling in parent chain
        fn find_next_parent_sibling(node: &FileNode, current: &PathBuf) -> Option<PathBuf> {
            for (idx, child) in node.children.iter().enumerate() {
                if child.path == *current {
                    if idx + 1 < node.children.len() {
                        return Some(node.children[idx + 1].path.clone());
                    }
                    return None;
                }

                if let Some(next) = find_next_parent_sibling(child, current) {
                    return Some(next);
                }
            }
            None
        }

        // Try to find next node in current subtree
        if let Some(next) = find_next(&self.file_tree, current) {
            self.selected_path = Some(next);
            return;
        }

        // If no next node in subtree, look for next parent sibling
        if let Some(next) = find_next_parent_sibling(&self.file_tree, current) {
            self.selected_path = Some(next);
        }
    }

    /// Select previous item in the file tree
    pub fn select_previous(&mut self) {
        if self.selected_path.is_none() {
            return;
        }

        let current = self.selected_path.as_ref().unwrap();

        /// Helper function to find previous node
        fn find_previous(node: &FileNode, current: &PathBuf) -> Option<PathBuf> {
            for (idx, child) in node.children.iter().enumerate() {
                if &child.path == current {
                    if idx == 0 {
                        // If first child, return parent
                        return Some(node.path.clone());
                    }
                    // Otherwise return last descendant of previous sibling
                    let prev_sibling = &node.children[idx - 1];
                    return Some(get_last_descendant(prev_sibling));
                }

                if let Some(prev) = find_previous(child, current) {
                    return Some(prev);
                }
            }
            None
        }

        /// Helper function to get the last descedant of a node
        fn get_last_descendant(node: &FileNode) -> PathBuf {
            if node.children.is_empty() {
                node.path.clone()
            } else {
                get_last_descendant(node.children.last().unwrap())
            }
        }

        if let Some(prev) = find_previous(&self.file_tree, current) {
            self.selected_path = Some(prev);
        }
    }

    /// Move to parent directory
    pub fn move_to_parent(&mut self) {
        if let Some(current) = &self.selected_path {
            if let Some(parent) = current.parent() {
                self.selected_path = Some(parent.to_path_buf());
            }
        }
    }
}
