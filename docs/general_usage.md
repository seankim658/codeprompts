# General Usage

*Note*: Assumes you've setup the code prompts tool as the alias `prompt`.

The only required argument to using the tool is to provide a path to the project root.

For example, if I wanted to generate the code blocks for this entire repository I would do something like this:

```bash
cd codeprompt/
prompt .
```

And my output would look something like this: 

```bash
..... Done!
[i] Token count: 46235
[✓] Prompt successfully copied to clipboard!
```

Here's a snippet of what will be copied to your clipboard: 

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
>   41 |     /// Whether to capture the git diff for staged changes only (equivalaent to running `git diff --cached` or `git diff --staged`. Defaults to False.
>   42 |     #[arg(short = 'd', long, action(ArgAction::SetTrue))]
>   43 |     diff_staged: bool,
>   44 | 
>   45 |     /// Whether to capture the git diff for unstaged changes only (equivalaent to running `git diff`). Defaults to False.
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
 > 143 |             std::process::exit(1);
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
