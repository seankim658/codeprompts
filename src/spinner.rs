//! # Spinner Module
//!
//! Module that handles the spinner (progress indicator).
//!

use super::constants::PROGRESS_SPINNER_TICK;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates the progress indicator.
///
/// ### Arguments
///
/// - `message`: A message to display with the progress indicator.
///
/// ### Returns
/// - `ProgressBar`: The configured progress bar.
///
pub fn setup_spinner(message: &str) -> ProgressBar {
    // Setup the initial spinner.
    let spinner = ProgressBar::new_spinner();
    // Spawns a background thread to tick the progress bar.
    spinner.enable_steady_tick(Duration::from_millis(PROGRESS_SPINNER_TICK));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[".", "..", "...", "....", "....."])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_owned());
    spinner
}
