use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

/// Sets up debug logging when verbose mode is enabled.
pub fn setup(verbose: bool) {
    if verbose {
        let filter = EnvFilter::new("codeprompt=debug");

        fmt::fmt()
            .with_max_level(Level::DEBUG)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(false)
            .with_target(true)
            .with_ansi(true)
            .with_env_filter(filter)
            .compact()
            .init();
    }
}
