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

You can run the tool by passing the full path to the executable. However, for convenience, you have a couple options to run it without specifying the full path each time:

1. Create an Alias

Open your shell configuration file (`~/.bashrc`, `~/.zshrc`, `~/.config/fish/config.fish`, etc) and add: 

```bash
alias <alias>='path/to/codeprompts'
```

2. Add to System PATH

Alternativley, you can add the executable to your system PATH.

```bash
sudo cp path/to/codeprompts /usr/local/bin/
```

After choosing one of the setup options, apply the changes by reloading your shell configuration:

```bash
# For bash
source ~/.bashrc

# For zsh
source ~/.zshrc

# For Fish
source ~/.config/fish/config.fish
```

## Setting up Shell Completions

Codeprompts supports shell completions for Bash, Zsh, Fish, and Powershell.

1. Generate the completion script for your shell or download from the `completions/` directory.

You can download the appropriate pre-defined completions file or generate it with:

```bash
codeprompts completion <shell> > codeprompts-completion.<shell>
```

Replace `<shell>` with `bash`, `zsh`, `fish`, or `powershell` as appropriate. 

You can place completion scripts where you prefer, but some standard locations and setup procedures for each shell are: 

For bash: 

```bash
mkdir -p ~/.local/share/bash-completion/completions
mv codeprompts-completion.bash ~/.local/share/bash-completion/completions
```

If you add it to a custom location, you'll have to source that location in your `.bashrc`.

For Zsh:
```bash
mkdir -p ~/.zsh/completion
mv codeprompts-completion.zsh ~/.zsh/completion/_codeprompts
echo 'fpath=(~/.zsh/completion $fpath)' >> ~/.zshrc
echo 'autoload -U compinit; compinit' >> ~/.zshrc
```

For Fish:
```fish
mkdir -p ~/.config/fish/completions
mv codeprompts-completion.fish ~/.config/fish/completions/codeprompts.fish
```

If you add it to a custom location, you'll have to source that location in your `fish.config`.

For PowerShell:
```ps1
mkdir -p $HOME\Documents\PowerShell\Completions
mv codeprompts-completion.ps1 $HOME\Documents\PowerShell\Completions\codeprompts.ps1
Add-Content $PROFILE '. $HOME\Documents\PowerShell\Completions\codeprompts.ps1'
```

After adding installing the completions, you'll have to re-source your config files.
