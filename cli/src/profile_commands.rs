use anyhow::{Context, Error, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use colored::*;
use git2::Repository;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Args;
use codeprompt::files::parse_comma_delim_patterns;
use codeprompt_core::profiles::{
    delete_profile, find_project_config, load_profiles, profile_to_flags, profile_to_toml, resolve,
    save_profile, Conflict, Profile, ProfileFlag, Scope,
};

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

/// Resolves the config file to read profiles from the correct scope.
fn scope_config_path(scope: Scope, anchor: &Path) -> Result<Option<PathBuf>, Error> {
    match scope {
        Scope::Project => find_project_config(anchor),
        Scope::Global => {
            let path = codeprompt_core::config_path()?;
            Ok(path.exists().then_some(path))
        }
    }
}

/// Resolves where to write a profile for the scope.
fn scope_write_destination(scope: Scope, anchor: &Path) -> Result<PathBuf, Error> {
    match scope {
        Scope::Project => project_config_destination(anchor),
        Scope::Global => Ok(codeprompt_core::config_path()?),
    }
}

fn try_load_scope_profiles(
    scope: Scope,
    anchor: &Path,
) -> Option<(PathBuf, HashMap<String, Profile>)> {
    let config_path = match scope_config_path(scope, anchor) {
        Ok(Some(path)) => path,
        Ok(None) => return None,
        Err(error) => exit_with_error(&format!("Failed to locate config file: {}", error)),
    };

    let profiles = match load_profiles(&config_path) {
        Ok(profiles) => profiles,
        Err(error) => exit_with_error(&format!("{}", error)),
    };

    Some((config_path, profiles))
}

fn load_scope_profiles(
    scope: Scope,
    anchor: &Path,
    action: &str,
) -> (PathBuf, HashMap<String, Profile>) {
    match try_load_scope_profiles(scope, anchor) {
        Some(result) => result,
        None => exit_with_error(&format!(
            "No {} .codeprompt.toml found, cannot {}.",
            scope.label(),
            action
        )),
    }
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

fn warn_shadowed_profile(name: &str) {
    eprintln!(
        "{}: using the project profile '{}', a global profile with the same name exists.",
        "warning".yellow().bold(),
        name
    );
}

fn profile_not_found_across_scope(
    name: &str,
    project: Option<&(PathBuf, HashMap<String, Profile>)>,
    global: Option<&(PathBuf, HashMap<String, Profile>)>,
) -> String {
    let mut available: Vec<&str> = Vec::new();
    if let Some((_, profiles)) = project {
        available.extend(profiles.keys().map(String::as_str));
    }
    if let Some((_, profiles)) = global {
        available.extend(profiles.keys().map(String::as_str));
    }
    available.sort_unstable();
    available.dedup();

    let listed = if available.is_empty() {
        "none defined".to_owned()
    } else {
        available.join(", ")
    };
    format!(
        "Profile '{}' not found in the project or global config. Available: {}",
        name, listed
    )
}

struct LocatedProfile {
    scope: Scope,
    config_path: PathBuf,
    profile: Profile,
}

fn locate_project_then_global(name: &str, anchor: &Path) -> LocatedProfile {
    let project = try_load_scope_profiles(Scope::Project, anchor);
    let global = try_load_scope_profiles(Scope::Global, anchor);

    let global_has_name = global
        .as_ref()
        .map(|(_, profiles)| profiles.contains_key(name))
        .unwrap_or(false);

    if let Some((path, profiles)) = &project {
        if let Some(profile) = profiles.get(name) {
            if global_has_name {
                warn_shadowed_profile(name);
            }
            return LocatedProfile {
                scope: Scope::Project,
                config_path: path.clone(),
                profile: profile.clone(),
            };
        }
    }

    if let Some((path, profiles)) = &global {
        if let Some(profile) = profiles.get(name) {
            return LocatedProfile {
                scope: Scope::Global,
                config_path: path.clone(),
                profile: profile.clone(),
            };
        }
    }

    exit_with_error(&profile_not_found_across_scope(
        name,
        project.as_ref(),
        global.as_ref(),
    ))
}

fn locate_profile(name: &str, scope: Scope, anchor: &Path) -> LocatedProfile {
    match scope {
        Scope::Global => {
            let (config_path, profiles) =
                load_scope_profiles(Scope::Global, anchor, &format!("load profile '{}'", name));
            match profiles.get(name) {
                Some(profile) => LocatedProfile {
                    scope: Scope::Global,
                    config_path,
                    profile: profile.clone(),
                },
                None => exit_with_error(&profile_not_found_message(name, &config_path, &profiles)),
            }
        }
        Scope::Project => locate_project_then_global(name, anchor),
    }
}

pub(crate) fn resolve_profile(
    name: &str,
    scope: Scope,
    project_root: &Path,
    args: &Args,
    matches: &ArgMatches,
) -> Profile {
    let located = locate_profile(name, scope, project_root);

    let overrides = overrides_from_args(args, matches);
    match resolve(&located.profile, &overrides) {
        Ok(resolved) => resolved,
        Err(conflicts) => exit_with_error(&format_conflicts(name, &conflicts)),
    }
}

pub(crate) fn delete_profile_command(
    name: &str,
    scope: Scope,
    anchor: Option<&Path>,
    force: bool,
) -> Result<(), Error> {
    let anchor = match anchor {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Failed to determine current directory")?,
    };

    let (config_path, profiles) =
        load_scope_profiles(scope, &anchor, &format!("delete profile '{}'", name));

    if !profiles.contains_key(name) {
        exit_with_error(&profile_not_found_message(name, &config_path, &profiles));
    }

    if !force && !confirm_delete(name)? {
        exit_with_error("Cancelled, profile was not deleted.");
    }

    delete_profile(&config_path, name)?;
    println!(
        "Deleted {} profile '{}' from {}.",
        scope.label(),
        name,
        config_path.display()
    );
    Ok(())
}

fn confirm_delete(name: &str) -> Result<bool, Error> {
    let answer = prompt_line(&format!("Delete profile '{}'? [y/N]: ", name))?;
    Ok(matches!(answer.to_lowercase().as_str(), "y" | "yes"))
}

pub(crate) fn show_profile_command(
    name: &str,
    scope: Scope,
    anchor: Option<&Path>,
) -> Result<(), Error> {
    let anchor = match anchor {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Failed to determine the current directory")?,
    };

    let located = locate_profile(name, scope, &anchor);
    let profile = &located.profile;

    let values = profile_to_toml(profile)?;
    let values = values.trim_end();
    let command = reconstruct_command(&profile_to_flags(profile));

    println!(
        "Profile '{}' ({}) in {}\n",
        name,
        located.scope.label(),
        located.config_path.display()
    );
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

fn print_profile_names(profiles: &HashMap<String, Profile>) {
    if profiles.is_empty() {
        println!("  (none)");
        return;
    }
    let mut names: Vec<&str> = profiles.keys().map(String::as_str).collect();
    names.sort_unstable();
    for name in names {
        println!("  {}", name);
    }
}

fn print_profile_section(scope: Scope, anchor: &Path) -> Result<(), Error> {
    let title = match scope {
        Scope::Global => "Global profiles",
        Scope::Project => "Project profiles",
    };

    match scope_config_path(scope, anchor)? {
        Some(path) => {
            let profiles = load_profiles(&path)?;
            println!("{} ({}):", title.bold(), path.display());
            print_profile_names(&profiles);
        }
        None => {
            println!("{}:", title.bold());
            println!("  (none)");
        }
    }
    Ok(())
}

pub(crate) fn list_profiles(scope: Scope, anchor: Option<&Path>) -> Result<(), Error> {
    let anchor = match anchor {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Failed to determine current directory")?,
    };

    match scope {
        Scope::Global => print_profile_section(Scope::Global, &anchor)?,
        Scope::Project => {
            print_profile_section(Scope::Global, &anchor)?;
            println!();
            print_profile_section(Scope::Project, &anchor)?;
        }
    }

    Ok(())
}

pub(crate) fn write_profile_command(
    requested_name: Option<String>,
    scope: Scope,
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
    let destination = scope_write_destination(scope, &anchor)?;

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
    println!(
        "Saved {} profile '{}' to {}.",
        scope.label(),
        name,
        destination.display()
    );
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
