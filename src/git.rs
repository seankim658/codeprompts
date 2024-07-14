//! Git Module
//!
//! Module that handles the Git operation functionality.

use anyhow::{Context, Error, Result};
use git2::{DiffFormat::Patch, DiffOptions, Repository};
use std::path::Path;

/// Generates a git diff in the repository.
///
/// ### Arguments
///
/// - `path`: The path to the repository.
/// - `mode`: The git diff mode to run (`0` for diff_tree_to_index, `1` for diff_index_to_workdir,
/// or `2` for both).
///
/// ### Returns
///
/// - `Result<String, anyhow::Error>`:
///
pub fn git_diff(path: &Path, mode: u8) -> Result<String, Error> {
    // Attempt to open the repository at the provided path.
    let repo = Repository::open(path).context("Failed to open the repository.")?;
    // Resolve the reference pointed at by HEAD.
    let head = repo.head().context("Failed to get the repository head.")?;
    let tree = head
        .peel_to_tree()
        .context("Failed to peel tree at head.")?;

    let mut opts = DiffOptions::new();

    let diff = match mode {
        0 => repo
            .diff_tree_to_index(Some(&tree), None, Some(&mut opts))
            .context("Failed to generate tree to index diff.")?,
        1 => repo
            .diff_index_to_workdir(None, Some(&mut opts))
            .context("Failed to generate index to workdir diff.")?,
        2 => {
            let mut diff = repo
                .diff_tree_to_index(Some(&tree), None, Some(&mut opts))
                .context("Failed to generate tree to index diff.")?;
            diff.merge(
                &repo
                    .diff_index_to_workdir(None, Some(&mut opts))
                    .context("Failed to generate index to workdir diff.")?,
            )?;
            diff
        }
        _ => {
            return Err(Error::msg(format!(
                "Invalid git diff mode. Expected 0, 1, or 2. Got {}.",
                mode
            )))
        }
    };

    // Using a Vec because Vec's grow more efficiently than Strings.
    let mut diff_text = Vec::new();
    diff.print(Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => b'+',
            '-' => b'-',
            _ => b' ',
        };
        diff_text.push(prefix);
        diff_text.extend_from_slice(line.content());
        true
    })
    .context("Failed to generate diff")?;

    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}
