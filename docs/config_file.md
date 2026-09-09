# Config File

Both `codeprompt` and the TUI read an optional config file at `~/.codeprompt.toml`.
If it's missing, both fall back to their built-in defaults.

The file has two top-level sections:

- `[tui]`: read only by the TUI. Sets the command it builds, where it looks for
  templates, and the starting state of every option toggle.
- `[global]`: read by **both** the TUI and the CLI it runs. This is how the two
  stay in agreement about what to skip, with no arguments passed between them.

A ready-to-edit sample lives at
[`codeprompt.example.toml`](../codeprompt.example.toml) in the repo root — copy it
to `~/.codeprompt.toml` and adjust. Every value is optional.

## `[tui]`

| Key | Default | Meaning |
| --- | --- | --- |
| `command` | `"codeprompt"` | The command the TUI builds and runs or submits. |
| `template_dir` | *(unset)* | Directory scanned for `.hbs` templates; a leading `~` is expanded. Unset disables the templates panel. |

### `[tui.defaults]`

The starting state of each toggle in the Options panel. These mirror the CLI
flags in [Options](./options.md); see that guide for what each one does.

| Key | Default |
| --- | --- |
| `exclude_priority` | `false` |
| `exclude_from_tree` | `false` |
| `literal_brackets` | `false` |
| `diff_staged` | `false` |
| `diff_unstaged` | `false` |
| `gitignore` | `true` |
| `no_tokens` | `false` |
| `no_line_numbers` | `false` |
| `no_codeblock` | `false` |
| `absolute_paths` | `false` |
| `no_clipboard` | `false` |
| `no_spinner` | `false` |

## `[global]`

### `ignore`

Directory names to skip while walking, matched on the final path component
(plain names, not globs). Because both binaries read this, what you see in the
TUI tree is what the CLI will dump.

The semantics are **replace, not merge**:

- omit the key → the built-in defaults are used: `node_modules`, `.git`, `venv`, `__pycache__`
- set it → your list fully replaces the defaults
- set it to `[]` → nothing is ignored

```toml
[global]
ignore = ["node_modules", ".git", "target"]
```

This is a pure replace with no special cases. If you drop `.git` from the list,
the walk descends into `.git`; binary objects are filtered out automatically, but
plain-text git files (like `HEAD` and `config`) would then be included. Keep
`.git` in the list unless you have a reason not to.

## Upgrading from the old format

Earlier versions used a flat, TUI-only file with `command`, `template_dir`, and a
`[defaults]` table at the top level. Those now live under `[tui]` and
`[tui.defaults]`. A file in the old shape still loads without error, but its TUI
settings are silently ignored and you'll get defaults — move them under `[tui]`.
