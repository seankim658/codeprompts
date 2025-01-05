use anyhow::{Context, Error, Result};
use arboard::Clipboard;
use clap::{ArgAction, Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use codeprompt::prelude::*;
use codeprompt::logging;
use colored::*;
use git2::Repository;
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

/// Create standardized LLM prompts from your code.
#[derive(Parser, Debug)]
#[clap(name = "codeprompt", version = "0.1.5")]
struct Args {
    /// Subcommand for shell completion generation.
    #[command(subcommand)]
    subcommand: Option<SubCommand>,

    /// Path to project directory.
    #[arg()]
    path: Option<PathBuf>,

    /// Glob patterns to include.
    #[arg(long)]
    include: Option<String>,

    /// Glob patterns to exclude.
    #[arg(long)]
    exclude: Option<String>,

    /// Change pattern priority in case of conflict to prioritize the exclusion pattern.
    #[arg(long, action(ArgAction::SetTrue))]
    exclude_priority: bool,

    /// Eclude files/folders from the source tree based on exclude patterns.
    #[arg(long, action(ArgAction::SetTrue))]
    exclude_from_tree: bool,

    /// Don't respect .gitignore file.
    #[arg(long, action(ArgAction::SetFalse))]
    gitignore: bool,

    /// Capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`.
    #[arg(short = 'd', long, action(ArgAction::SetTrue))]
    diff_staged: bool,

    /// Capture the git diff for unstaged changes only (equivalent to running `git diff`).
    #[arg(short = 'u', long, action(ArgAction::SetTrue))]
    diff_unstaged: bool,

    /// Don't display approximate token count of the genrated prompt.
    #[arg(long, action(ArgAction::SetTrue))]
    no_tokens: bool,

    /// Tokenizer to use for token count.
    ///
    /// Right now cl100k is the only supported tokenizer.
    #[arg(short = 'c', long, default_value = "cl100k")]
    encoding: String,

    /// Redirect output to file.
    #[arg(short = 'o', long)]
    output: Option<String>,

    /// Turn off line numbers in source code blocks.
    #[arg(short = 'l', long, action(ArgAction::SetTrue))]
    no_line_numbers: bool,

    /// Disable wrapping code inside markdown code blocks.
    #[arg(long, action(ArgAction::SetTrue))]
    no_codeblock: bool,

    /// Use relative paths instead of absolute paths, including parent directory.
    #[arg(long, action(ArgAction::SetFalse))]
    relative_paths: bool,

    /// Disable copying to clipboard.
    #[arg(long, action(ArgAction::SetTrue))]
    no_clipboard: bool,

    /// Optional path to Handlebars template.
    #[arg(short = 't', long)]
    template: Option<PathBuf>,

    /// Whether to render the spinner.
    #[arg(long, action(ArgAction::SetTrue))]
    no_spinner: bool,

    /// Whether to print the output as JSON. Defaults to False.
    #[arg(long, action(ArgAction::SetTrue))]
    json: bool,

    /// Fetch a specific Github issue for the repository.
    #[arg(long)]
    issue: Option<u32>,

    /// Run in verbose mode to investigate glob pattern matching.
    #[arg(long, action(ArgAction::SetTrue))]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    #[command(about = "Generate shell completion scripts.")]
    Completion {
        #[clap(value_enum)]
        shell: Shell,
    },
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_owned(), &mut std::io::stdout());
}

/// Main entry point for the codeprompt application.
///
/// ### Returns
///
/// - `Result<(), Error>`: Ok(()) on successful execution, or an Error if any step fails.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    logging::setup(args.verbose);

    let project_root = match &args.subcommand {
        Some(SubCommand::Completion { shell }) => {
            let mut cmd = Args::command();
            print_completions(*shell, &mut cmd);
            return Ok(());
        }
        None => {
            if let Some(project_root) = args.path {
                project_root
            } else {
                eprintln!(
                    "{}{}{} {}\n",
                    "[".bold().white(),
                    "!".bold().red(),
                    "]".bold().white(),
                    "Error: PATH argument is required when not using the completion subcommand."
                        .bold()
                        .red()
                );
                std::process::exit(1);
            }
        }
    };

    let (template, template_name) = get_template(&args.template)?;
    let handlebars = setup_handlebars_registry(&template, template_name)?;

    let spinner = if !args.no_spinner {
        Some(setup_spinner("Building directory tree..."))
    } else {
        None
    };

    let include_patterns = parse_comma_delim_patterns(&args.include);
    let exclude_patterns = parse_comma_delim_patterns(&args.exclude);

    let tree_data = traverse_directory(
        &project_root,
        &include_patterns,
        &exclude_patterns,
        args.exclude_priority,
        args.no_line_numbers,
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
            eprintln!(
                "\n{}{}{} {}",
                "[".bold().white(),
                "!".bold().red(),
                "]".bold().white(),
                format!("Failed to traverse directories: {}", e).red()
            );
            std::process::exit(1);
        }
    };

    let repo = if args.diff_unstaged || args.diff_staged || args.issue.is_some() {
        Some(
            Repository::open(&project_root)
                .context("Failed to open the repository. Check your current working directory.")?,
        )
    } else {
        None
    };

    let git_diff_str = if args.diff_unstaged || args.diff_staged {
        if let Some(s) = &spinner {
            s.set_message("Generating git diff...");
        }
        match (args.diff_staged, args.diff_unstaged) {
            (true, true) => repo.as_ref().map_or(
                Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
                |repo| git_diff(repo, 2))?,
            (true, false) => repo.as_ref().map_or(
                Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
                |repo| git_diff(repo, 0))?,
            (false, true) => repo.as_ref().map_or(
                Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
                |repo| git_diff(repo, 1))?,
            (_, _) => return Err(Error::msg("Error parsing git diff arguments.")),
        }
    } else {
        String::new()
    };

    if let Some(s) = &spinner {
        s.finish_with_message("Done!".green().to_string());
    }

    let mut json_data = json!({
        "absolute_code_path": basename(&project_root),
        "source_tree": tree,
        "files": files,
        "git_diff": git_diff_str,
    });

    if let Some(issue_number) = args.issue {
        if let Some(s) = &spinner {
            s.set_message(format!("Fetching Github issue #{}...", issue_number));
        }
        let (owner, repo_name) = repo.as_ref().map_or(
            Err(Error::msg("Used issue flag but failed to open the repository. Check your current working directory.")),
            |repo| get_repo_info(repo))?;
        match fetch_github_issue(&owner, &repo_name, issue_number).await {
            Ok(issue) => {
                json_data["github_issue"] = serde_json::to_value(issue)?;
                if let Some(s) = &spinner {
                    s.finish_with_message(
                        format!("Github issue #{} fetched successfully!", issue_number)
                            .green()
                            .to_string(),
                    );
                }
            }
            Err(e) => {
                if let Some(s) = &spinner {
                    s.finish_with_message(
                        format!("Failed to fetch Github issue #{}: {}", issue_number, e)
                            .red()
                            .to_string(),
                    );
                }
                eprintln!(
                    "\n{}{}{} {}",
                    "[".bold().white(),
                    "!".bold().red(),
                    "]".bold().white(),
                    format!("Failed to retrieve Github repo.").red()
                );
                std::process::exit(1);
            }
        }
    }

    let rendered_output = render_template(&handlebars, template_name, &json_data)?;

    let tokens = if !args.no_tokens {
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
            "directory_name": basename(&project_root),
            "token_count": tokens,
            "files": paths,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
        return Ok(());
    } else {
        if !args.no_tokens {
            println!(
                "\n{}{}{} Token count: {}",
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
