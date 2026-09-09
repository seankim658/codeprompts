use anyhow::Result;
use codeprompt_core::GlobalConfig;
use serde::Deserialize;
use std::path::PathBuf;

/// TUI configuration loaded from `~/.codeprompt.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// TUI-specific settings (`[tui]`).
    pub tui: TuiConfig,
    /// Settings shared with the CLI (`[global]`).
    pub global: GlobalConfig,
    /// Whether a config file was found and loaded. Not read from the file.
    #[serde(skip)]
    pub config_status: bool,
}

/// The `[tui]` section: settings that only affect the TUI wrapper.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    /// Command the TUI builds and runs or submits.
    pub command: String,
    /// Directory scanned for `.hbs` template files.
    pub template_dir: Option<PathBuf>,
    /// Default option toggles (`[tui.defaults]`).
    pub defaults: OptionState,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            command: "codeprompt".to_owned(),
            template_dir: None,
            defaults: OptionState::default(),
        }
    }
}

/// Default options that can be configured
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct OptionState {
    pub exclude_priority: bool,
    pub exclude_from_tree: bool,
    pub literal_brackets: bool,
    pub diff_staged: bool,
    pub diff_unstaged: bool,
    pub gitignore: bool,
    pub no_tokens: bool,
    pub no_line_numbers: bool,
    pub no_codeblock: bool,
    pub absolute_paths: bool,
    pub no_clipboard: bool,
    pub no_spinner: bool,
}

impl Default for OptionState {
    fn default() -> Self {
        Self {
            exclude_priority: false,
            exclude_from_tree: false,
            literal_brackets: false,
            diff_staged: false,
            diff_unstaged: false,
            gitignore: true,
            no_tokens: false,
            no_line_numbers: false,
            no_codeblock: false,
            absolute_paths: false,
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
            ("Literal Brackets", &mut self.literal_brackets),
            ("Gitignore", &mut self.gitignore),
            ("Diff Staged", &mut self.diff_staged),
            ("Diff Unstaged", &mut self.diff_unstaged),
            ("No Tokens", &mut self.no_tokens),
            ("No Line Numbers", &mut self.no_line_numbers),
            ("No Codeblock", &mut self.no_codeblock),
            ("Absolute Paths", &mut self.absolute_paths),
            ("No Clipboard", &mut self.no_clipboard),
            ("No Spinner", &mut self.no_spinner),
        ]
    }

    pub fn num(&mut self) -> usize {
        self.options_iter_mut().len()
    }
}

impl Config {
    /// Loads the configuration from `~/.codeprompt.toml`.
    ///
    /// A missing file yields the default configuration with `config_status`
    /// left false.
    ///
    /// ### Returns
    ///
    /// - `Result<Self>`: The loaded configuration, or an error if the file
    ///   exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let config_path = codeprompt_core::config_path()?;

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

