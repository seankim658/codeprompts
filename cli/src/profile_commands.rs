use anyhow::{Context, Error, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use codeprompt::files::parse_comma_delim_patterns;
use codeprompt_core::profiles::{
    delete_profile, find_project_config, load_profiles, profile_to_flags, profile_to_toml, resolve,
    save_profile, Conflict, Profile, ProfileFlag,
};
use colored::*;
use git2::Repository;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Args;

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

pub(crate) fn apply_profile(args: &mut Args, resolved: &Profile) {
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

fn load_project_profiles(project_root: &Path, action: &str) -> (PathBuf, HashMap<String, Profile>) {
    let config_path = match find_project_config(project_root) {
        Ok(Some(path)) => path,
        Ok(None) => exit_with_error(&format!(
            "No .codeprompt.toml found in this repository, cannot {}.",
            action
        )),
        Err(error) => exit_with_error(&format!("Failed to locate config file: {}", error)),
    };

    let profiles = match load_profiles(&config_path) {
        Ok(profiles) => profiles,
        Err(error) => exit_with_error(&format!("{}", error)),
    };

    (config_path, profiles)
}

fn profile_not_found_message(
    name: &str,
    config_path: &Path,
    profiles: &HashMap<String, Profile>,
) -> String {
    let mut available: Vec<&str> = profiles.keys().map(String::as_str).collect();
    available.sort_unstable();
    let listed = if available.is_empty() {
        "none defined".to_owned()
    } else {
        available.join(", ")
    };
    format!(
        "Profile '{}' not found in {}. Available: {}",
        name,
        config_path.display(),
        listed
    )
}

pub(crate) fn resolve_profile(
    name: &str,
    project_root: &Path,
    args: &Args,
    matches: &ArgMatches,
) -> Profile {
    let (config_path, profiles) =
        load_project_profiles(project_root, &format!("load profile '{}'", name));

    let profile = match profiles.get(name) {
        Some(profile) => profile,
        None => exit_with_error(&profile_not_found_message(name, &config_path, &profiles)),
    };

    let overrides = overrides_from_args(args, matches);
    match resolve(profile, &overrides) {
        Ok(resolved) => resolved,
        Err(conflicts) => exit_with_error(&format_conflicts(name, &conflicts)),
    }
}

pub(crate) fn delete_profile_command(
    name: &str,
    anchor: Option<&Path>,
    force: bool,
) -> Result<(), Error> {
    let anchor = match anchor {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Failed to determine current directory")?,
    };

    let (config_path, profiles) =
        load_project_profiles(&anchor, &format!("delete profile '{}'", name));

    if !profiles.contains_key(name) {
        exit_with_error(&profile_not_found_message(name, &config_path, &profiles));
    }

    if !force && !confirm_delete(name)? {
        exit_with_error("Cancelled, profile was not deleted.");
    }

    delete_profile(&config_path, name)?;
    println!("Deleted profile '{}' from {}.", name, config_path.display());
    Ok(())
}

fn confirm_delete(name: &str) -> Result<bool, Error> {
    let answer = prompt_line(&format!("Delete profile '{}'? [y/N]: ", name))?;
    Ok(matches!(answer.to_lowercase().as_str(), "y" | "yes"))
}

pub(crate) fn show_profile_command(name: &str, anchor: Option<&Path>) -> Result<(), Error> {
    let anchor = match anchor {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Failed to determine current directory")?,
    };

    let (config_path, profiles) =
        load_project_profiles(&anchor, &format!("show profile '{}'", name));

    let profile = match profiles.get(name) {
        Some(profile) => profile,
        None => exit_with_error(&profile_not_found_message(name, &config_path, &profiles)),
    };

    let values = profile_to_toml(profile)?;
    let values = values.trim_end();
    let command = reconstruct_command(&profile_to_flags(profile));

    println!("Profile '{}' in {}\n", name, config_path.display());
    println!("{}", "Values:".bold());
    if values.is_empty() {
        println!("(no values set)\n");
    } else {
        println!("{}\n", values);
    }
    println!("{}", "Command:".bold());
    println!("{}", command);
    Ok(())
}

fn reconstruct_command(flags: &[ProfileFlag]) -> String {
    let mut parts = vec!["codeprompt".to_owned()];
    for flag in flags {
        parts.push(flag.flag.to_owned());
        if let Some(value) = &flag.value {
            parts.push(quote_value(value));
        }
    }
    parts.push("<path>".to_owned());
    parts.join(" ")
}

fn quote_value(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\\\""))
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

pub(crate) fn list_profiles(anchor: Option<&Path>) -> Result<(), Error> {
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

pub(crate) fn write_profile_command(
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
