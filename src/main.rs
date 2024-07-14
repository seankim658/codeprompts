use anyhow::{Context, Error, Result};
use arboard::Clipboard;
use clap::{ArgAction, Parser};
use codeprompt::prelude::*;
use colored::*;
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "codeprompt", version = "0.1.0")]
struct Args {
    /// Path to project directory.
    #[arg()]
    path: PathBuf,

    /// Glob patterns to include.
    #[arg(long)]
    include: Option<String>,

    /// Glob patterns to exclude.
    #[arg(long)]
    exclude: Option<String>,

    /// Pattern priority in case of conflict (True to prioritize Include pattern, False to
    /// prioritize exclude pattern). Defaults to True.
    #[arg(long, action(ArgAction::SetFalse))]
    include_priority: bool,

    /// Whether to exclude files/folders from the source tree based on exclude patterns. Defaults
    /// to False.
    #[arg(long, action(ArgAction::SetTrue))]
    exclude_from_tree: bool,

    /// Whether to respect the .gitignore file. Defaults to True.
    #[arg(long, action(ArgAction::SetFalse))]
    gitignore: bool,

    /// Whether to capture the git diff for staged changes only (equivalaent to running `git diff --cached` or `git diff --staged`. Defaults to False.
    #[arg(short = 'd', long, action(ArgAction::SetTrue))]
    diff_staged: bool,

    /// Whether to capture the git diff for unstaged changes only (equivalaent to running `git diff`). Defaults to False.
    #[arg(short = 'u', long, action(ArgAction::SetTrue))]
    diff_unstaged: bool,

    /// Display approximate token count of the genrated prompt. Defaults to True.
    #[arg(long, action(ArgAction::SetFalse))]
    tokens: bool,

    /// Tokenizer to use for token count.
    ///
    /// Right now cl100k is the only supported tokenizer.
    #[arg(short = 'c', long, default_value = "cl100k")]
    encoding: String,

    /// Redirect output to file.
    #[arg(short = 'o', long)]
    output: Option<String>,

    /// Toggle line numbers to source code. Defaults to True.
    #[arg(short = 'l', long, action(ArgAction::SetFalse))]
    line_numbers: bool,

    /// Disable wrapping code inside markdown code blocks. Defaults to False.
    #[arg(long, action(ArgAction::SetTrue))]
    no_codeblock: bool,

    /// Use relative paths instead of absolute paths, including parent directory. Defaults to True.
    #[arg(long, action(ArgAction::SetFalse))]
    relative_paths: bool,

    /// Disable copying to clipboard. Defaults to False.
    #[arg(long, action(ArgAction::SetTrue))]
    no_clipboard: bool,

    /// Optional path to Handlebars template.
    #[arg(short = 't', long)]
    template: Option<PathBuf>,

    /// Whether to render the spinner (incurs some overhead but is nice to look at). Defaults to
    /// True.
    #[arg(long, action(ArgAction::SetFalse))]
    spinner: bool,

    /// Whether to print the output as JSON. Defaults to False.
    #[arg(long, action(ArgAction::SetTrue))]
    json: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let (template, template_name) = get_template(&args.template)?;
    let handlebars = setup_handlebars_registry(&template, template_name)?;

    let spinner = if args.spinner {
        Some(setup_spinner("Building directory tree..."))
    } else {
        None
    };

    let include_patterns = parse_comma_delim_patterns(&args.include);
    let exclude_patterns = parse_comma_delim_patterns(&args.exclude);

    let tree_data = traverse_directory(
        &args.path,
        &include_patterns,
        &exclude_patterns,
        args.include_priority,
        args.line_numbers,
        args.relative_paths,
        args.exclude_from_tree,
        args.no_codeblock,
        args.gitignore,
    );

    let (tree, files) = match tree_data {
        Ok(result) => result,
        Err(e) => {
            if let Some(s) = &spinner {
                s.finish_with_message("Failed!".red().to_string());
            }
            eprint!(
                "{}{}{} {}",
                "[".bold().white(),
                "!".bold().red(),
                "]".bold().white(),
                format!("Failed to traverse directories: {}", e).red()
            );
            std::process::exit(1);
        }
    };

    let git_diff_str = if args.diff_unstaged || args.diff_staged {
        if let Some(s) = &spinner {
            s.set_message("Generating git diff...");
        }
        match (args.diff_staged, args.diff_unstaged) {
            (true, true) => git_diff(&args.path, 2)?,
            (true, false) => git_diff(&args.path, 0)?,
            (false, true) => git_diff(&args.path, 1)?,
            (_, _) => return Err(Error::msg("Error parsing git diff arguments.")),
        }
    } else {
        String::new()
    };

    if let Some(s) = &spinner {
        s.finish_with_message("Done!".green().to_string());
    }

    let json_data = json!({
        "absolute_code_path": basename(&args.path),
        "source_tree": tree,
        "files": files,
        "git_diff": git_diff_str,
    });

    let rendered_output = render_template(&handlebars, template_name, &json_data)?;

    let tokens = if args.tokens {
        let bpe = tokenizer_init(&args.encoding);
        bpe.encode_with_special_tokens(&rendered_output).len()
    } else {
        0
    };

    let paths: Vec<String> = files
        .iter()
        .filter_map(|f| f.get("path").and_then(|p| p.as_str()).map(|s| s.to_owned()))
        .collect();

    if args.json {
        let json_output = json!({
            "prompt": rendered_output,
            "directory_name": basename(&args.path),
            "token_count": tokens,
            "files": paths,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
        return Ok(());
    } else {
        if args.tokens {
            println!(
                "{}{}{} Token count: {}",
                "[".bold().white(),
                "i".bold().blue(),
                "]".bold().white(),
                tokens.to_string().bold().yellow()
            );
        }
    }

    if !args.no_clipboard {
        copy_to_clipboard(&rendered_output)?;
    }

    if let Some(output_path) = &args.output {
        write_output_file(output_path, &rendered_output)?;
    }

    Ok(())
}

/// Copies the output to the system clipboard.
///
/// ### Arguments
///
/// - `content`: The content to copy to the clipboard.
///
/// ### Returns
///
/// - `Result<(), anyhow::Error>`: Unit tuple on success or an anyhow error.
///
fn copy_to_clipboard(content: &str) -> Result<(), Error> {
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

/// Writes the output to an output file.
///
/// ### Arguments
///
/// - `path`: The path to the output file.
/// - `content`: The content to write to the output file.
///
/// ### Returns
///
/// - `Result<(), anyhow::Error>`: Unit tuple on success or an anyhow error.
///
fn write_output_file(path: &str, content: &str) -> Result<(), Error> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    write!(writer, "{}", content)?;
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        format!("Prompt successfully written to file: {}", path).green()
    );
    Ok(())
}
