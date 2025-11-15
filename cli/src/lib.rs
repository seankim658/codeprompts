//! # CodePrompt Crate

pub mod constants {
    //! Module for the codeprompt constants.
    pub const DEFAULT_TEMPLATE_NAME: &str = "default";
    pub const CUSTOM_TEMPLATE_NAME: &str = "custom";
    pub const PROGRESS_SPINNER_TICK: u64 = 120;
}

pub mod files;
pub mod git;
pub mod logging;
pub mod spinner;
pub mod template;
pub mod tokenizer;
pub mod validation;

pub mod prelude {
    //! Easy import prelude module.
    pub use crate::files::{
        basename, check_sensitive_files, parse_comma_delim_patterns, traverse_directory,
    };
    pub use crate::git::{fetch_github_issue, get_repo_info, git_diff};
    pub use crate::spinner::setup_spinner;
    pub use crate::template::{get_template, render_template, setup_handlebars_registry};
    pub use crate::tokenizer::tokenizer_init;
}
