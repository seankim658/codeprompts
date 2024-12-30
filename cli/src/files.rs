//! Files Module
//!
//! Module that handles all file and file pathing functionality.

use anyhow::Result;
use glob::Pattern;
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
/// - `gitignore`: Whether or not to respect the gitignore file.
/// - `verbose`: Whether to print the glob pattern matching for investigation.
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
    gitignore: bool,
    verbose: bool,
) -> Result<(String, Vec<serde_json::Value>)> {
    // Will hold the files found in the traversal.
    let mut files = Vec::new();
    // Canonicalize returns the canonical, absolute form of a path with all intermediate components
    // normalized and symbolic links resolved. It errors if the path does not exist or if the final
    // component in path is not a directory.
    let canonical_root_path = root.canonicalize()?;
    let parent_dir = basename(&canonical_root_path);

    if verbose {
        println!(
            "Include patterns: {:?}\nExclude patterns: {:?}",
            include, exclude
        );
    }

    // Walk through the directory tree.
    // Initialize a WalkBuilder with the canonical root path (by default, respects .gitignore).
    let tree = WalkBuilder::new(&canonical_root_path)
        .git_ignore(gitignore)
        .build()
        // Filter out errors, only keep successful entries.
        .filter_map(|e| e.ok())
        // Use fold to accumulate entries into the tree structure, starting from the root
        // directory.
        .fold(Tree::new(parent_dir.to_owned()), |mut root, entry| {
            let path = entry.path();
            // Computes the relative path from the root directory.
            if let Ok(relative_path) = path.strip_prefix(&canonical_root_path) {
                // Initialize the current tree to the root of the tree.
                let mut current_tree = &mut root;
                // Iterate over each part of the relative path.
                for component in relative_path.components() {
                    let component_string = component.as_os_str().to_string_lossy().to_string();
                    // Check if the file should be excluded from the tree based on the
                    // exclude patterns and exclude_from_tree arguments. Break the path component
                    // loop if it any part of the path should be excluded.
                    if exclude_from_tree && !include_file(path, include, exclude, include_priority, relative_paths, verbose)
                    {
                        break;
                    }

                    // Check if the current component already exists in the current tree.
                    current_tree = if let Some(index) = current_tree
                        .leaves
                        .iter_mut()
                        .position(|child| child.root == component_string)
                    {
                        // Update the current tree to point to the existing component.
                        &mut current_tree.leaves[index]
                    // Component doesn't already exist, create a new tree node and add to the
                    // current tree.
                    } else {
                        let new_tree = Tree::new(component_string.clone());
                        current_tree.leaves.push(new_tree);
                        current_tree.leaves.last_mut().unwrap()
                    };
                }

                if path.is_file() && include_file(path, include, exclude, include_priority, relative_paths, verbose) {
                    // Read in the file contents into bytes.
                    if let Ok(file_bytes) = fs::read(path) {
                        let code_string = String::from_utf8_lossy(&file_bytes);
                        // Get the formatted content block.
                        let formatted_block = wrap_content(
                            &code_string,
                            path.extension().and_then(|ext| ext.to_str()).unwrap_or(""),
                            line_numbers,
                            no_codeblock,
                        );

                        if !formatted_block.trim().is_empty()
                            && !formatted_block.contains(char::REPLACEMENT_CHARACTER)
                        {
                            // If the relative paths bool is True, get the relative path.
                            let file_path = if relative_paths {
                                format!("{}/{}", parent_dir, relative_path.display())
                            // If the relative paths bool is False, get the full path.
                            } else {
                                path.display().to_string()
                            };

                            files.push(json!({
                                "path": file_path,
                                "extension": path.extension().and_then(|ext| ext.to_str()).unwrap_or(""),
                                "code": formatted_block
                            }));
                        }
                    }
                }
            }
            root
        });
    Ok((tree.to_string(), files))
}

