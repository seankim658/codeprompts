use colored::*;
use std::path::PathBuf;
use git2::Repository;
use std::io::{self, Write};

/// Token count threshold for warning.
const TOKEN_WARNING_THRESHOLD: usize = 30_000;

/// Token count threshold for clipboard safety prompt.
const CLIPBOARD_TOKEN_THRESHOLD: usize = 200_000;

/// Represents different types of validation warnings.
#[derive(Debug)]
pub enum ValidationWarning {
    /// Warning for when a git diff option is used without a template.
    GitDiffNoTemplate,
    /// Warning for when the git issue option is used without a template.
    IssueNoTemplate,
    /// Warning for when token count is high.
    LargeTokenCount(usize),
}

impl ValidationWarning {
    fn prefix() -> String {
        format!(
            "{}{}{} ",
            "[".bold().white(),
            "!".bold().yellow(),
            "]".bold().white()
        )
    }

    /// Formats the warning message with appropriate styling.
    pub fn format(&self) -> String {
        let prefix = Self::prefix();

        match self {
            Self::GitDiffNoTemplate => format!(
                "{}{}", 
                prefix, 
                "Git diff option used without a template. Consider using --template with some git template.".yellow()
            ),
            Self::IssueNoTemplate => format!(
                "{}{}", 
                prefix, 
                "Issue option used without a template.Consider using --template with some git template.".yellow()
            ),
            Self::LargeTokenCount(count) => format!(
                "{}{}", 
                prefix, 
                format!("Large token count ({}). You might want to consider using the --output option to write to a file instead of the clipboard", count).yellow(),
            ),
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    /// Error when using git features without a git repository
    NoGitRepo,
}

impl ValidationError {
    fn prefix() -> String {
        format!(
            "{}{}{} ",
            "[".bold().white(),
            "!".bold().red(),
            "]".bold().white()
        )
    }

    /// Formats the error message with appropriate styling.
    pub fn format(&self) -> String {
        let prefix = Self::prefix();
        match self {
            Self::NoGitRepo => format!(
                "{}{}", 
                prefix,
                "Git features used but no git repository found in current directory".red()
            )
        }
    }
}
/// Configuration options to validate.
#[derive(Debug)]
pub struct ValidationConfig<'a> {
    pub diff_staged: bool,
    pub diff_unstaged: bool,
    pub issue: Option<u32>,
    pub template: &'a Option<PathBuf>,
}

impl<'a> ValidationConfig<'a> {
    /// Constructor.
    pub fn new(diff_staged: bool, diff_unstaged: bool, issue: Option<u32>, template: &'a Option<PathBuf>) -> Self {
        Self {
            diff_staged,
            diff_unstaged,
            issue,
            template,
        }
    }

    /// Performs the validation logic.
    pub fn validate(&self) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        // Check for git diff options without template
        if (self.diff_staged || self.diff_unstaged) && self.template.is_none() {
            warnings.push(ValidationWarning::GitDiffNoTemplate);
        }

        // Check for issue option without template
        if self.issue.is_some() && self.template.is_none() {
            warnings.push(ValidationWarning::IssueNoTemplate);
        }

        warnings
    }

    /// Validates git repository presence when git features used.
    pub fn validate_git_repo(&self, path: &PathBuf) -> Result<(), ValidationError> {
        if (self.diff_staged || self.diff_unstaged || self.issue.is_some()) && !Repository::open(path).is_ok() {
            Err(ValidationError::NoGitRepo)
        } else {
            Ok(())
        }
    }
}

/// Validates token count and returns warning if above threshold.
///
/// ### Parameters
/// - `token_count`: The number of tokens to check.
///
/// ### Returns
/// - `Option<ValidationWarning>`: The warning, if applicable.
pub fn validate_token_count(token_count: usize) -> Option<ValidationWarning> {
    if token_count > TOKEN_WARNING_THRESHOLD {
        Some(ValidationWarning::LargeTokenCount(token_count))
    } else {
        None
    }
}

/// Validates whether clipboard copy should proceed based on token count.
/// Prompts user if above threshold.
///
/// ### Arguments
///
/// - `token_count`: The number of tokens in the output.
///
/// ### Returns
///
/// - `bool`: True if clipboard copy should proceed, False otherwise.
///
pub fn validate_clipboard_copy(token_count: usize) -> bool {
    if token_count > CLIPBOARD_TOKEN_THRESHOLD {
        prompt_for_large_clipboard(token_count)
    } else {
        true
    }
}

/// Prompts the user to confirm whether to copy to clipboard when output is very large.
///
/// ### Arguments
///
/// - `token_count`: The number of tokens in the output.
///
/// ### Returns
///
/// - `bool`: True if user confirms to copy, False otherwise.
///
fn prompt_for_large_clipboard(token_count: usize) -> bool {
    eprintln!(
        "\n{}{}{} {}",
        "[".bold().white(),
        "!".bold().yellow(),
        "]".bold().white(),
        "Warning: Output is very large".yellow()
    );

    eprintln!(
        "  Token count: {} (threshold: {})",
        token_count.to_string().red(),
        CLIPBOARD_TOKEN_THRESHOLD.to_string().yellow()
    );

    eprintln!(
        "\n{}",
        "Copying this much data to clipboard may cause system issues.".yellow()
    );
    eprint!("{}", "Copy to clipboard anyway? [y/N] ".bold());
    io::stdout().flush().unwrap();

    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();

    matches!(response.trim().to_lowercase().as_str(), "y" | "yes")
}
