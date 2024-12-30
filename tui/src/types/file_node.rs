use ratatui::prelude::Text;
use std::path::PathBuf;
use tui_tree_widget::TreeItem;

/// Represents a file or directory in the file system tree.
#[derive(Debug)]
pub struct FileNode {
    /// Path to the file/directory.
    pub path: PathBuf,
    /// Whether this node represents a directory
    pub is_dir: bool,
    /// Whether this file is explicitly included
    pub included: bool,
    /// Whether this file is explicitly excluded
    pub excluded: bool,
    /// Child nodes (empty for files, contains subdirectories/files for directories)
    pub children: Vec<FileNode>,
}

impl FileNode {
    /// Creates a `FileNode` from a given path, recursively building the tree.
    ///
    /// ### Parameters
    /// - `path`: The Path to create the node from
    /// - `max_depth`: Optional maximum depth to auto-expand to (`None` for no auto-expand)
    /// - `current_depth`: Current depth in the recusion
    pub fn from_path_with_depth(
        path: PathBuf,
        max_depth: Option<usize>,
        current_depth: usize,
    ) -> Self {
        let is_dir = path.is_dir();

        let children = if is_dir {
            std::fs::read_dir(&path)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| {
                    FileNode::from_path_with_depth(entry.path(), max_depth, current_depth + 1)
                })
                .collect()
        } else {
            vec![]
        };

        FileNode {
            path,
            is_dir,
            included: false,
            excluded: false,
            children,
        }
    }

    /// Creates a `FileNode` from a given path, recursively building the tree.
    pub fn from_path(path: PathBuf) -> Self {
        // Expand first level by default
        Self::from_path_with_depth(path, Some(1), 0)
    }

    /// Converts the `FileNode` into a `TreeItem` for display.
    pub fn to_tree_item<'a>(&'a self, include_priority: bool) -> TreeItem<'a, PathBuf> {
        // Determine display prefix based on include/exclude state
        let prefix = match (self.included, self.excluded) {
            (true, true) => {
                if include_priority {
                    "[+] "
                } else {
                    "[-] "
                }
            }
            (true, false) => "[+] ",
            (false, true) => "[-] ",
            _ => "[] ",
        };

        // Get the filename for display
        let name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text = format!("{}{}", prefix, name);

        // Create TreeItem with the path as identifier
        TreeItem::new(
            self.path.clone(),
            Text::from(text),
            self.children
                .iter()
                .map(|child| child.to_tree_item(include_priority))
                .collect(),
        )
        .expect("Valid TreeItem")
    }

    /// Gets include/exclude patterns for command construction.
    pub fn get_patterns(&self) -> (Vec<String>, Vec<String>) {
        let mut includes = Vec::new();
        let mut excludes = Vec::new();
        self.collect_patterns(&mut includes, &mut excludes);
        (includes, excludes)
    }

    /// Recursively collects include/exclude patterns
    fn collect_patterns(&self, includes: &mut Vec<String>, excludes: &mut Vec<String>) {
        if self.included {
            if self.is_dir {
                includes.push(format!("{}/*", self.path.display()));
            } else {
                includes.push(self.path.display().to_string());
            }
        } else if self.excluded {
            if self.is_dir {
                excludes.push(format!("{}/*", self.path.display()));
            } else {
                excludes.push(self.path.display().to_string());
            }
        }

        for child in &self.children {
            child.collect_patterns(includes, excludes);
        }
    }

    /// Toggle include/exclude state for a node
    pub fn toggle_state(&mut self) {
        if self.included {
            self.included = false;
            self.excluded = true;
        } else if self.excluded {
            self.included = false;
            self.excluded = false;
        } else {
            self.included = true;
            self.excluded = false;
        }
    }

    /// Find node by path and toggle its state
    pub fn toggle_node_by_path(&mut self, path: &PathBuf) {
        if self.path == *path {
            self.toggle_state();
            return;
        }

        for child in &mut self.children {
            child.toggle_node_by_path(path);
        }
    }
}
