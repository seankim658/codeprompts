# Git Features

Right now, the code prompt tool supports 3 options for using git features.

- [Git Diff](#git-diff)
  - [Example Output](#diff-example)
- [Git Issues](#git-issues)
  - [Example Output](#issue-example)

## Git Diff

- **Staged Changes**: You can generate the `git diff` for staged changes using the `--diff-staged` option (or `-d`). The `--diff-staged` flag is equivalent to running `git diff --staged`.
- **Unstaged Changes**: You can generate the `git diff` for unstaged changes using the `--diff-unstaged` option (or `u`). The `--diff-unstaged` flag is equivalent to running `git diff`. 
- **All Changes**: If both options are used, the diffs will be concatenated and both will be used.

A common workflow is to:

- Make your changes to the codebase.
- Run `git status` to view the changed files.
- Stage a file using `git add /file/to/stage`.
- Use the [`git_commit.hbs`](../src/templates/git_commit.hbs) template to generate the commit message for the changes in that file.
  - Ex: `prompt . --diff-staged -t <path/to/git_commit.hbs>`
- Paste output into LLM of choice and get your comprehensive commit message.

From the pre-defined templates, the diff flags can be used with the [`git_commit.hbs`](../src/templates/git_commit.hbs) and [`git_issue.hbs`](../src/templates/git_issue.hbs) templates.

### Diff Example

In this example I made some minor typo and formatting changes to the project's README. I staged the `README.md` file and used the `-d` option flag to render a git diff template for the change. These changes are very minor and its unlikely you'd need an LLM to generate the commit message for these changes but for examples sake this is sufficient.

> Project Path: codeprompt
> 
> Please generate a git commit message for the provided `git diff` output.
> 
> Source Tree:
> ```
> codeprompt
> ├── src
> │   ├── main.rs
> │   ├── git.rs
> │   ├── templates
> │   │   ├── default_template.hbs
> │   │   ├── code_optimization.hbs
> │   │   ├── git_commit.hbs
> │   │   ├── documentation_template.hbs
> │   │   └── git_issue.hbs
> │   ├── template.rs
> │   ├── lib.rs
> │   ├── tokenizer.rs
> │   ├── files.rs
> │   └── spinner.rs
> ├── README.md
> ├── docs
> │   ├── README.md
> │   ├── git_features.md
> │   ├── setup.md
> │   └── general_usage.md
> ├── Cargo.lock
> └── Cargo.toml
> 
> ```
> 
> ## Diff
> 
> Diff:
> ```
>  diff --git a/README.md b/README.md
> index a0f3870..bf86946 100644
> --- a/README.md
> +++ b/README.md
>  @@ -11,7 +11,7 @@ This was a project to brush up on Rust and is based on [code2prompt](https://git
>  - [Usage](#usage)
>    - [Arguments](#arguments)
>  - [Templates](#templates)
> -- [Notes](#notes)
> +- [Usage Guides]()
>  
>  ---
>  
>  @@ -31,6 +31,8 @@ To get the most out of LLMs, prompting has to be clear, comprehensive, and consi
>  
>  ## Installation
>  
> +To download and use the code prompts command-line tool, you have two options: you can download the release binary or compile from source.
> +
>  ### Release Binary
>  
>  To download a release binary, go to the [releases](https://github.com/seankim658/codeprompts/releases) and download the binary for your OS.
>  @@ -54,6 +56,8 @@ cargo build --release
>  
>  ## Usage
>  
> +More detailed usage guides can be found [here]().
> +
>  ### Arguments
>  
>  The code prompts command line tool has the following arguments:
>  @@ -146,7 +150,3 @@ Currently, the included pre-defined templates are:
>  | [`git_commit.hbs`](./src/templates/git_commit.hbs)                         | Template for creating a concise and accurate git commit message. Can be used with both the `diff-staged` and `diff-unstaged` options.                                      |
>  | [`git_issues.hbs`](./src/templates/git_issue.hbs)                          | Template for implementing changes based on a Github issue.                                                                                                                 |
>  | [`code_optimization.hbs`](./src/templates/code_optimization.hbs)           | Template for optimizing code in time and space complexity.                                                                                                                 |
> -
> -## Notes
> -
> -The `isssues` flag will only work for public repositories.
> 
> ```
> 
> ## Request
> 
> Make sure to thoroughly analyze the git diff output in order to understand its purpose, that is crucial to generating the highest quality commit message. The git commit should have the following attributes:
> 
> 1. Concise Subject: Short and informative subject line, less than 50 characters.
> 2. Descriptive Body: Optional summary of 1 to 2 sentences, wrapping at 72 characters.
> 3. Use Imperative Mood: For example, "Fix bug" instead of "Fixed bug" or "Fixes bug."
> 4. Capitalize the Subject: First letter of the subject should be capitalized.
> 5. No Period at the End: Subject line does not end with a period.
> 7. Separate Subject From Body With a Blank Line: If using a body, leave one blank line after the subject.
> 
> Write the content in Markdown format. Try to stick to what can be learned and inferred from the diff output itself. However, let me know if additional context is needed. The commit message should make be clear enough that an outside contributor can quickly scan the commit message and diff output and understand what happened and what was changed or fixed through that particular commit.

## Git Issues

You can include the `--issue` flag along with a specific issue number in combination with the [`git_issue.hbs`](../src/templates/git_issue.hbs) template in order to generate a prompt to implement the changes, suggestions, bug fixes, and/or requests in the specified issue ticket.  

Note that the issues flag will only work for public repositories.

### Issue Example

In this example I used issue [#9](https://github.com/seankim658/codeprompts/issues/9) on this repository by specifing the issue flag like `--issue 9`. For the example's completeness I also left the `-d` flag to show that the template can also make use of recent changes (using the `-d` or `-u` options) to provide the LLM with additional context that might be useful in implementing the issue changes.

> Project Path: codeprompt
> 
> Source Tree:
> 
> ```
> codeprompt
> ├── src
> │   ├── main.rs
> │   ├── git.rs
> │   ├── templates
> │   │   ├── default_template.hbs
> │   │   ├── code_optimization.hbs
> │   │   ├── git_commit.hbs
> │   │   ├── documentation_template.hbs
> │   │   └── git_issue.hbs
> │   ├── template.rs
> │   ├── lib.rs
> │   ├── tokenizer.rs
> │   ├── files.rs
> │   └── spinner.rs
> ├── README.md
> ├── docs
> │   ├── README.md
> │   ├── git_features.md
> │   ├── setup.md
> │   └── general_usage.md
> ├── Cargo.lock
> └── Cargo.toml
> 
> ```
> 
> ## Code
> 
> `codeprompt/src/main.rs`:
> 
> ```rs
>    1 | use anyhow::{Context, Error, Result};
>    2 | use arboard::Clipboard;
>    3 | use clap::{ArgAction, Parser};
>    4 | use codeprompt::prelude::*;
>    5 | use colored::*;
>    6 | use git2::Repository;
>    7 | use serde_json::json;
>    8 | use std::io::Write;
>    9 | use std::path::PathBuf;
>   10 | 
>   11 | /// Command-line arguments for the codeprompt application.
>   12 | #[derive(Parser, Debug)]
>   13 | #[clap(name = "codeprompt", version = "0.1.3")]
>   14 | struct Args {
>   15 |     /// Path to project directory.
>   16 |     #[arg()]
>   17 |     path: PathBuf,
>   18 | 
>   19 |     /// Glob patterns to include.
>   20 |     #[arg(long)]
>   21 |     include: Option<String>,
>   22 | 
>   23 |     /// Glob patterns to exclude.
>   24 |     #[arg(long)]
>   25 |     exclude: Option<String>,
>   26 | 
>   27 |     /// Pattern priority in case of conflict (True to prioritize Include pattern, False to
>   28 |     /// prioritize exclude pattern). Defaults to True.
>   29 |     #[arg(long, action(ArgAction::SetFalse))]
>   30 |     include_priority: bool,
>   31 | 
>   32 |     /// Whether to exclude files/folders from the source tree based on exclude patterns. Defaults
>   33 |     /// to False.
>   34 |     #[arg(long, action(ArgAction::SetTrue))]
>   35 |     exclude_from_tree: bool,
>   36 | 
>   37 |     /// Whether to respect the .gitignore file. Defaults to True.
>   38 |     #[arg(long, action(ArgAction::SetFalse))]
>   39 |     gitignore: bool,
>   40 | 
>   41 |     /// Whether to capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`. Defaults to False.
>   42 |     #[arg(short = 'd', long, action(ArgAction::SetTrue))]
>   43 |     diff_staged: bool,
>   44 | 
>   45 |     /// Whether to capture the git diff for unstaged changes only (equivalent to running `git diff`). Defaults to False.
>   46 |     #[arg(short = 'u', long, action(ArgAction::SetTrue))]
>   47 |     diff_unstaged: bool,
>   48 | 
>   49 |     /// Display approximate token count of the genrated prompt. Defaults to True.
>   50 |     #[arg(long, action(ArgAction::SetFalse))]
>   51 |     tokens: bool,
>   52 | 
>   53 |     /// Tokenizer to use for token count.
>   54 |     ///
>   55 |     /// Right now cl100k is the only supported tokenizer.
>   56 |     #[arg(short = 'c', long, default_value = "cl100k")]
>   57 |     encoding: String,
>   58 | 
>   59 |     /// Redirect output to file.
>   60 |     #[arg(short = 'o', long)]
>   61 |     output: Option<String>,
>   62 | 
>   63 |     /// Toggle line numbers to source code. Defaults to True.
>   64 |     #[arg(short = 'l', long, action(ArgAction::SetFalse))]
>   65 |     line_numbers: bool,
>   66 | 
>   67 |     /// Disable wrapping code inside markdown code blocks. Defaults to False.
>   68 |     #[arg(long, action(ArgAction::SetTrue))]
>   69 |     no_codeblock: bool,
>   70 | 
>   71 |     /// Use relative paths instead of absolute paths, including parent directory. Defaults to True.
>   72 |     #[arg(long, action(ArgAction::SetFalse))]
>   73 |     relative_paths: bool,
>   74 | 
>   75 |     /// Disable copying to clipboard. Defaults to False.
>   76 |     #[arg(long, action(ArgAction::SetTrue))]
>   77 |     no_clipboard: bool,
>   78 | 
>   79 |     /// Optional path to Handlebars template.
>   80 |     #[arg(short = 't', long)]
>   81 |     template: Option<PathBuf>,
>   82 | 
>   83 |     /// Whether to render the spinner (incurs some overhead but is nice to look at). Defaults to
>   84 |     /// True.
>   85 |     #[arg(long, action(ArgAction::SetFalse))]
>   86 |     spinner: bool,
>   87 | 
>   88 |     /// Whether to print the output as JSON. Defaults to False.
>   89 |     #[arg(long, action(ArgAction::SetTrue))]
>   90 |     json: bool,
>   91 | 
>   92 |     /// Fetch a specific Github issue for the repository.
>   93 |     #[arg(long)]
>   94 |     issue: Option<u32>,
>   95 | }
>   96 | 
>   97 | /// Main entry point for the codeprompt application.
>   98 | ///
>   99 | /// ### Returns
>  100 | ///
>  101 | /// - `Result<(), Error>`: Ok(()) on successful execution, or an Error if any step fails.
>  102 | #[tokio::main]
>  103 | async fn main() -> Result<(), Error> {
>  104 |     let args = Args::parse();
>  105 | 
>  106 |     let (template, template_name) = get_template(&args.template)?;
>  107 |     let handlebars = setup_handlebars_registry(&template, template_name)?;
>  108 | 
>  109 |     let spinner = if args.spinner {
>  110 |         Some(setup_spinner("Building directory tree..."))
>  111 |     } else {
>  112 |         None
>  113 |     };
>  114 | 
>  115 |     let include_patterns = parse_comma_delim_patterns(&args.include);
>  116 |     let exclude_patterns = parse_comma_delim_patterns(&args.exclude);
>  117 | 
>  118 |     let tree_data = traverse_directory(
>  119 |         &args.path,
>  120 |         &include_patterns,
>  121 |         &exclude_patterns,
>  122 |         args.include_priority,
>  123 |         args.line_numbers,
>  124 |         args.relative_paths,
>  125 |         args.exclude_from_tree,
>  126 |         args.no_codeblock,
>  127 |         args.gitignore,
>  128 |     );
>  129 | 
>  130 |     let (tree, files) = match tree_data {
>  131 |         Ok(result) => result,
>  132 |         Err(e) => {
>  133 |             if let Some(s) = &spinner {
>  134 |                 s.finish_with_message("Failed!".red().to_string());
>  135 |             }
>  136 |             eprint!(
>  137 |                 "\n{}{}{} {}",
>  138 |                 "[".bold().white(),
>  139 |                 "!".bold().red(),
>  140 |                 "]".bold().white(),
>  141 |                 format!("Failed to traverse directories: {}", e).red()
>  142 |             );
>  143 |             std::process::exit(1);
>  144 |         }
>  145 |     };
>  146 | 
>  147 |     let repo = if args.diff_unstaged || args.diff_staged || args.issue.is_some() {
>  148 |         Some(
>  149 |             Repository::open(&args.path)
>  150 |                 .context("Failed to open the repository. Check your current working directory.")?,
>  151 |         )
>  152 |     } else {
>  153 |         None
>  154 |     };
>  155 | 
>  156 |     let git_diff_str = if args.diff_unstaged || args.diff_staged {
>  157 |         if let Some(s) = &spinner {
>  158 |             s.set_message("Generating git diff...");
>  159 |         }
>  160 |         match (args.diff_staged, args.diff_unstaged) {
>  161 |             (true, true) => repo.as_ref().map_or(
>  162 |                 Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
>  163 |                 |repo| git_diff(repo, 2))?,
>  164 |             (true, false) => repo.as_ref().map_or(
>  165 |                 Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
>  166 |                 |repo| git_diff(repo, 0))?,
>  167 |             (false, true) => repo.as_ref().map_or(
>  168 |                 Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
>  169 |                 |repo| git_diff(repo, 1))?,
>  170 |             (_, _) => return Err(Error::msg("Error parsing git diff arguments.")),
>  171 |         }
>  172 |     } else {
>  173 |         String::new()
>  174 |     };
>  175 | 
>  176 |     if let Some(s) = &spinner {
>  177 |         s.finish_with_message("Done!".green().to_string());
>  178 |     }
>  179 | 
>  180 |     let mut json_data = json!({
>  181 |         "absolute_code_path": basename(&args.path),
>  182 |         "source_tree": tree,
>  183 |         "files": files,
>  184 |         "git_diff": git_diff_str,
>  185 |     });
>  186 | 
>  187 |     if let Some(issue_number) = args.issue {
>  188 |         if let Some(s) = &spinner {
>  189 |             s.set_message(format!("Fetching Github issue #{}...", issue_number));
>  190 |         }
>  191 |         let (owner, repo_name) = repo.as_ref().map_or(
>  192 |             Err(Error::msg("Used issue flag but failed to open the repository. Check your current working directory.")),
>  193 |             |repo| get_repo_info(repo))?;
>  194 |         match fetch_github_issue(&owner, &repo_name, issue_number).await {
>  195 |             Ok(issue) => {
>  196 |                 json_data["github_issue"] = serde_json::to_value(issue)?;
>  197 |                 if let Some(s) = &spinner {
>  198 |                     s.finish_with_message(
>  199 |                         format!("Github issue #{} fetched successfully!", issue_number)
>  200 |                             .green()
>  201 |                             .to_string(),
>  202 |                     );
>  203 |                 }
>  204 |             }
>  205 |             Err(e) => {
>  206 |                 if let Some(s) = &spinner {
>  207 |                     s.finish_with_message(
>  208 |                         format!("Failed to fetch Github issue #{}: {}", issue_number, e)
>  209 |                             .red()
>  210 |                             .to_string(),
>  211 |                     );
>  212 |                 }
>  213 |                 eprintln!(
>  214 |                     "\n{}{}{} {}",
>  215 |                     "[".bold().white(),
>  216 |                     "!".bold().red(),
>  217 |                     "]".bold().white(),
>  218 |                     format!("Failed to retrieve Github repo.").red()
>  219 |                 );
>  220 |                 std::process::exit(1);
>  221 |             }
>  222 |         }
>  223 |     }
>  224 | 
>  225 |     let rendered_output = render_template(&handlebars, template_name, &json_data)?;
>  226 | 
>  227 |     let tokens = if args.tokens {
>  228 |         let bpe = tokenizer_init(&args.encoding);
>  229 |         bpe.encode_with_special_tokens(&rendered_output).len()
>  230 |     } else {
>  231 |         0
>  232 |     };
>  233 | 
>  234 |     let paths: Vec<String> = files
>  235 |         .iter()
>  236 |         .filter_map(|f| f.get("path").and_then(|p| p.as_str()).map(|s| s.to_owned()))
>  237 |         .collect();
>  238 | 
>  239 |     if args.json {
>  240 |         let json_output = json!({
>  241 |             "prompt": rendered_output,
>  242 |             "directory_name": basename(&args.path),
>  243 |             "token_count": tokens,
>  244 |             "files": paths,
>  245 |         });
>  246 |         println!("{}", serde_json::to_string_pretty(&json_output)?);
>  247 |         return Ok(());
>  248 |     } else {
>  249 |         if args.tokens {
>  250 |             println!(
>  251 |                 "{}{}{} Token count: {}",
>  252 |                 "[".bold().white(),
>  253 |                 "i".bold().blue(),
>  254 |                 "]".bold().white(),
>  255 |                 tokens.to_string().bold().yellow()
>  256 |             );
>  257 |         }
>  258 |     }
>  259 | 
>  260 |     if !args.no_clipboard {
>  261 |         copy_to_clipboard(&rendered_output)?;
>  262 |     }
>  263 | 
>  264 |     if let Some(output_path) = &args.output {
>  265 |         write_output_file(output_path, &rendered_output)?;
>  266 |     }
>  267 | 
>  268 |     Ok(())
>  269 | }
>  270 | 
>  271 | /// Copies the output to the system clipboard.
>  272 | ///
>  273 | /// ### Arguments
>  274 | ///
>  275 | /// - `content`: The content to copy to the clipboard.
>  276 | ///
>  277 | /// ### Returns
>  278 | ///
>  279 | /// - `Result<(), anyhow::Error>`: Unit tuple on success or an anyhow error.
>  280 | ///
>  281 | fn copy_to_clipboard(content: &str) -> Result<(), Error> {
>  282 |     let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard.");
>  283 |     clipboard
>  284 |         .set_text(content.to_owned())
>  285 |         .context("Failed to copy output to clipboard.")?;
>  286 |     println!(
>  287 |         "{}{}{} {}",
>  288 |         "[".bold().white(),
>  289 |         "✓".bold().green(),
>  290 |         "]".bold().white(),
>  291 |         "Prompt successfully copied to clipboard!".green()
>  292 |     );
>  293 |     Ok(())
>  294 | }
>  295 | 
>  296 | /// Writes the output to an output file.
>  297 | ///
>  298 | /// ### Arguments
>  299 | ///
>  300 | /// - `path`: The path to the output file.
>  301 | /// - `content`: The content to write to the output file.
>  302 | ///
>  303 | /// ### Returns
>  304 | ///
>  305 | /// - `Result<(), anyhow::Error>`: Unit tuple on success or an anyhow error.
>  306 | ///
>  307 | fn write_output_file(path: &str, content: &str) -> Result<(), Error> {
>  308 |     let file = std::fs::File::create(path)?;
>  309 |     let mut writer = std::io::BufWriter::new(file);
>  310 |     write!(writer, "{}", content)?;
>  311 |     println!(
>  312 |         "{}{}{} {}",
>  313 |         "[".bold().white(),
>  314 |         "✓".bold().green(),
>  315 |         "]".bold().white(),
>  316 |         format!("Prompt successfully written to file: {}", path).green()
>  317 |     );
>  318 |     Ok(())
>  319 | }
> 
> ```
> 
> 
> ## Issue Details
> 
> ### Issue #9: Add output for conflicting flags/templates 
> - State: open
> - URL: https://github.com/seankim658/codeprompts/issues/9
> 
> - Body: Conflicting options can be easy to miss and in some cases can be difficult to figure out why your output is not as you expected. For some common cases, adding some messages indicating that the user has some conflicting option flags can be helpful.
> 
> ## Diff (Recent Changes)
> ```
>  diff --git a/README.md b/README.md
> index a0f3870..bf86946 100644
> --- a/README.md
> +++ b/README.md
>  @@ -11,7 +11,7 @@ This was a project to brush up on Rust and is based on [code2prompt](https://git
>  - [Usage](#usage)
>    - [Arguments](#arguments)
>  - [Templates](#templates)
> -- [Notes](#notes)
> +- [Usage Guides]()
>  
>  ---
>  
>  @@ -31,6 +31,8 @@ To get the most out of LLMs, prompting has to be clear, comprehensive, and consi
>  
>  ## Installation
>  
> +To download and use the code prompts command-line tool, you have two options: you can download the release binary or compile from source.
> +
>  ### Release Binary
>  
>  To download a release binary, go to the [releases](https://github.com/seankim658/codeprompts/releases) and download the binary for your OS.
>  @@ -54,6 +56,8 @@ cargo build --release
>  
>  ## Usage
>  
> +More detailed usage guides can be found [here]().
> +
>  ### Arguments
>  
>  The code prompts command line tool has the following arguments:
>  @@ -146,7 +150,3 @@ Currently, the included pre-defined templates are:
>  | [`git_commit.hbs`](./src/templates/git_commit.hbs)                         | Template for creating a concise and accurate git commit message. Can be used with both the `diff-staged` and `diff-unstaged` options.                                      |
>  | [`git_issues.hbs`](./src/templates/git_issue.hbs)                          | Template for implementing changes based on a Github issue.                                                                                                                 |
>  | [`code_optimization.hbs`](./src/templates/code_optimization.hbs)           | Template for optimizing code in time and space complexity.                                                                                                                 |
> -
> -## Notes
> -
> -The `isssues` flag will only work for public repositories.
> 
> ```
> 
> I need help with the described Github issue for my code. Based on the code and issue details please help me implement the ticket suggestions, changes, or bug reports. I've provided you with the issue number, title, state, URL, and the issue body (the issue description).
> 
> Start with providing an outlined, high level plan for what has to be done. Then go into the specifics of the code that needs to be changed and how to change it in relation to the issue information.
> 
> Try to stick to clear, readable, and good coding practices. Let me know if there is any additional context or dependencies that you need in order to implement the changes. Also let me know if you have any questions regarding the requset. Please review your work before finishing.
