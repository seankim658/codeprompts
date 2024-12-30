use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the TUI application
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub template_dir: Option<PathBuf>,
    pub defaults: Options,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Options {
    pub include_priority: bool,
    pub exclude_from_tree: bool,
    pub gitignore: bool,
    pub diff_staged: bool,
    pub diff_unstaged: bool,
    pub tokens: bool,
    pub encoding: String,
    pub line_numbers: bool,
    pub no_codeblock: bool,
    pub relative_paths: bool,
    pub no_clipboard: bool,
    pub spinner: bool,
    pub json: bool,
    pub verbose: bool,
    pub template: Option<String>,
    pub output: Option<String>,
    pub issue: Option<usize>,
}

impl Config {
    /// Try to load config from `~/.codeprompt.toml`
    pub fn load() -> Result<Option<Self>> {
        let config_path = home_dir().map(|mut path| {
            path.push(".codeprompt.toml");
            path
        });

        match config_path {
            Some(path) if path.exists() => {
                let contents = std::fs::read_to_string(path)?;
                Ok(Some(toml::from_str(&contents)?))
            }
            _ => Ok(None),
        }
    }

    /// Get available templates from configured template directory
    pub fn get_available_templates(&self) -> Result<Vec<String>> {
        let mut templates = Vec::new();
        if let Some(template_dir) = &self.template_dir {
            for entry in std::fs::read_dir(template_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "hbs") {
                    if let Some(name) = path.file_stem() {
                        templates.push(name.to_string_lossy().into_owned());
                    }
                }
            }
        }

        Ok(templates)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            include_priority: true,
            exclude_from_tree: false,
            gitignore: true,
            diff_staged: false,
            diff_unstaged: false,
            tokens: true,
            encoding: "cl100k".to_owned(),
            line_numbers: true,
            no_codeblock: false,
            relative_paths: true,
            no_clipboard: false,
            spinner: true,
            json: false,
            verbose: false,
            template: None,
            output: None,
            issue: None,
        }
    }
}
