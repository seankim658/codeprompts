//! # CodePrompt Crate

pub mod constants {
    //! Module for the codeprompt constants.
    pub const DEFAULT_TEMPLATE_NAME: &str = "default";
    pub const CUSTOM_TEMPLATE_NAME: &str = "custom";
    pub const PROGRESS_SPINNER_TICK: u64 = 120;
}

pub mod template;
pub mod spinner;
pub mod files;

pub mod prelude {
    //! Easy import prelude module.
    pub use crate::template::{setup_handlebars_registry, get_template};
    pub use crate::spinner::{setup_spinner};
    pub use crate::files::{parse_comma_delim_patterns};
}
