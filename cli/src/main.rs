use anyhow::anyhow;
use anyhow::{Context, Error, Result};
use arboard::Clipboard;
use clap::parser::ValueSource;
use clap::{ArgAction, ArgMatches, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use codeprompt::files::prompt_for_sensitive_files;
use codeprompt::logging;
use codeprompt::prelude::*;
use codeprompt::validation::{validate_clipboard_copy, validate_token_count, ValidationConfig};
use codeprompt_core::profiles::{
    find_project_config, load_profiles, resolve, save_profile, Conflict, Profile,
};
use colored::*;
use git2::Repository;
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};

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

    /// Overwrite an existing profile without prompting (used with --write-profile).
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
        return list_profiles(args.path.as_deref());
    }

    if let Some(requested_name) = args.write_profile.clone() {
        return write_profile_command(requested_name, args.path.as_deref(), &args, &matches);
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
        let resolved = resolve_profile(&profile_name, &project_root, &args, &matches);
        apply_profile(&mut args, &resolved);
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

/// Builds a sparse [`Profile`] from only the arguments the user set explicitly.
fn overrides_from_args(args: &Args, matches: &ArgMatches) -> Profile {
    let is_set = |id: &str| matches.value_source(id) == Some(ValueSource::CommandLine);

    let bool_flag = |id: &str, value: bool| if is_set(id) { Some(value) } else { None };

    Profile {
        include: if is_set("include") {
            Some(parse_comma_delim_patterns(&args.include))
        } else {
            None
        },
        exclude: if is_set("exclude") {
            Some(parse_comma_delim_patterns(&args.exclude))
        } else {
            None
        },
        output: if is_set("output") {
            args.output.clone()
        } else {
            None
        },
        template: if is_set("template") {
            args.template.clone()
        } else {
            None
        },
        encoding: if is_set("encoding") {
            Some(args.encoding.clone())
        } else {
            None
        },
        exclude_priority: bool_flag("exclude_priority", args.exclude_priority),
        exclude_from_tree: bool_flag("exclude_from_tree", args.exclude_from_tree),
        literal_brackets: bool_flag("literal_brackets", args.literal_brackets),
        gitignore: bool_flag("gitignore", args.gitignore),
        diff_staged: bool_flag("diff_staged", args.diff_staged),
        diff_unstaged: bool_flag("diff_unstaged", args.diff_unstaged),
        no_tokens: bool_flag("no_tokens", args.no_tokens),
        no_line_numbers: bool_flag("no_line_numbers", args.no_line_numbers),
        no_codeblock: bool_flag("no_codeblock", args.no_codeblock),
        absolute_paths: bool_flag("absolute_paths", args.absolute_paths),
        no_clipboard: bool_flag("no_clipboard", args.no_clipboard),
        no_spinner: bool_flag("no_spinner", args.no_spinner),
        json: bool_flag("json", args.json),
    }
}

/// Writes a resolved profile's set fields onto the parsed arguments.
fn apply_profile(args: &mut Args, resolved: &Profile) {
    if let Some(include) = &resolved.include {
        args.include = Some(include.join(", "));
    }
    if let Some(exclude) = &resolved.exclude {
        args.exclude = Some(exclude.join(", "));
    }
    if let Some(output) = &resolved.output {
        args.output = Some(output.clone());
    }
    if let Some(template) = &resolved.template {
        args.template = Some(template.clone());
    }
    if let Some(encoding) = &resolved.encoding {
        args.encoding = encoding.clone();
    }
    if let Some(value) = resolved.exclude_priority {
        args.exclude_priority = value;
    }
    if let Some(value) = resolved.exclude_from_tree {
        args.exclude_from_tree = value;
    }
    if let Some(value) = resolved.literal_brackets {
        args.literal_brackets = value;
    }
    if let Some(value) = resolved.gitignore {
        args.gitignore = value;
    }
    if let Some(value) = resolved.diff_staged {
        args.diff_staged = value;
    }
    if let Some(value) = resolved.diff_unstaged {
        args.diff_unstaged = value;
    }
    if let Some(value) = resolved.no_tokens {
        args.no_tokens = value;
    }
    if let Some(value) = resolved.no_line_numbers {
        args.no_line_numbers = value;
    }
    if let Some(value) = resolved.no_codeblock {
        args.no_codeblock = value;
    }
    if let Some(value) = resolved.absolute_paths {
        args.absolute_paths = value;
    }
    if let Some(value) = resolved.no_clipboard {
        args.no_clipboard = value;
    }
    if let Some(value) = resolved.no_spinner {
        args.no_spinner = value;
    }
    if let Some(value) = resolved.json {
        args.json = value;
    }
}

/// Formats profile/flag conflicts into a human-readable message body.
fn format_conflicts(profile_name: &str, conflicts: &[Conflict]) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "conflicting settings between profile '{}' and command-line flags:",
        profile_name
    );
    for conflict in conflicts {
        let _ = writeln!(
            out,
            "  {:9} profile: {:30} flag: {}",
            conflict.field, conflict.profile_value, conflict.cli_value
        );
    }
    out.push_str("Remove the flag or drop it from the profile, then re-run.");
    out
}

