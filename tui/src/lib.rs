//! # Codeprompt TUI Library
//!
//! This library provides the core functionality for the Codeprompt wrapper application. it
//! includes main application state, UI components, and configuration handling.

pub mod app;
pub mod config;
pub mod handler;
pub mod panels;
pub mod ui;

pub mod prelude {
    pub use crate::app::{ActivePanel, App};
    pub use crate::config::{Config, OptionState};
    pub use crate::handler::handle_input;
    pub use crate::panels::{
        file_tree::FileTree, options_panel::OptionsPanel, templates_panel::TemplatesPanel, Panel,
    };
    pub use crate::ui;
}
