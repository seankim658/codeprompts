# Setup

*Note*: The following usage guide is assuming you are using bash as your shell.

Once you download the release binary or compile the release binary from source, you might have to change the file permissions to allow it to be be run as an executable. If needed, can be done with:

```bash
chmod +x <path/to/codeprompts>
```

The tool can then be run by passing the path to the executable. If you want to run the tool without having to pass the file path everytiem then you have two options: 

## Create Alias

Open your shell configuration file and add: 

```bash
alias <alias>='path/to/codeprompts'
```

```bash
source ~/.bashrc
```

## Add to Path

Alternativley, you can add the executable to your system PATH.

```bash
sudo cp path/to/codeprompts /usr/local/bin/
```

Once you've added the path, save the file and apply the changes, for example: 

```bash
source ~/.bashrc
```
