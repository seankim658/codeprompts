//! # Template Module
//!
//! Handles the handlebars template integration including setting up the Handlebars template
//! engine, rendering the template with the data, copying the output to the clipboard, and
//! writing the output to a file.
//!
//! Right now does not support user-defined variables but this is planned in the future.
//!

use super::constants::{CUSTOM_TEMPLATE_NAME, DEFAULT_TEMPLATE_NAME};
use anyhow::{Context, Result};
use handlebars::{no_escape, Handlebars};
use std::path::PathBuf;

/// Sets up the Handlebars template engine.
///
/// ### Arguments
///
/// - `template_content`: The Handlebars template content string.
/// - `template_name`: The name of the template.
///
/// ### Returns
///
/// - `Result<Handlebars<'static>>`: The Handlebars instance.
///
pub fn setup_handlebars_registry(
    template_content: &str,
    template_name: &str,
) -> Result<Handlebars<'static>> {
    // Create the handlebars registry.
    let mut handlebars = Handlebars::new();
    // By default, Handlebars automatically escapes HTML special characters in variable values to
    // prevent potential cross-site scripting attacks. Since we are just creating formatted
    // prompts, if the variable contents are HTML we want the actual HTML, not the escaped values.
    handlebars.register_escape_fn(no_escape);
    // Add a template to the Handlebars registry. Allows for pre-compiling and storing the template
    // in memory.
    handlebars
        .register_template_string(template_name, template_content)
        .map_err(|e| anyhow::anyhow!("Failed to register the Handlebars template: {}", e))?;

    Ok(handlebars)
}

/// Retrieve the template content and name based on the user passed arguments. If no template
/// argument is passed by the user defaults to the default template.
///
/// ### Arguments
///
/// - `path`: The Option containing the PathBuf from the user passed arguments.
///
/// ### Returns
///
/// - `Result<(String, &str)>`: A tuple containing the template content and name.
///
pub fn get_template<'a>(path: &'a Option<PathBuf>) -> Result<(String, &'a str)> {
    // Grab the custom template content if provided.
    if let Some(template_path) = path {
        let content = std::fs::read_to_string(template_path)
            .context("Failed to read custom template path.")?;
        Ok((content, CUSTOM_TEMPLATE_NAME))
    // Fallback to default template.
    } else {
        // Use the include_str! macro to include the contents of the file as a string literal with a
        // `static lifetime in the code at compile time.
        Ok((
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/default_template.hbs"
            ))
            .to_owned(),
            DEFAULT_TEMPLATE_NAME,
        ))
    }
}
