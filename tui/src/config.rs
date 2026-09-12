use crate::gutter::GutterMode;
use anyhow::Result;
use codeprompt_core::GlobalConfig;
use serde::Deserialize;
use std::path::PathBuf;

const MAX_ESCAPE_SEQUENCE_LEN: usize = 4;

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
    /// Show an absolute line-number gutter in the list panels.
    pub line_numbers: bool,
    /// Show a relative line-number gutter.
    pub relative_line_numbers: bool,
    /// A sequence of characters that leaves the finder's insert mode.
    pub escape_sequence: Option<String>,
    /// Default option toggles (`[tui.defaults]`).
    pub defaults: OptionState,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            command: "codeprompt".to_owned(),
            template_dir: None,
            line_numbers: false,
            relative_line_numbers: false,
            escape_sequence: None,
            defaults: OptionState::default(),
        }
    }
}

impl TuiConfig {
    /// The resolved line-number gutter mode from the two config flags.
    pub fn gutter_mode(&self) -> GutterMode {
        GutterMode::from_flags(self.line_numbers, self.relative_line_numbers)
    }

    /// The validated insert-mode escape sequence.
    pub fn escape_sequence_chars(&self) -> Result<Option<Vec<char>>> {
        self.escape_sequence
            .as_deref()
            .map(validate_escape_sequence)
            .transpose()
    }
}

/// Validates the configured insert-mode escape sequence.
///
/// A valid sequence is 1 to [`MAX_ESCAPE_SEQUENCE_LEN`] characters, none
/// of which are whitespace or control characters.
fn validate_escape_sequence(sequence: &str) -> Result<Vec<char>> {
    let chars: Vec<char> = sequence.chars().collect();
    if chars.is_empty() {
        anyhow::bail!("tui.escape_sequence must not be emtpy");
    }
    if chars.len() > MAX_ESCAPE_SEQUENCE_LEN {
        anyhow::bail!(
            "tui.escape_sequence must be at most {} characters, got {}",
            MAX_ESCAPE_SEQUENCE_LEN,
            chars.len()
        );
    }
    if let Some(bad) = chars.iter().find(|c| c.is_whitespace() || c.is_control()) {
        anyhow::bail!(
            "tui.escape_sequence must not contain whitespace or control characters (found {:?})",
            bad
        );
    }
    Ok(chars)
}

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
