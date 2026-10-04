//! Home-directory shorthand (`~`) for paths stored in config files.
//!
//! Store paths as `~/...` so a config file copied to another machine resolves
//! against that machine's home directory.

use std::path::{Path, PathBuf};

const HOME_SHORTHAND: &str = "~";

/// Replace a leading `~` component with the current user's home directory.
pub fn expand_home(path: &Path) -> PathBuf {
    match dirs::home_dir() {
        Some(home) => expand_home_with(path, &home),
        None => path.to_path_buf(),
    }
}

/// Rewrite a path inside the current user's home directory as `~/...`.
pub fn contract_home(path: &Path) -> PathBuf {
    match dirs::home_dir() {
        Some(home) => contract_home_with(path, &home),
        None => path.to_path_buf(),
    }
}

pub fn expand_home_str(path: &str) -> String {
    into_string_or(expand_home(Path::new(path)), path)
}

pub fn contract_home_str(path: &str) -> String {
    into_string_or(contract_home(Path::new(path)), path)
}

fn expand_home_with(path: &Path, home: &Path) -> PathBuf {
    match path.strip_prefix(HOME_SHORTHAND) {
        Ok(rest) if rest.as_os_str().is_empty() => home.to_path_buf(),
        Ok(rest) => home.join(rest),
        Err(_) => path.to_path_buf(),
    }
}

/// Join components with `/` on every platform so the stored value parses on any
/// OS. Leave paths with non-UTF-8 components unchanged.
fn contract_home_with(path: &Path, home: &Path) -> PathBuf {
    let Ok(rest) = path.strip_prefix(home) else {
        return path.to_path_buf();
    };

    let mut contracted = String::from(HOME_SHORTHAND);
    for component in rest.components() {
        let Some(part) = component.as_os_str().to_str() else {
            return path.to_path_buf();
        };
        contracted.push('/');
        contracted.push_str(part);
    }
    PathBuf::from(contracted)
}

fn into_string_or(path: PathBuf, fallback: &str) -> String {
    match path.to_str() {
        Some(converted) => converted.to_owned(),
        None => fallback.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/home/alice";

    fn contract(path: &str) -> PathBuf {
        contract_home_with(Path::new(path), Path::new(HOME))
    }

    fn expand(path: &str) -> PathBuf {
        expand_home_with(Path::new(path), Path::new(HOME))
    }

    #[test]
    fn contracts_path_inside_home() {
        assert_eq!(
            contract("/home/alice/.config/codeprompt/git_commit.hbs"),
            PathBuf::from("~/.config/codeprompt/git_commit.hbs")
        );
    }

    #[test]
    fn contracts_home_itself_to_bare_tilde() {
        assert_eq!(contract("/home/alice"), PathBuf::from("~"));
    }

    #[test]
    fn leaves_path_outside_home_unchanged() {
        assert_eq!(contract("/tmp/out.md"), PathBuf::from("/tmp/out.md"));
    }

    #[test]
    fn leaves_sibling_sharing_name_prefix_unchanged() {
        assert_eq!(
            contract("/home/alice2/x.hbs"),
            PathBuf::from("/home/alice2/x.hbs")
        );
    }

    #[test]
    fn leaves_relative_path_unchanged() {
        assert_eq!(
            contract("templates/x.hbs"),
            PathBuf::from("templates/x.hbs")
        );
    }

    #[test]
    fn expands_leading_tilde() {
        assert_eq!(
            expand("~/out/dump.md"),
            PathBuf::from("/home/alice/out/dump.md")
        );
    }

    #[test]
    fn expands_bare_tilde() {
        assert_eq!(expand("~"), PathBuf::from(HOME));
    }

    #[test]
    fn leaves_other_user_tilde_unchanged() {
        assert_eq!(expand("~bob/x.hbs"), PathBuf::from("~bob/x.hbs"));
    }

    #[test]
    fn leaves_inner_tilde_unchanged() {
        assert_eq!(expand("notes/~/x.md"), PathBuf::from("notes/~/x.md"));
    }

    #[test]
    fn round_trips_through_contract_and_expand() {
        let original = "/home/alice/.config/codeprompt/git_commit.hbs";
        let stored = contract(original);
        assert_eq!(expand(stored.to_str().unwrap()), PathBuf::from(original));
    }
}
