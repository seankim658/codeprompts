//! # Tokenizer Module
//!
//! Handles the tokenizer functionality.

use tiktoken_rs::{cl100k_base, CoreBPE};

/// Returns the byte pair encoder to use for tokenization.
///
/// ### Arguments
///
/// - `encoding`: Specifies the encoding to use for the tokenization.
///
/// ### Returns
///
/// - `CoreBPE`: The tokenizer.
///
pub fn tokenizer_init(encoding: &str) -> CoreBPE {
    match encoding.to_lowercase().trim() {
        "cl100k" => cl100k_base().unwrap(),
        _ => cl100k_base().unwrap(),
    }
}
