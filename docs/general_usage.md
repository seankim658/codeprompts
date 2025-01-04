# General Usage

*Note*: Assumes the executable was built from source so it is just named `codeprompt`.

If I wanted to use the default template and generate the codeprompt for the `main.rs` file, I would do something like this:

```bash
codeprompt . --include ./src/main.rs
```

And my output would look something like this: 

```bash
..... Done!
[i] Token count: 4046
[✓] Prompt successfully copied to clipboard!
```

Here's a snippet of what will be copied to your clipboard: 

Project Path: codeprompts

Source Tree:

> ```
> codeprompts
> ├── Cargo.toml
> ├── LICENSE
> ├── completions
> │   ├── codeprompt-completions.bash
> │   ├── codeprompt-completions.ps1
> │   ├── codeprompt-completions.zsh
> │   └── codeprompt-completions.fish
> ├── Cargo.lock
> ├── docs
> │   ├── git_features.md
> │   ├── setup.md
> │   ├── README.md
> │   ├── options.md
> │   └── general_usage.md
> ├── README.md
> └── src
>     ├── lib.rs
>     ├── git.rs
>     ├── files.rs
>     ├── spinner.rs
>     ├── main.rs
>     ├── templates
>     │   ├── git_commit.hbs
>     │   ├── code_optimization.hbs
>     │   ├── git_issue.hbs
>     │   ├── default_template.hbs
>     │   └── documentation_template.hbs
>     ├── template.rs
>     └── tokenizer.rs
> 
> ```
> 
> ## Code
> 
> `codeprompts/src/main.rs`:
> 
> ```rs
>    1 | use anyhow::{Context, Error, Result};
>    2 | use arboard::Clipboard;
>    3 | use clap::{ArgAction, Command, CommandFactory, Parser, Subcommand};
>    4 | use clap_complete::{generate, Generator, Shell};
>    5 | use codeprompt::prelude::*;
>    6 | use colored::*;
>    7 | use git2::Repository;
>    8 | use serde_json::json;
>    9 | use std::io::Write;
>   10 | use std::path::PathBuf;
>   11 | 
>   12 | /// Create standardized LLM prompts from your code.
>   13 | #[derive(Parser, Debug)]
>   14 | #[clap(name = "codeprompt", version = "0.1.5")]
>   15 | struct Args {
>   16 |     /// Subcommand for shell completion generation.
>   17 |     #[command(subcommand)]
>   18 |     subcommand: Option<SubCommand>,
>   19 | 
>   20 |     /// Path to project directory.
>   21 |     #[arg()]
>   22 |     path: Option<PathBuf>,
>   23 | 
>   24 |     /// Glob patterns to include.
>   25 |     #[arg(long)]
>   26 |     include: Option<String>,
>   27 | 
>   28 |     /// Glob patterns to exclude.
>   29 |     #[arg(long)]
>   30 |     exclude: Option<String>,
>   31 | 
>   32 |     /// Change pattern priority in case of conflict to prioritize the exclusion pattern.
>   33 |     #[arg(long, action(ArgAction::SetFalse))]
>   34 |     exclude_priority: bool,
>   35 | 
>   36 |     /// Eclude files/folders from the source tree based on exclude patterns.
>   37 |     #[arg(long, action(ArgAction::SetTrue))]
>   38 |     exclude_from_tree: bool,
>   39 | 
>   40 |     /// Don't respect .gitignore file.
>   41 |     #[arg(long, action(ArgAction::SetFalse))]
>   42 |     gitignore: bool,
>   43 | 
>   44 |     /// Capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`.
>   45 |     #[arg(short = 'd', long, action(ArgAction::SetTrue))]
>   46 |     diff_staged: bool,
>   47 | 
>   48 |     /// Capture the git diff for unstaged changes only (equivalent to running `git diff`).
>   49 |     #[arg(short = 'u', long, action(ArgAction::SetTrue))]
>   50 |     diff_unstaged: bool,
>   51 | 
>   52 |     /// Don't display approximate token count of the genrated prompt.
>   53 |     #[arg(long, action(ArgAction::SetTrue))]
>   54 |     no_tokens: bool,
>   55 | 
>   56 |     /// Tokenizer to use for token count.
>   57 |     ///
>   58 |     /// Right now cl100k is the only supported tokenizer.
>   59 |     #[arg(short = 'c', long, default_value = "cl100k")]
>   60 |     encoding: String,
>   61 | 
>   62 |     /// Redirect output to file.
>   63 |     #[arg(short = 'o', long)]
>   64 |     output: Option<String>,
>   65 | 
>   66 |     /// Turn off line numbers in source code blocks.
>   67 |     #[arg(short = 'l', long, action(ArgAction::SetTrue))]
>   68 |     no_line_numbers: bool,
>   69 | 
>   70 |     /// Disable wrapping code inside markdown code blocks.
>   71 |     #[arg(long, action(ArgAction::SetTrue))]
>   72 |     no_codeblock: bool,
>   73 | 
>   74 |     /// Use relative paths instead of absolute paths, including parent directory.
>   75 |     #[arg(long, action(ArgAction::SetFalse))]
>   76 |     relative_paths: bool,
>   77 | 
>   78 |     /// Disable copying to clipboard.
>   79 |     #[arg(long, action(ArgAction::SetTrue))]
>   80 |     no_clipboard: bool,
>   81 | 
>   82 |     /// Optional path to Handlebars template.
>   83 |     #[arg(short = 't', long)]
>   84 |     template: Option<PathBuf>,
>   85 | 
>   86 |     /// Whether to render the spinner.
>   87 |     #[arg(long, action(ArgAction::SetTrue))]
>   88 |     no_spinner: bool,
>   89 | 
>   90 |     /// Whether to print the output as JSON. Defaults to False.
>   91 |     #[arg(long, action(ArgAction::SetTrue))]
>   92 |     json: bool,
>   93 | 
>   94 |     /// Fetch a specific Github issue for the repository.
>   95 |     #[arg(long)]
>   96 |     issue: Option<u32>,
>   97 | 
>   98 |     /// Run in verbose mode to investigate glob pattern matching.
>   99 |     #[arg(long, action(ArgAction::SetTrue))]
>  100 |     verbose: bool,
>  101 | }
>  102 | 
>  103 | #[derive(Subcommand, Debug)]
>  104 | enum SubCommand {
>  105 |     #[command(about = "Generate shell completion scripts.")]
>  106 |     Completion {
>  107 |         #[clap(value_enum)]
>  108 |         shell: Shell,
>  109 |     },
>  110 | }
>  111 | 
>  112 | fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
>  113 |     generate(gen, cmd, cmd.get_name().to_owned(), &mut std::io::stdout());
>  114 | }
>  115 | 
>  116 | /// Main entry point for the codeprompt application.
>  117 | ///
>  118 | /// ### Returns
>  119 | ///
>  120 | /// - `Result<(), Error>`: Ok(()) on successful execution, or an Error if any step fails.
>  121 | #[tokio::main]
>  122 | async fn main() -> Result<(), Error> {
>  123 |     let args = Args::parse();
>  124 | 
>  125 |     let project_root = match &args.subcommand {
>  126 |         Some(SubCommand::Completion { shell }) => {
>  127 |             let mut cmd = Args::command();
>  128 |             print_completions(*shell, &mut cmd);
>  129 |             return Ok(());
>  130 |         }
>  131 |         None => {
>  132 |             if let Some(project_root) = args.path {
>  133 |                 project_root
>  134 |             } else {
>  135 |                 eprintln!(
>  136 |                     "{}{}{} {}\n",
>  137 |                     "[".bold().white(),
>  138 |                     "!".bold().red(),
>  139 |                     "]".bold().white(),
>  140 |                     "Error: PATH argument is required when not using the completion subcommand."
>  141 |                         .bold()
>  142 |                         .red()
>  143 |                 );
>  144 |                 std::process::exit(1);
>  145 |             }
>  146 |         }
>  147 |     };
>  148 | 
>  149 |     let (template, template_name) = get_template(&args.template)?;
>  150 |     let handlebars = setup_handlebars_registry(&template, template_name)?;
>  151 | 
>  152 |     let spinner = if !args.no_spinner {
>  153 |         Some(setup_spinner("Building directory tree..."))
>  154 |     } else {
>  155 |         None
>  156 |     };
>  157 | 
>  158 |     let include_patterns = parse_comma_delim_patterns(&args.include);
>  159 |     let exclude_patterns = parse_comma_delim_patterns(&args.exclude);
>  160 | 
>  161 |     let tree_data = traverse_directory(
>  162 |         &project_root,
>  163 |         &include_patterns,
>  164 |         &exclude_patterns,
>  165 |         args.exclude_priority,
>  166 |         args.no_line_numbers,
>  167 |         args.relative_paths,
>  168 |         args.exclude_from_tree,
>  169 |         args.no_codeblock,
>  170 |         args.gitignore,
>  171 |         args.verbose,
>  172 |     );
>  173 | 
>  174 |     let (tree, files) = match tree_data {
>  175 |         Ok(result) => result,
>  176 |         Err(e) => {
>  177 |             if let Some(s) = &spinner {
>  178 |                 s.finish_with_message("Failed!".red().to_string());
>  179 |             }
>  180 |             eprintln!(
>  181 |                 "\n{}{}{} {}",
>  182 |                 "[".bold().white(),
>  183 |                 "!".bold().red(),
>  184 |                 "]".bold().white(),
>  185 |                 format!("Failed to traverse directories: {}", e).red()
>  186 |             );
>  187 |             std::process::exit(1);
>  188 |         }
>  189 |     };
>  190 | 
>  191 |     let repo = if args.diff_unstaged || args.diff_staged || args.issue.is_some() {
>  192 |         Some(
>  193 |             Repository::open(&project_root)
>  194 |                 .context("Failed to open the repository. Check your current working directory.")?,
>  195 |         )
>  196 |     } else {
>  197 |         None
>  198 |     };
>  199 | 
>  200 |     let git_diff_str = if args.diff_unstaged || args.diff_staged {
>  201 |         if let Some(s) = &spinner {
>  202 |             s.set_message("Generating git diff...");
>  203 |         }
>  204 |         match (args.diff_staged, args.diff_unstaged) {
>  205 |             (true, true) => repo.as_ref().map_or(
>  206 |                 Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
>  207 |                 |repo| git_diff(repo, 2))?,
>  208 |             (true, false) => repo.as_ref().map_or(
>  209 |                 Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
>  210 |                 |repo| git_diff(repo, 0))?,
>  211 |             (false, true) => repo.as_ref().map_or(
>  212 |                 Err(Error::msg("Used git diff flag but failed to open the repository. Check your current working directory.")),
>  213 |                 |repo| git_diff(repo, 1))?,
>  214 |             (_, _) => return Err(Error::msg("Error parsing git diff arguments.")),
>  215 |         }
>  216 |     } else {
>  217 |         String::new()
>  218 |     };
>  219 | 
>  220 |     if let Some(s) = &spinner {
>  221 |         s.finish_with_message("Done!".green().to_string());
>  222 |     }
>  223 | 
>  224 |     let mut json_data = json!({
>  225 |         "absolute_code_path": basename(&project_root),
>  226 |         "source_tree": tree,
>  227 |         "files": files,
>  228 |         "git_diff": git_diff_str,
>  229 |     });
>  230 | 
>  231 |     if let Some(issue_number) = args.issue {
>  232 |         if let Some(s) = &spinner {
>  233 |             s.set_message(format!("Fetching Github issue #{}...", issue_number));
>  234 |         }
>  235 |         let (owner, repo_name) = repo.as_ref().map_or(
>  236 |             Err(Error::msg("Used issue flag but failed to open the repository. Check your current working directory.")),
>  237 |             |repo| get_repo_info(repo))?;
>  238 |         match fetch_github_issue(&owner, &repo_name, issue_number).await {
>  239 |             Ok(issue) => {
>  240 |                 json_data["github_issue"] = serde_json::to_value(issue)?;
>  241 |                 if let Some(s) = &spinner {
>  242 |                     s.finish_with_message(
>  243 |                         format!("Github issue #{} fetched successfully!", issue_number)
>  244 |                             .green()
>  245 |                             .to_string(),
>  246 |                     );
>  247 |                 }
>  248 |             }
>  249 |             Err(e) => {
>  250 |                 if let Some(s) = &spinner {
>  251 |                     s.finish_with_message(
>  252 |                         format!("Failed to fetch Github issue #{}: {}", issue_number, e)
>  253 |                             .red()
>  254 |                             .to_string(),
>  255 |                     );
>  256 |                 }
>  257 |                 eprintln!(
>  258 |                     "\n{}{}{} {}",
>  259 |                     "[".bold().white(),
>  260 |                     "!".bold().red(),
>  261 |                     "]".bold().white(),
>  262 |                     format!("Failed to retrieve Github repo.").red()
>  263 |                 );
>  264 |                 std::process::exit(1);
>  265 |             }
>  266 |         }
>  267 |     }
>  268 | 
>  269 |     let rendered_output = render_template(&handlebars, template_name, &json_data)?;
>  270 | 
>  271 |     let tokens = if !args.no_tokens {
>  272 |         let bpe = tokenizer_init(&args.encoding);
>  273 |         bpe.encode_with_special_tokens(&rendered_output).len()
>  274 |     } else {
>  275 |         0
>  276 |     };
>  277 | 
>  278 |     let paths: Vec<String> = files
>  279 |         .iter()
>  280 |         .filter_map(|f| f.get("path").and_then(|p| p.as_str()).map(|s| s.to_owned()))
>  281 |         .collect();
>  282 | 
>  283 |     if args.json {
>  284 |         let json_output = json!({
>  285 |             "prompt": rendered_output,
>  286 |             "directory_name": basename(&project_root),
>  287 |             "token_count": tokens,
>  288 |             "files": paths,
>  289 |         });
>  290 |         println!("{}", serde_json::to_string_pretty(&json_output)?);
>  291 |         return Ok(());
>  292 |     } else {
>  293 |         if !args.no_tokens {
>  294 |             println!(
>  295 |                 "\n{}{}{} Token count: {}",
>  296 |                 "[".bold().white(),
>  297 |                 "i".bold().blue(),
>  298 |                 "]".bold().white(),
>  299 |                 tokens.to_string().bold().yellow()
>  300 |             );
>  301 |         }
>  302 |     }
>  303 | 
>  304 |     if !args.no_clipboard {
>  305 |         copy_to_clipboard(&rendered_output)?;
>  306 |     }
>  307 | 
>  308 |     if let Some(output_path) = &args.output {
>  309 |         write_output_file(output_path, &rendered_output)?;
>  310 |     }
>  311 | 
>  312 |     Ok(())
>  313 | }
>  314 | 
>  315 | /// Copies the output to the system clipboard.
>  316 | ///
>  317 | /// ### Arguments
>  318 | ///
>  319 | /// - `content`: The content to copy to the clipboard.
>  320 | ///
>  321 | /// ### Returns
>  322 | ///
>  323 | /// - `Result<(), anyhow::Error>`: Unit tuple on success or an anyhow error.
>  324 | ///
>  325 | fn copy_to_clipboard(content: &str) -> Result<(), Error> {
>  326 |     let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard.");
>  327 |     clipboard
>  328 |         .set_text(content.to_owned())
>  329 |         .context("Failed to copy output to clipboard.")?;
>  330 |     println!(
>  331 |         "{}{}{} {}",
>  332 |         "[".bold().white(),
>  333 |         "✓".bold().green(),
>  334 |         "]".bold().white(),
>  335 |         "Prompt successfully copied to clipboard!".green()
>  336 |     );
>  337 |     Ok(())
>  338 | }
>  339 | 
>  340 | /// Writes the output to an output file.
>  341 | ///
>  342 | /// ### Arguments
>  343 | ///
>  344 | /// - `path`: The path to the output file.
>  345 | /// - `content`: The content to write to the output file.
>  346 | ///
>  347 | /// ### Returns
>  348 | ///
>  349 | /// - `Result<(), anyhow::Error>`: Unit tuple on success or an anyhow error.
>  350 | ///
>  351 | fn write_output_file(path: &str, content: &str) -> Result<(), Error> {
>  352 |     let file = std::fs::File::create(path)?;
>  353 |     let mut writer = std::io::BufWriter::new(file);
>  354 |     write!(writer, "{}", content)?;
>  355 |     println!(
>  356 |         "{}{}{} {}",
>  357 |         "[".bold().white(),
>  358 |         "✓".bold().green(),
>  359 |         "]".bold().white(),
>  360 |         format!("Prompt successfully written to file: {}", path).green()
>  361 |     );
>  362 |     Ok(())
>  363 | }
