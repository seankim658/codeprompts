# Profiles

Profiles are named bundles of command-line settings. Both project specific and global profiles are supported and both allow you to replay a set of flags by profile name instead of retyping them.

A profile is **sparse**, it stores only the setting you set when saving it, everything else inherits the default value (see [Precedence and Conflicts](#precedence-and-conflicts)).

## Project vs. Global Scope

Profiles live in one of two places:

| Scope       | File                           | Best for                          |
| ----------- | ------------------------------ | --------------------------------- |
| **Project** | `.codeprompt.toml` in the repo | Profiles specific to one codebase |
| **Global**  | `~/codeprompt.toml`            | Profiles you reuse across repos   |

Both files keep profiles under a `[profiles]` table. The schema is identical in both.

Example candidates for **global** profiles are ones not tied to a particular repo:

- A **git-diff** profile pairing `--diff-staged` (or `--diff-unstaged`) with a commit-message template
- A **rust** profiles that excludes `Cargo.lock`

By default every profile command works on the project scope. Add `--global` to any of them to work on `~/.codeprompt.toml` instead.

### How the Project File is Found

For project-scope commands, codeprompt walks up from the target path looking for a `.codeprompt.toml`, stopping at the git root. Your home `~/.codeprompt.toml` is never picked up as a project file. Writing a new project profile requires being inside a git repository.

## Commands

ALl of these accept `--global` to target `~/.codeprompt.toml`.

| Command                   | What it does                                                                                                       |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `--list-profiles`         | List profiles. Without `--global`, shows both scopes under separate headers; with `--global`, only the global ones |
| `--write-profile <name>`  | Save the flags on the command line as a profile                                                                    |
| `--show-profile <name>`   | Print a profile's stored settings and the equivalent command                                                       |
| `--delete-profile <name>` | Remove a profile                                                                                                   |
| `--profile <name>`        | Apply a profile to a dump (see [Using a profile](#using-a-profile))                                                |

`--force` skips the confirmation prompt when `--write-profile` would overwrite an
existing profile or when `--delete-profile` removes one. `--write-profile` can't
be combined with `--profile`.

### Saving a Profile

Pass `--write-profile <name>` together with flags you want to store:

```bash
# A global profile that excludes the lockfile in a Rust repo
codeprompt --write-profile rust --global --exclude "Cargo.lock"

# A global profile pairing the staged diff with a commit-message template
codeprompt --write-profile git-diff --diff-staged -t <path/to/git_commit.hbs>
```

Project profiles are written to the existing `.codeprompt.toml` found while walking up to the git root, or to a new `.codeprompt.toml` at the repo root. Global profiles are written to `~/.codeprompt.toml`.

### Using a Profile

Apply a saved profile to a run with `--profile <name>`:

```bash
codeprompt --profile rust .
```

Without `--global`, codeprompt looks for the profile in the project file first, then falls back to the global file. With `--global`, it looks only in the global file.

## Precedence and Conflicts

When you both apply a profile and pass flags on the command line, they combine
as follows:

- **Boolean toggles** (`--diff-staged`, `--gitignore`, `--no-tokens`, …): the
  command-line flag wins over the profile's value.
- **Value settings** (`--include`, `--exclude`, `--output`, `--template`,
  `--encoding`): setting the same one both in the profile and on the command line
  is treated as a **conflict**, not an override. codeprompt stops and lists the
  clash so you can remove one:

```
conflicting settings between profile 'rust' and command-line flags:
  exclude    profile: ["Cargo.lock"]         flag: ["target"]
Remove the flag or drop it from the profile, then re-run.
```

If only one side sets a value, that side is used.

### Shadowing Between Scopes

A project profile takes precedence over a global profile with the same name. When that happens during `--profile` (or `--show-profile`) without a `--global`, codeprompt prints a warning message:

```
warning: using the project profile 'rust', a global profile with the same name exists.
```

## See also

- [Config File](./config_file.md) — the `~/.codeprompt.toml` schema, including `[profiles]`.
- [Options](./options.md) — the flags you can store in a profile.
- [Git Features](./git_features.md) — the diff flags and templates the git-diff example uses.
