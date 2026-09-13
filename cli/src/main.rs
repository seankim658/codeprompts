mod profile_commands;

use anyhow::{Context, Error, Result};
use clap::{ArgAction, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use colored::*;
use git2::Repository;
use serde_json::json;
use std::path::PathBuf;

use codeprompt::files::prompt_for_sensitive_files;
use codeprompt::logging;
use codeprompt::output::{copy_to_clipboard, write_output_file};
use codeprompt::prelude::*;
use codeprompt::validation::{validate_clipboard_copy, validate_token_count, ValidationConfig};

/// Create standardized LLM prompts from your code.
#[derive(Parser, Debug)]
#[clap(name = "codeprompt", version = env!("CARGO_PKG_VERSION"))]
struct Args {
    /// Subcommand for shell completion generation.
    #[command(subcommand)]
    subcommand: Option<SubCommand>,

    /// Path to project directory.
    #[arg()]
    path: Option<PathBuf>,

    /// Run using a saved profile from the project-local `.codeprompt.toml`.
    #[arg(long)]
    profile: Option<String>,

    /// List the profiles defined in the project-local `codeprompt.toml` and exit.
    #[arg(long)]
    list_profiles: bool,

    /// Save the flags from this run as a profile, then exit. Use `--write-profile NAME`
    /// to name it, or bare `--write-profile` to be prompted
    #[arg(long, value_name = "NAME", num_args = 0..=1, require_equals = true)]
    write_profile: Option<Option<String>>,

    /// Delete a saved profile from the project-local `.codeprompt.toml`.
    #[arg(long, value_name = "NAME")]
    delete_profile: Option<String>,

    /// Show a saved profile's values and the command it would run.
    #[arg(long, value_name = "NAME")]
    show_profile: Option<String>,

    /// Skip the confirmation prompt when overwriting (--write-profile) or
    /// deleting (--delete_profile) a profile.
    #[arg(long)]
    force: bool,

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

    /// Treat `[` and `]` in include/exclude patterns as literal characters
    /// rather than glob character classes. Useful for SvelteKit/Next.js
    /// dynamic route directories like `[param]`.
    #[arg(long, action(ArgAction::SetTrue))]
    literal_brackets: bool,

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

    /// Use absolute paths instead of relative paths. Relative paths are the default.
    #[arg(long, action(ArgAction::SetTrue))]
    absolute_paths: bool,

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

    /// Ignore all warnings (sensitive files, large token counts, template warnings).
    #[arg(long, action(ArgAction::SetTrue))]
    no_warnings: bool,
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
    let matches = Args::command().get_matches();
    let mut args = match Args::from_arg_matches(&matches) {
        Ok(args) => args,
        Err(error) => error.exit(),
    };

    logging::setup(args.verbose);

    if args.list_profiles {
        return profile_commands::list_profiles(args.path.as_deref());
    }

    if let Some(requested_name) = args.write_profile.clone() {
        return profile_commands::write_profile_command(
            requested_name,
            args.path.as_deref(),
            &args,
            &matches,
        );
    }
    if let Some(name) = args.delete_profile.clone() {
        return profile_commands::delete_profile_command(&name, args.path.as_deref(), args.force);
    }
    if let Some(name) = args.show_profile.clone() {
        return profile_commands::show_profile_command(&name, args.path.as_deref());
    }

    let project_root = match &args.subcommand {
        Some(SubCommand::Completion { shell }) => {
            let mut cmd = Args::command();
            print_completions(*shell, &mut cmd);
            return Ok(());
        }
        None => {
            if let Some(project_root) = args.path.clone() {
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

    if let Some(profile_name) = args.profile.clone() {
        let resolved =
            profile_commands::resolve_profile(&profile_name, &project_root, &args, &matches);
        profile_commands::apply_profile(&mut args, &resolved);
    }

    let validation_config = ValidationConfig::new(
        args.diff_staged,
        args.diff_unstaged,
        args.issue,
        &args.template,
    );

    if let Err(error) = validation_config.validate_git_repo(&project_root) {
        eprintln!("{}", error.format());
        std::process::exit(1);
    }

    // Get other warnings
    let mut warnings = if !args.no_warnings {
        validation_config.validate()
    } else {
        Vec::new()
    };

    let (template, template_name) = get_template(&args.template)?;
    let handlebars = setup_handlebars_registry(&template, template_name)?;

    let include_patterns = parse_comma_delim_patterns(&args.include);
    let exclude_patterns = parse_comma_delim_patterns(&args.exclude);

    let global_config = codeprompt_core::load_global_config().unwrap_or_else(|error| {
        eprintln!(
            "{}{}{} {}",
            "[".bold().white(),
            "!".bold().yellow(),
            "]".bold().white(),
            format!("Ignoring config file: {}", error).yellow()
        );
        codeprompt_core::GlobalConfig::default()
    });
    let ignore_names = global_config.effective_ignore();

    let sensitive_files = check_sensitive_files(
        &project_root,
        &include_patterns,
        &exclude_patterns,
        &ignore_names,
        args.exclude_priority,
        !args.absolute_paths,
        args.gitignore,
        args.literal_brackets,
    )?;

    if !args.no_warnings
        && !sensitive_files.is_empty()
        && !prompt_for_sensitive_files(&sensitive_files)
    {
        eprintln!(
            "\n{}{}{} {}",
            "[".bold().white(),
            "!".bold().red(),
            "]".bold().white(),
            "Operation cancelled by user".red()
        );
        std::process::exit(1);
    }

    let spinner = if !args.no_spinner {
        Some(setup_spinner("Building directory tree..."))
    } else {
        None
    };

    let tree_data = traverse_directory(
        &project_root,
        &include_patterns,
        &exclude_patterns,
        &ignore_names,
        args.exclude_priority,
        args.no_line_numbers,
        !args.absolute_paths,
        args.exclude_from_tree,
        args.no_codeblock,
        args.gitignore,
        args.literal_brackets,
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

    // Add token count warning if needed
    if !args.no_clipboard && !args.no_warnings {
        if let Some(warning) = validate_token_count(tokens) {
            warnings.push(warning);
        }
    }

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

    let should_copy_to_clipboard = if args.no_clipboard {
        false
    } else if !args.no_tokens {
        validate_clipboard_copy(tokens, args.no_warnings)
    } else {
        true
    };

    if should_copy_to_clipboard {
        copy_to_clipboard(&rendered_output)?;
    } else if !args.no_clipboard {
        eprintln!(
            "{}{}{} {}",
            "[".bold().white(),
            "i".bold().blue(),
            "]".bold().white(),
            "Skipped copying to clipboard".dimmed()
        );
    }

    if let Some(output_path) = &args.output {
        if let Err(e) = write_output_file(output_path, &rendered_output) {
            eprintln!(
                "{}{}{} {}",
                "[".bold().white(),
                "!".bold().red(),
                "]".bold().white(),
                format!("Output error: {}", e).red()
            );
        }
    }

    // Print warnings if needed
    if !args.no_warnings && !warnings.is_empty() {
        for warning in warnings {
            eprintln!("{}", warning.format());
        }
    }

    Ok(())
}
