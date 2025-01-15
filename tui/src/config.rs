use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

/// TUI configuration loaded from `~/.codeprompt.toml`
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Command to run
    pub command: String,
    /// Path to the template directory
    pub template_dir: Option<PathBuf>,
    /// Default options
    pub defaults: OptionState,
    /// Config status
    #[serde(skip)]
    pub config_status: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            command: "codeprompt".to_owned(),
            template_dir: None,
            defaults: OptionState::default(),
            config_status: false,
        }
    }
}

/// Default options that can be configured
#[derive(Debug, Deserialize, Clone)]
pub struct OptionState {
    pub exclude_priority: bool,
    pub exclude_from_tree: bool,
    pub diff_staged: bool,
    pub diff_unstaged: bool,
    pub gitignore: bool,
    pub no_tokens: bool,
    pub no_line_numbers: bool,
    pub no_codeblock: bool,
    pub relative_paths: bool,
    pub no_clipboard: bool,
    pub no_spinner: bool,
}

impl Default for OptionState {
    fn default() -> Self {
        Self {
            exclude_priority: false,
            exclude_from_tree: false,
            diff_staged: false,
            diff_unstaged: false,
            gitignore: true,
            no_tokens: false,
            no_line_numbers: false,
            no_codeblock: false,
            relative_paths: false,
            no_clipboard: false,
            no_spinner: false,
        }
    }
}

impl OptionState {
    pub fn options_iter_mut(&mut self) -> Vec<(&'static str, &mut bool)> {
        vec![
            ("Exclude Priority", &mut self.exclude_priority),
            ("Exclude From Tree", &mut self.exclude_from_tree),
            ("Gitignore", &mut self.gitignore),
            ("Diff Staged", &mut self.diff_staged),
            ("Diff Unstaged", &mut self.diff_unstaged),
            ("No Tokens", &mut self.no_tokens),
            ("No Line Numbers", &mut self.no_line_numbers),
            ("No Codeblock", &mut self.no_codeblock),
            ("Relative Paths", &mut self.relative_paths),
            ("No CLipboard", &mut self.no_clipboard),
            ("No Spinner", &mut self.no_spinner),
        ]
    }

    pub fn num(&mut self) -> usize {
        self.options_iter_mut().len()
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".codeprompt.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let mut config: Self = toml::from_str(&content)?;
            config.config_status = true;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}
