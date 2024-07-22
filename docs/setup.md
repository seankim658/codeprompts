# Setup

This guide will help you install Codeprompts, set it up to run easily, and configure shell completions.

- [Installation](#installation)
- [Running the Tool](#running-the-tool)
- [Setting up Shell Completions](#setting-up-shell-completions)

---

## Installation

Once you download the release binary or compile the release binary from source, you might have to change the file permissions to allow it to be be run as an executable. If needed, can be done with:

```bash
chmod +x <path/to/codeprompts>
```

## Running the Tool

You can run the tool by passing the full path to the executable. However, for convenience, you have several options to run it without specifying the full path each time.

1. Create Alias

Open your shell configuration file (`~/.bashrc`, `~/.zshrc`, `~/.config/fish/config.fish`, etc) and add: 

```bash
alias <alias>='path/to/codeprompts'
```

2. Add to Path

Alternativley, you can add the executable to your system PATH.

```bash
sudo cp path/to/codeprompts /usr/local/bin/
```

After choosing one of the setup options, apply the changes:

```bash
# For bash
source ~/.bashrc

# For zsh
source ~/.zshrc

# For Fish
source ~/.config/fish/config.fish
```

## Setting up Shell Completions

Codeprompts supports shell completions for bash, zsh, fish, and powershell (shell completions will only work if you install the tool to the system path). To set up completions:

1. Generate the completion script for your shell or download from the `completions/` directory.

You can download the appropriate pre-defined completions file or generate it with:

```bash
codeprompts completion <shell>
```

Replace `<shell>` with `bash`, `zsh`, `fish`, or `powershell` as appropriate. For `bash`, `zsh`, and `fish` it is common to store these files in `~/.local/share/<directory>/codeprompts-completions.<shell>`, such as:

```bash
mkdir -p ~/.local/share/codeprompts-completions
mv codeprompts-completions.<shell> ~/.local/share/codeprompts-completions/
```

2. Add the following to your shell configuration file:

For bash (`~/.bashrc`), zsh (`~/.zshrc`), for fish (`~/.config/fish/config.fish`):

```bash
source ~/.local/share/codeprompts-completions/codeprompts-completions.<shell>
```

For powershell (`$PROFILE`):
```ps1
. ~/.local/share/codeprompts-completions/codeprompts-completions.ps1
```

Again, apply the changes: 

```bash
# For bash
source ~/.bashrc

# For zsh
source ~/.zshrc

# For Fish
source ~/.config/fish/config.fish

# For PowerShell, restart your PowerShell session
```
