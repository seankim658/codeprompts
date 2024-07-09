//! Files Module
//!
//! Module that handles all file and file pathing functionality.
//!

use anyhow::Result;
use ignore::WalkBuilder;
use serde_json::json;
use std::fs;
use std::path::Path;
use termtree::Tree;

/// Parses a comma-delimited list from the user arguments.
///
/// ### Arguments
///
/// - `patterns`: Optional string contining comma-delimited patterns.
///
/// ### Returns
///
/// - `Vec<String>`: The parsed list of patterns.
///
pub fn parse_comma_delim_patterns(patterns: &Option<String>) -> Vec<String> {
    if let Some(patterns) = patterns {
        if !patterns.is_empty() {
            return patterns.split(',').map(|s| s.trim().to_owned()).collect();
        }
    }
    vec![]
}

/// Starts at the directory root path and traverses the files to build a tree representation.
///
/// ### Arguments
///
/// - `root`: The path to the root directory.
/// - `include`: The include patterns.
/// - `exclude`: The exclude patterns.
/// - `include_priority`: Whether to give priority to the include patterns.
/// - `line_numbers`: Whether to add line numbers to the code sections.
/// - `relative_paths`: Whether to use relative paths in the file tree.
/// - `exclude_from_tree`: Whether to exclude files picked up by the exclude patterns from the
/// tree.
/// - `no_codeblock`: Whether to wrap the code in markdown code blocks.
///
/// ### Returns
///
/// - `Result<(String, Vec<serde_json::Value>)>`: The string representation of the tree and the
/// JSON representation.
///
pub fn traverse_directory(
    root: &Path,
    include: &[String],
    exclude: &[String],
    include_priority: bool,
    line_numbers: bool,
    relative_paths: bool,
    exclude_from_tree: bool,
    no_codeblock: bool,
) -> Result<(String, Vec<serde_json::Value>)> {
    // Will hold the files found in the traversal.
    let mut files = Vec::new();
    let canonical_root_path = root.canonicalize()?;
    // TODO
}

/// Whether to include a file based on the include/exclude patterns.
///
/// ### Arguments
///
/// - `path`: The path to the file to check.
/// - `include_patterns`: The include patterns.
/// - `exclude_patterns`: The exclude patterns.
/// - `include_priority`: Whether to put precedence on the include or exclude patterns if they
/// conflict.
///
/// ### Returns
///
/// - `bool`: True if the file should be included, False otherwise.
///
fn include(path: &Path, include: &[String], exclude: &[String], include_priority: bool) -> bool {
    let canonical_root_path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(e) => {
            println!("Failed to canonicalize path: {}", e);
            return false;
        }
    };
    let path_string = canonical_root_path.to_str().unwrap();
}
