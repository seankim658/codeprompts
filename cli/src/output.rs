use anyhow::{anyhow, Context, Error, Result};
use arboard::Clipboard;
use colored::*;
use std::io::Write;


pub fn copy_to_clipboard(content: &str) -> Result<(), Error> {
    let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard.");
    clipboard
        .set_text(content.to_owned())
        .context("Failed to copy output to clipboard.")?;
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Prompt successfully copied to clipboard!".green()
    );
    Ok(())
}

pub fn write_output_file(path: &str, content: &str) -> Result<(), Error> {
    let path_obj = std::path::Path::new(path);
    if let Some(parent) = path_obj.parent() {
        if !parent.exists() {
            return Err(anyhow!(
                "Output directory '{}' does not exist",
                parent.display()
            ));
        }
    }

    let file = std::fs::File::create(path)
        .with_context(|| format!("Failed to create output file: {}", path))?;
    let mut writer = std::io::BufWriter::new(file);

    write!(writer, "{}", content)
        .with_context(|| format!("Failed to write to output file: {}", path))?;

    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        format!("Prompt successfully written to file: {}", path).green()
    );
    Ok(())
}
