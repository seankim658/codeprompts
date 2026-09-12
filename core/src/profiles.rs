//! Saved profiles: named bundles of CLI settings stored in a project-local
//! `.codeprompt.toml`. A profile is a sparse subset of the CLI arguments.
//! Every field is optional, and only the fields that differ from their
//! defaults are stored. A present field overrides the default, and absent
//! field inherits it.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const PROJECT_CONFIG_FILE: &str = ".codeprompt.toml";

/// Locates the project-local config file by walking up from `start`
/// toward the git root.
pub fn find_project_config(start: &Path) -> Result<Option<PathBuf>> {
    let start = start
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", start.display()))?;

    let mut dir = if start.is_dir() {
        start.as_path()
    } else {
        start.parent().unwrap_or(start.as_path())
    };

    loop {
        let candidate = dir.join(PROJECT_CONFIG_FILE);
        if candidate.is_file() {
            return Ok(Some(candidate));
        }

        if dir.join(".git").exists() {
            return Ok(None);
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Ok(None),
        }
    }
}

/// Loads the saved profiles from a project-local config file.
pub fn load_profiles(path: &Path) -> Result<HashMap<String, Profile>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let parsed: ProjectConfig = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    Ok(parsed.profiles)
}

pub fn save_profile(path: &Path, name: &str, profile: &Profile) -> Result<()> {
    let mut doc = if path.exists() {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        contents
            .parse::<toml_edit::DocumentMut>()
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?
    } else {
        toml_edit::DocumentMut::new()
    };

    let profiles = doc
        .entry("profiles")
        .or_insert_with(|| {
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            toml_edit::Item::Table(table)
        })
        .as_table_mut()
        .context("`profiles` in the config file is not a table")?;

    profiles.insert(name, toml_edit::Item::Table(profile_to_table(profile)?));

    std::fs::write(path, doc.to_string())
        .with_context(|| format!("Failed to write config files: {}", path.display()))?;
    Ok(())
}

fn profile_to_table(profile: &Profile) -> Result<toml_edit::Table> {
    let serialized = toml::to_string(profile).context("Failed to serialize profile")?;
    let parsed = serialized
        .parse::<toml_edit::DocumentMut>()
        .context("Failed to build profile table")?;

    let mut table = toml_edit::Table::new();
    for (key, item) in parsed.as_table().iter() {
        table.insert(key, item.clone());
    }
    Ok(table)
}

/// A value field set both in a profile and on the command line.
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    /// The field name
    pub field: &'static str,
    /// The profile's value
    pub profile_value: String,
    /// The command-line value
    pub cli_value: String,
}

/// Merges a value field, recording a conflict when both sides set it.
fn merge_value<T: Clone + std::fmt::Debug>(
    conflicts: &mut Vec<Conflict>,
    field: &'static str,
    profile: &Option<T>,
    overrides: &Option<T>,
) -> Option<T> {
    match (profile, overrides) {
        (Some(p), Some(o)) => {
            conflicts.push(Conflict {
                field,
                profile_value: format!("{:?}", p),
                cli_value: format!("{:?}", o),
            });
            None
        }
        (Some(p), None) => Some(p.clone()),
        (None, Some(o)) => Some(o.clone()),
        (None, None) => None,
    }
}

/// Resolves a saved profile against the explicitly-set command-line settings.
///
/// `overrides` is a sparse `Profile` holding only the settings the user set
/// explicitly (via CLI flags). Precedence is `overrides` over `profile`.
pub fn resolve(profile: &Profile, overrides: &Profile) -> Result<Profile, Vec<Conflict>> {
    let mut conflicts = Vec::new();

    let include = merge_value(
        &mut conflicts,
        "include",
        &profile.include,
        &overrides.include,
    );
    let exclude = merge_value(
        &mut conflicts,
        "exclude",
        &profile.exclude,
        &overrides.exclude,
    );
    let output = merge_value(&mut conflicts, "output", &profile.output, &overrides.output);
    let template = merge_value(
        &mut conflicts,
        "template",
        &profile.template,
        &overrides.template,
    );
    let encoding = merge_value(
        &mut conflicts,
        "encoding",
        &profile.encoding,
        &overrides.encoding,
    );

    if !conflicts.is_empty() {
        return Err(conflicts);
    }

    Ok(Profile {
        include,
        exclude,
        output,
        template,
        encoding,
        // Boolean overrides win when present
        exclude_priority: overrides.exclude_priority.or(profile.exclude_priority),
        exclude_from_tree: overrides.exclude_from_tree.or(profile.exclude_from_tree),
        literal_brackets: overrides.literal_brackets.or(profile.literal_brackets),
        gitignore: overrides.gitignore.or(profile.gitignore),
        diff_staged: overrides.diff_staged.or(profile.diff_staged),
        diff_unstaged: overrides.diff_unstaged.or(profile.diff_unstaged),
        no_tokens: overrides.no_tokens.or(profile.no_tokens),
        no_line_numbers: overrides.no_line_numbers.or(profile.no_line_numbers),
        no_codeblock: overrides.no_codeblock.or(profile.no_codeblock),
        absolute_paths: overrides.absolute_paths.or(profile.absolute_paths),
        no_clipboard: overrides.no_clipboard.or(profile.no_clipboard),
        no_spinner: overrides.no_spinner.or(profile.no_spinner),
        json: overrides.json.or(profile.json),
    })
}

/// A single saved profile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_priority: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_from_tree: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_brackets: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitignore: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_staged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_unstaged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_tokens: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_line_numbers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_codeblock: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_paths: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_clipboard: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_spinner: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub profiles: HashMap<String, Profile>,
}
