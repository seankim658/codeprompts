use anyhow::{Context, Result};
use clap::Parser;
use codeprompt::prelude::*;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "codeprompt", version = "0.1.0", author = "seankim658")]
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
    #[arg(long, default_value_t = true)]
    include_priority: bool,

    /// Whether to exclude files/folders from the source tree based on exclude patterns. Defaults
    /// to False.
    #[arg(long, default_value_t = false)]
    exclude_from_tree: bool,

    /// Display approximate token count of the genrated prompt.
    #[arg(long, default_value_t = false)]
    tokens: bool,

    /// Tokenizer to use for token count. Defaults to cl100k.
    ///
    /// Right now cl100k is the only supported tokenizer.
    #[arg(short = 'c', long, default_value = "cl100k")]
    encoding: String,

    /// Redirect output to file.
    #[arg(short = 'o', long)]
    output: Option<String>,

    /// Toggle line numbers to source code. Defaults to True.
    #[arg(short = 'l', long, default_value_t = true)]
    line_numbers: bool,

    /// Disable wrapping code inside markdown code blocks. Defaults to False.
    #[arg(long, default_value_t = false)]
    no_codeblock: bool,

    /// Use relative paths instead of absolute paths, including parent directory. Defaults to True.
    #[arg(long, default_value_t = true)]
    relative_paths: bool,

    /// Disable copying to clipboard. Defaults to False.
    #[arg(long, default_value_t = false)]
    no_clipboard: bool,

    /// Optional path to Handlebars template.
    #[arg(short = 'p', long)]
    template: Option<PathBuf>,

    /// Whether to render the spinner (incurs some overhead but is nice to look at). Defaults to
    /// True.
    #[arg(long, default_value_t = true)]
    spinner: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Get the template to use.
    let (template, template_name) = get_template(&args.template)?;
    // Setup the Handlebars registry.
    let handlebars = setup_handlebars_registry(&template, template_name)?;

    if args.spinner {
        let spinner = setup_spinner("Building directory tree...");
    }

    let include_patterns = parse_comma_delim_patterns(&args.include);
    let exclude_patterns = parse_comma_delim_patterns(&args.exclude);

    // TODO : delete printing
    // println!("Path: {:?}", args.path);
    // println!(
    //     "Include: {}",
    //     args.include.unwrap_or("No include.".to_string())
    // );
    // println!(
    //     "Exclude: {}",
    //     args.exclude.unwrap_or("No exclude.".to_string())
    // );
    // println!("Include priority: {}", args.include_priority);
    // println!("Exclude from tree: {}", args.exclude_from_tree);
    // println!("Tokens: {}", args.tokens);
    // println!("Token encoder: {}", args.encoding);
    // println!(
    //     "Output: {}",
    //     args.output.unwrap_or("No output.".to_string())
    // );
    // println!("Line numbers: {}", args.line_numbers);
    // println!("No codeblock: {}", args.no_codeblock);
    // println!("Relative paths: {}", args.relative_paths);
    // println!("No clipboard: {}", args.no_clipboard);
    // println!("Template path: {:?}", args.template);

    Ok(())
}