/// Loads and resolves a saved profile against the explicit CLI flags.
fn resolve_profile(name: &str, project_root: &Path, args: &Args, matches: &ArgMatches) -> Profile {
    let config_path = match find_project_config(project_root) {
        Ok(Some(path)) => path,
        Ok(None) => exit_with_error(&format!(
            "No .codeprompt.toml found in this repository; cannot load profile '{}'.",
            name
        )),
        Err(error) => exit_with_error(&format!("Failed to locate config file: {}", error)),
    };

    let profiles = match load_profiles(&config_path) {
        Ok(profiles) => profiles,
        Err(error) => exit_with_error(&format!("{}", error)),
    };

    let profile = match profiles.get(name) {
        Some(profile) => profile,
        None => {
            let mut available: Vec<&str> = profiles.keys().map(String::as_str).collect();
            available.sort_unstable();
            let listed = if available.is_empty() {
                "none defined".to_owned()
            } else {
                available.join(", ")
            };
            exit_with_error(&format!(
                "Profile '{}' not found in {}. Available: {}",
                name,
                config_path.display(),
                listed
            ))
        }
    };

    let overrides = overrides_from_args(args, matches);
    match resolve(profile, &overrides) {
        Ok(resolved) => resolved,
        Err(conflicts) => exit_with_error(&format_conflicts(name, &conflicts)),
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!(
        "{}{}{} {}",
        "[".bold().white(),
        "!".bold().red(),
        "]".bold().white(),
        message.red()
    );
    std::process::exit(1);
}

fn list_profiles(anchor: Option<&Path>) -> Result<(), Error> {
    let anchor = match anchor {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Failed to determine current directory")?,
    };

    let config_path = match find_project_config(&anchor)? {
        Some(path) => path,
        None => {
            println!("No .codeprompt.toml found in this repository.");
            return Ok(());
        }
    };

    let profiles = load_profiles(&config_path)?;
    if profiles.is_empty() {
        println!("No profiles defined in {}.", config_path.display());
        return Ok(());
    }

    let mut names: Vec<&str> = profiles.keys().map(String::as_str).collect();
    names.sort_unstable();
    for name in names {
        println!("{}", name);
    }
    Ok(())
}

fn write_profile_command(
    requested_name: Option<String>,
    anchor: Option<&Path>,
    args: &Args,
    matches: &ArgMatches,
) -> Result<(), Error> {
    if args.profile.is_some() {
        exit_with_error("--write-profile cannot be combined with --profile.");
    }

    let overrides = overrides_from_args(args, matches);
    if overrides == Profile::default() {
        exit_with_error("No settings to save; pass some flags alongside --write-profile.");
    }

    let was_prompted = requested_name.is_none();
    let name = match requested_name {
        Some(name) => name,
        None => prompt_line("Profile name: ")?,
    };
    if !is_valid_profile_name(&name) {
        exit_with_error(&format!(
            "Invalid profile name '{}'. Use letters, digits, '-', or '_'.",
            name
        ));
    }

    let anchor = match anchor {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Failed to determine current directory")?,
    };
    let destination = project_config_destination(&anchor)?;

    if profile_exists(&destination, &name)? && !args.force {
        if was_prompted {
            if !confirm_overwrite(&name)? {
                exit_with_error("Cancelled, profile was not written.");
            }
        } else {
            exit_with_error(&format!(
                "Profile '{}' already exists. Re-run with --force to overwrite.",
                name,
            ));
        }
    }

    save_profile(&destination, &name, &overrides)?;
    println!("Saved profile '{}' to {}.", name, destination.display());
    Ok(())
}

fn project_config_destination(anchor: &Path) -> Result<PathBuf, Error> {
    if let Some(existing) = find_project_config(anchor)? {
        return Ok(existing);
    }
    let repo = Repository::discover(anchor)
        .context("Not inside a git repository; cannot determine where to write the profile")?;
    let root = repo
        .workdir()
        .context("Repository has no working directory")?;
    Ok(root.join(".codeprompt.toml"))
}

fn profile_exists(path: &Path, name: &str) -> Result<bool, Error> {
    if !path.exists() {
        return Ok(false);
    }
    Ok(load_profiles(path)?.contains_key(name))
}

fn confirm_overwrite(name: &str) -> Result<bool, Error> {
    let answer = prompt_line(&format!("Profile '{}' exists. Overwrite? [y/N]: ", name))?;
    Ok(matches!(answer.to_lowercase().as_str(), "y" | "yes"))
}

fn prompt_line(prompt: &str) -> Result<String, Error> {
    print!("{}", prompt);
    std::io::stdout()
        .flush()
        .context("Failed to flush stdout")?;
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;
    Ok(input.trim().to_owned())
}

fn is_valid_profile_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
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
