//! Git Module
//!
//! Module that handles the Git operation functionality.

use anyhow::{anyhow, Context, Error, Result};
use git2::{DiffFormat::Patch, DiffOptions, Repository};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Struct to represent a parsed Github issue.
#[derive(Debug, Deserialize, Serialize)]
pub struct Issue {
    /// The github issue number.
    pub number: u32,
    /// The github issue title.
    pub title: String,
    /// The github issue body.
    pub body: Option<String>,
    /// The github issue state.
    pub state: String,
    /// the github issue raw HTML url.
    pub html_url: String,
}

/// Generates a git diff in the repository.
///
/// ### Arguments
///
/// - `path`: The path to the repository.
/// - `mode`: The git diff mode to run:
///   - `0`: for diff_tree_to_index (staged changes)
///   - `1`: for diff_index_to_workdir (unstaged changes)
///   - `2`: for both staged and unstaged changes
///
/// ### Returns
///
/// - `Result<String, anyhow::Error>`: A string containing the git diff on success, or an Error if
/// the diff generation fails.
///
pub fn git_diff(repo: &Repository, mode: u8) -> Result<String, Error> {
    // Resolve the reference pointed at by HEAD.
    let head = repo.head().context("Failed to get the repository head.")?;
    let tree = head
        .peel_to_tree()
        .context("Failed to peel tree at head.")?;

    let mut opts = DiffOptions::new();

    let diff = match mode {
        0 => repo
            .diff_tree_to_index(Some(&tree), None, Some(&mut opts))
            .context("Failed to generate tree to index diff.")?,
        1 => repo
            .diff_index_to_workdir(None, Some(&mut opts))
            .context("Failed to generate index to workdir diff.")?,
        2 => {
            let mut diff = repo
                .diff_tree_to_index(Some(&tree), None, Some(&mut opts))
                .context("Failed to generate tree to index diff.")?;
            diff.merge(
                &repo
                    .diff_index_to_workdir(None, Some(&mut opts))
                    .context("Failed to generate index to workdir diff.")?,
            )?;
            diff
        }
        _ => {
            return Err(Error::msg(format!(
                "Invalid git diff mode. Expected 0, 1, or 2. Got {}.",
                mode
            )))
        }
    };

    // Using a Vec because Vec's grow more efficiently than Strings.
    let mut diff_text = Vec::new();
    diff.print(Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => b'+',
            '-' => b'-',
            _ => b' ',
        };
        diff_text.push(prefix);
        diff_text.extend_from_slice(line.content());
        true
    })
    .context("Failed to generate diff")?;

    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}

/// Extracts the owner and repository name from a Git repository.
///
/// Attempts to parse the remote URL of the repository and extract the owner (username or
/// organization) and the repository name.
///
/// ### Arguments
///
/// - `repo`: The Git repository to extract information from.
///
/// ### Returns
///
/// - `Result<(String, String), Error>`: A tuple containing the owner and repository name on
/// success, or an Error if the information cannot be extracted.
///
pub fn get_repo_info(repo: &Repository) -> Result<(String, String), Error> {
    let remote = repo
        .find_remote("origin")
        .context("Failed to find `origin` remote.")?;
    let remote_url = remote.url().context("Failed to get remote URL")?;
    let url = if remote_url.contains(".git") {
        &remote_url.replace(".git", "")
    } else {
        remote_url
    };

    let re = Regex::new(r"github\.com[:/]([^/]+)/([^/]+)(?:\.git)?$").unwrap();
    re.captures(url)
        .and_then(|cap| {
            Some((
                cap.get(1).unwrap().as_str().to_owned(),
                cap.get(2).unwrap().as_str().to_owned(),
            ))
        })
        .context("Unable to parse Github repository information.")
}

/// Fetches a Github issue using the Github API.
///
/// ### Arguments
///
/// - `owner`: The owner (username or organization) of the repository.
/// - `repo`: The name of the repository.
/// - `issue_number`: The number of the issue to fetch.
///
/// ### Returns
///
/// - `Result<Issue, Error>`: An Issue struct containing the fetched issue details on success, or
/// an Error if the API request fails or the response cannot be parsed.
///
/// ### Errors
///
/// This function will return an error if:
/// - The API request fails (e.g., network issues, authentication problems)
/// - The API response indicates a non-success status code
/// - The API response cannot be deserialized into the Issue struct
///
pub async fn fetch_github_issue(
    owner: &str,
    repo: &str,
    issue_number: u32,
) -> Result<Issue, Error> {
    let client = Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues/{}",
        owner, repo, issue_number
    );

    let response = client
        .get(&url)
        .header("User-Agent", "codeprompt")
        .send()
        .await
        .context("Failed to send issue request to Github API")?;

    if response.status().is_success() {
        let issue: Issue = response
            .json()
            .await
            .context("Failed to parse Github issue API response.")?;
        Ok(issue)
    } else {
        Err(anyhow!(
            "Failed to fetch issue, status: {}, message: {}",
            response.status(),
            response.text().await.unwrap()
        ))
    }
}
