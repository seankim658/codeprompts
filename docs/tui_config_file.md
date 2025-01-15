# TUI Config File

The TUI will look for an optional configuration file at `~/.codeprompt.toml`. If no configuration file is found, the TUI will fall back on the default options specified in the [CLI options](/docs/options.md). Here, you can override the default values, specify where the TUI should look for templates, and update the command to run.  


```toml
command = "codeprompt"
# Replace this with your own value
template_dir = "path/to/your/templates/directory/"

[defaults]
# Give precedence to exclude patterns over include patterns when they conflict
exclude_priority = false
# Exclude files/folders from the source tree based on exclude patterns
exclude_from_tree = false
diff_staged = false
diff_unstaged = false
# Whether to respect the .gitignore file
gitignore = true
no_tokens = false
no_line_numbers = false
no_codeblock = false
relative_paths = false
no_clipboard = false
no_spinner = false
```
