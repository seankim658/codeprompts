//! Files Module
//!
//! Module that handles all file and file pathing functionality.

use anyhow::{anyhow, Result};
use colored::Colorize;
use glob::Pattern;
use ignore::WalkBuilder;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use termtree::Tree;
use tracing::debug;

const CODE_BLOCK_TICKS: &str = "```";

const SENSITIVE_FILE_PATTERNS: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_dsa",
    "*.secret",
    "secrets.yml",
    "secrets.yaml",
    "credentials.json",
];

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
/// - `exclude_priority`: Whether to give priority to the exclude patterns.
/// - `no_line_numbers`: Whether to skip adding line numbers to the code sections.
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
    exclude_priority: bool,
    no_line_numbers: bool,
    relative_paths: bool,
    exclude_from_tree: bool,
    no_codeblock: bool,
    gitignore: bool,
) -> Result<(String, Vec<serde_json::Value>)> {
    debug!(
        include_patterns = ?include,
        exclude_patterns = ?exclude,
        "Starting directory traversal"
    );

    // Will hold the files found in the traversal.
    let mut files = Vec::new();
    // Canonicalize returns the canonical, absolute form of a path with all intermediate components
    // normalized and symbolic links resolved. It errors if the path does not exist or if the final
    // component in path is not a directory.
    let canonical_root_path = root.canonicalize()?;
    let parent_dir = basename(&canonical_root_path);

    // Compile glob patterns
    let include_patterns = compile_patterns(include)?;
    let exclude_patterns = compile_patterns(exclude)?;

    let mut sensitive_files = Vec::new();

    let tree = WalkBuilder::new(&canonical_root_path)
        .standard_filters(false)
        .git_ignore(gitignore)
        .build();

    for entry in tree.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && include_file(
                path,
                &include_patterns,
                &exclude_patterns,
                exclude_priority,
                relative_paths,
            )
            && is_sensitive_file(path)
        {
            let display_path = if relative_paths {
                path.strip_prefix(std::env::current_dir().unwrap())
                    .unwrap_or(path)
                    .display()
                    .to_string()
            } else {
                path.display().to_string()
            };
            sensitive_files.push(display_path);
        }
    }

    if !sensitive_files.is_empty() && !prompt_for_sensitive_files(&sensitive_files) {
        return Err(anyhow!("Operation cancelled by user"));
    }

    // TODO : Probably a better way to handle this later instead of walking twice
    let tree = WalkBuilder::new(&canonical_root_path)
        .standard_filters(false)
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
                    if exclude_from_tree
                        && !include_file(
                            path,
                            &include_patterns,
                            &exclude_patterns,
                            exclude_priority,
                            relative_paths,
                        )
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

                if path.is_file()
                    && include_file(
                        path,
                        &include_patterns,
                        &exclude_patterns,
                        exclude_priority,
                        relative_paths,
                    )
                {
                    // Read in the file contents into bytes.
                    if let Ok(file_bytes) = fs::read(path) {
                        let code_string = String::from_utf8_lossy(&file_bytes);
                        // Get the formatted content block.
                        let formatted_block = wrap_content(
                            &code_string,
                            path.extension().and_then(|ext| ext.to_str()).unwrap_or(""),
                            no_line_numbers,
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
                                "extension": path.extension()
                                    .and_then(|ext| ext.to_str())
                                    .unwrap_or(""),
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
/// - `include_patterns`: The pre-compiled include patterns.
/// - `exclude_patterns`: The pre-compiled exclude patterns.
/// - `exclude_priority`: Whether to put precedence on the include or exclude patterns if they
/// conflict.
/// - `relative_paths`: Whether to use relative paths for each file included.
///
/// ### Returns
///
/// - `bool`: True if the file should be included, False otherwise.
///
fn include_file(
    path: &Path,
    include_patterns: &HashSet<Pattern>,
    exclude_patterns: &HashSet<Pattern>,
    exclude_priority: bool,
    relative_paths: bool,
) -> bool {
    let canonical_root_path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to canonicalize path: {}", e);
            return false;
        }
    };
    let path_string = canonical_root_path.to_str().unwrap();
    let relative_path = path
        .strip_prefix(std::env::current_dir().unwrap())
        .unwrap_or(path);
    let relative_path_string = relative_path.to_str().unwrap();

    debug!("----------------------------------------------------------------");
    debug!(
        path = if relative_paths {
            relative_path_string
        } else {
            path_string
        },
        "processing file"
    );

    // Function to strip "./" prefix for relative paths.
    let strip_relative_prefix: for<'a> fn(&'a str) -> &'a str =
        |s| s.strip_prefix("./").unwrap_or(s);

    // Check the glob patterns.
    let include_bool = if relative_paths {
        let stripped_path = strip_relative_prefix(relative_path_string);
        let matches = include_patterns
            .iter()
            .any(|pattern| pattern.matches(stripped_path));
        debug!(patterns = ?include_patterns, matches = matches, "Checking include patterns");
        matches
    } else {
        let matches = include_patterns
            .iter()
            .any(|pattern| pattern.matches(path_string));
        debug!(patterns = ?include_patterns, matches = matches, "Checking include patterns");
        matches
    };

    let exclude_bool = if relative_paths {
        let stripped_path = strip_relative_prefix(relative_path_string);
        let matches = exclude_patterns
            .iter()
            .any(|pattern| pattern.matches(stripped_path));
        debug!(patterns = ?exclude_patterns, matches = matches, "Checking exclude patterns");
        matches
    } else {
        let matches = exclude_patterns
            .iter()
            .any(|pattern| pattern.matches(path_string));
        debug!(patterns = ?exclude_patterns, matches = matches, "Checking exclude patterns");
        matches
    };

    // Determine if the file should be included.
    let result = match (include_bool, exclude_bool) {
        (true, true) => {
            debug!(
                exclude_priority = exclude_priority,
                result = if exclude_priority {
                    "excluding"
                } else {
                    "including"
                },
                "Pattern match conflict"
            );
            !exclude_priority
        }
        (true, false) => {
            debug!("File included by pattern match");
            true
        }
        (false, true) => {
            debug!("File excluded by pattern match");
            false
        }
        (false, false) => {
            debug!(
                fallback = include_patterns.is_empty(),
                "No pattern matches, using fallback"
            );
            include_patterns.is_empty()
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
/// - `no_line_numbers`: Whether to skip adding line numbers.
/// - `no_codeblock`: Whether to wrap the file content or not.
///
/// ### Returns
///
/// - `String`: The formatted file content.
///
fn wrap_content(
    content: &str,
    extension: &str,
    no_line_numbers: bool,
    no_codeblock: bool,
) -> String {
    let mut formatted_block = String::new();

    if !no_line_numbers {
        for (idx, line) in content.lines().enumerate() {
            formatted_block.push_str(&format!("{:4} | {}\n", idx + 1, line));
        }
    } else {
        formatted_block = content.to_owned();
    }

    if no_codeblock {
        formatted_block
    } else {
        let mut result = String::with_capacity(
            formatted_block.len() + CODE_BLOCK_TICKS.len() * 2 + extension.len() + 2,
        );
        result.push_str(CODE_BLOCK_TICKS);
        result.push_str(extension);
        result.push_str("\n");
        result.push_str(&formatted_block);
        result.push_str(CODE_BLOCK_TICKS);
        result
    }
}

/// Compiles glob paterns into a reusable set.
fn compile_patterns(patterns: &[String]) -> Result<HashSet<Pattern>> {
    patterns
        .iter()
        .map(|p| {
            let normalized = p.strip_prefix("./").unwrap_or(p);
            Pattern::new(normalized).map_err(|e| anyhow!("Invalid pattern {}: {}", p, e))
        })
        .collect()
}

/// Checks if a file path matches a sensitive file pattern
///
/// ### Arguments
///
/// - `path`: The file path to check
///
/// ### Returns
///
/// - `bool`: True if the file matches a sensitive pattern
///
fn is_sensitive_file(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    SENSITIVE_FILE_PATTERNS.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|p| p.matches(file_name))
            .unwrap_or(false)
    })
}

/// Prompts the user to confirm whether to continue when sensitive files are detected.
///
/// ### Arguments
///
/// - `sensitive_files`: List of sensitive file paths detected.
///
/// ### Returns
///
/// - `bool`: True if user confirms to continue, False otherwise.
///
fn prompt_for_sensitive_files(sensitive_files: &[String]) -> bool {
    eprintln!(
        "\n{}{}{} {}",
        "[".bold().white(),
        "!".bold().yellow(),
        "]".bold().white(),
        "Warning: Sensitive files detected:".yellow()
    );

    for file in sensitive_files {
        eprintln!("  - {}", file.red());
    }

    eprintln!(
        "\n{}",
        "These files may contain secrets or credentials.".yellow()
    );
    eprint!("{}", "Continue anyway? [y/N] ".bold());
    io::stdout().flush().unwrap();

    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();

    matches!(response.trim().to_lowercase().as_str(), "y" | "yes")
}
