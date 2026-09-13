//! # codeprompt-core
//!
//! Shared configuration schema for the `codeprompt` CLI and the `codeprompt-tui`
//! wrapper. Both binaries read the same `~/.codeprompt.toml` and honor its
//! `[global]` section independently, so the file tree the TUI previews matches
//! what the CLI actually walks — with no argument pass-through between them.

pub mod profiles;

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Directory names skipped by default when `[global].ignore` is not configured.
pub const DEFAULT_IGNORE: &[&str] = &["node_modules", ".git", "venv", "__pycache__"];

/// The `[global]` section of `~/.codeprompt.toml`, honored by both binaries.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    /// Directory names to skip while walking the repository.
    pub ignore: Option<Vec<String>>,
}

impl GlobalConfig {
    /// Resolves the effective ignore list,.
    ///
    /// ### Returns
    ///
    /// - `Vec<String>`: The directory names that should be skipped.
    pub fn effective_ignore(&self) -> Vec<String> {
        match &self.ignore {
            Some(list) => list.clone(),
            None => DEFAULT_IGNORE.iter().map(|name| name.to_string()).collect(),
        }
    }
}

/// Top-level view of the config file when only `[global]` is needed.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GlobalOnly {
    global: GlobalConfig,
}

/// Returns the path to the shared config file, `~/.codeprompt.toml`.
///
/// ### Returns
///
/// - `Result<PathBuf>`: The config file path, or an error if the home
///   directory cannot be determined.
pub fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".codeprompt.toml"))
}

/// Loads the `[global]` section from `~/.codeprompt.toml`.
///
/// ### Returns
///
/// - `Result<GlobalConfig>`: The parsed `[global]` config, defaults if the file
///   is absent, or an error if the file exists but is unreadable or malformed.
pub fn load_global_config() -> Result<GlobalConfig> {
    let path = config_path()?;
    load_global_config_from(&path)
}

/// Loads the `[global]` section from a specific path.
///
/// ### Arguments
///
/// - `path`: The config file to read.
///
/// ### Returns
///
/// - `Result<GlobalConfig>`: See [`load_global_config`].
fn load_global_config_from(path: &Path) -> Result<GlobalConfig> {
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }

    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let parsed: GlobalOnly = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(parsed.global)
}

/// Returns whether a path's final component matches one of the ignored names.
///
/// ### Arguments
///
/// - `path`: The path to test.
/// - `ignore`: The effective list of ignored directory names.
///
/// ### Returns
///
/// - `bool`: True if the path should be skipped.
pub fn is_ignored(path: &Path, ignore: &[String]) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| ignore.iter().any(|ignored| ignored == name))
        .unwrap_or(false)
}
