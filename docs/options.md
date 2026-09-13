# Usage

```
Usage: codeprompt [OPTIONS] [PATH] [COMMAND]

Commands:
  completion  Generate shell completion scripts.
  help        Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]
          Path to project directory

Options:
      --profile <PROFILE>
          Run using a saved profile from the project-local `.codeprompt.toml`

      --list-profiles
          List the profiles defined in the project-local `codeprompt.toml` and exit

      --write-profile[=<NAME>]
          Save the flags from this run as a profile, then exit. Use `--write-profile NAME` to name it, or bare `--write-profile` to be prompted

      --delete-profile <NAME>
          Delete a saved profile from the project-local `.codeprompt.toml`

      --show-profile <NAME>
          Show a saved profile's values and the command it would run

      --force
          Skip the confirmation prompt when overwriting (--write-profile) or deleting (--delete_profile) a profile

      --global
          Operate on global profiles instead of project-local ones

      --include <INCLUDE>
          Glob patterns to include

      --exclude <EXCLUDE>
          Glob patterns to exclude

      --exclude-priority
          Change pattern priority in case of conflict to prioritize the exclusion pattern

      --exclude-from-tree
          Eclude files/folders from the source tree based on exclude patterns

      --literal-brackets
          Treat `[` and `]` in include/exclude patterns as literal characters rather than glob character classes. Useful for SvelteKit/Next.js dynamic route directories like `[param]`

      --gitignore
          Don't respect .gitignore file

  -d, --diff-staged
          Capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`

  -u, --diff-unstaged
          Capture the git diff for unstaged changes only (equivalent to running `git diff`)

      --no-tokens
          Don't display approximate token count of the genrated prompt

  -c, --encoding <ENCODING>
          Tokenizer to use for token count.

          Right now cl100k is the only supported tokenizer.

          [default: cl100k]

  -o, --output <OUTPUT>
          Redirect output to file

  -l, --no-line-numbers
          Turn off line numbers in source code blocks

      --no-codeblock
          Disable wrapping code inside markdown code blocks

      --absolute-paths
          Use absolute paths instead of relative paths. Relative paths are the default

      --no-clipboard
          Disable copying to clipboard

  -t, --template <TEMPLATE>
          Optional path to Handlebars template

      --no-spinner
          Whether to render the spinner

      --json
          Whether to print the output as JSON. Defaults to False

      --issue <ISSUE>
          Fetch a specific Github issue for the repository

      --verbose
          Run in verbose mode to investigate glob pattern matching

      --no-warnings
          Ignore all warnings (sensitive files, large token counts, template warnings)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

**Default exclusions.** Some directories (`node_modules`, `.git`, `venv`,
`__pycache__`) are skipped by default. This isn't a flag, it's controlled by the
`[global].ignore` key in the [config file](./config_file.md), read by both the
CLI and the TUI.

**Profiles.** Any of the options above can be saved as a named profile and
replayed with `--profile <name>`. The profile-management flags
(`--write-profile`, `--list-profiles`, `--show-profile`, `--delete-profile`,
`--global`, `--force`) are documented in [Profiles](./profiles.md).