/// Gets the basename of the filepath.
///
/// ### Arguments
///
/// - `p`: The file/directory path to process.
///
/// ### Returns
///
/// - `String`: The path basename.
///
pub fn basename<P>(p: P) -> String
where
    P: AsRef<Path>,
{
    let path = p.as_ref();
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => handle_special_case(path),
    }
}

/// Handles the special cases for the basename function.
///
/// ### Arguments
///
/// - `p`: The special case path to process.
///
/// ### Returns
/// - `String`:
///
fn handle_special_case(p: &Path) -> String {
    if p.as_os_str().is_empty() || p == Path::new(".") || p == Path::new("..") {
        std::env::current_dir()
            .ok()
            .and_then(|d| d.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| ".".to_owned())
    } else {
        p.to_string_lossy().into_owned()
    }
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
fn include_file(
    path: &Path,
    include: &[String],
    exclude: &[String],
    include_priority: bool,
    relative_paths: bool,
    verbose: bool,
) -> bool {
    let canonical_root_path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(e) => {
            println!("Failed to canonicalize path: {}", e);
            return false;
        }
    };
    let path_string = canonical_root_path.to_str().unwrap();
    let relative_path = path
        .strip_prefix(std::env::current_dir().unwrap())
        .unwrap_or(path);
    let relative_path_string = relative_path.to_str().unwrap();

    if verbose {
        if relative_paths {
            println!("=> Target path: {}", relative_path_string);
        } else {
            println!("=> Target path: {}", path_string);
        }
    }

    // Function to strip "./" prefix for relative paths.
    let strip_relative_prefix: for<'a> fn(&'a str) -> &'a str = |s| s.strip_prefix("./").unwrap_or(s);

    // Check the glob patterns.
    let include_bool = include.iter().any(|pattern| {
        let matches = if relative_paths {
            let stripped_pattern = strip_relative_prefix(pattern);
            Pattern::new(stripped_pattern).unwrap().matches(relative_path_string)
        } else {
            Pattern::new(pattern).unwrap().matches(path_string)
        };
        if verbose {
            println!("\tChecking include pattern '{}': {}", pattern, matches);
        }
        matches
    });

    let exclude_bool = exclude.iter().any(|pattern| {
        let matches = if relative_paths {
            let stripped_pattern = strip_relative_prefix(pattern);
            Pattern::new(stripped_pattern).unwrap().matches(relative_path_string)
        } else {
            Pattern::new(pattern).unwrap().matches(path_string)
        };
        if verbose {
            println!("\tChecking exclude pattern '{}': {}", pattern, matches);
        }
        matches
    });

    // Determine if the file should be included.
    let result = match (include_bool, exclude_bool) {
        (true, true) => {
            if verbose {
                println!("\tMatch conflict, include priority: {}", include_priority);
            }
            include_priority
        }
        (true, false) => {
            if verbose {
                println!("\tInclude: true");
            }
            true
        }
        (false, true) => {
            if verbose {
                println!("\tInclude: false");
            }
            false
        }
        (false, false) => {
            if verbose {
                println!(
                    "\tNot in either condition, fallback: {}",
                    include.is_empty()
                );
            }
            include.is_empty()
        }
    };

    result
}

/// Wrap the file code content into a markdown code block and add line numbers if applicable.
///
/// ### Arguments
///
/// - `content`: The content to wrap.
/// - `extension`: The file extension.
/// - `line_numbers`: Whether to add line numbers.
/// - `no_codeblock`: Whether to wrap the file content or not.
///
/// ### Returns
///
/// - `String`: The formatted file content.
///
fn wrap_content(content: &str, extension: &str, line_numbers: bool, no_codeblock: bool) -> String {
    let codeblock_tick = "`".repeat(3);
    let mut formatted_block = String::new();

    if line_numbers {
        for (idx, line) in content.lines().enumerate() {
            formatted_block.push_str(&format!("{:4} | {}\n", idx + 1, line));
        }
    } else {
        formatted_block = content.to_owned();
    }

    if no_codeblock {
        formatted_block
    } else {
        format!(
            "{}{}\n{}\n{}",
            codeblock_tick, extension, formatted_block, codeblock_tick
        )
    }
}
