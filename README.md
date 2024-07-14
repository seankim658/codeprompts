# Code Prompts

Command line tool for creating LLM prompts from your code using [Handlebars](https://handlebarsjs.com/) templates.

This was a project to brush up on Rust and is based on [code2prompt](https://github.com/mufeedvh/code2prompt) with some additional functionality that I found useful.

- [Why Code Prompts?](#why-code-prompts)
- [Installation](#installation)
  - [Release Binary](#release-binary)
  - [Building From Source](#building-from-source)
- [Usage](#usage)
  - [Arguments](#arguments)
- [Templates](#templates)

---

## Why Code Prompts?

Manually copy and pasting code and code snippets to LLMs has several issues:

1. **You lose formatting and structure**. For a language like Python that uses whitespace, the formatting and structure of the code is important for the LLM to accurately and comprehensively understand the code for your request.
2. **You lose context**. In order for the LLM to provide the best responses, it requires the context from all of your code. This includes your code snippet's dependencies, other user defined modules/objects, and other scope variables and concepts.
3. **You lose prompt standardization**: If you are using LLMs to perform repeated tasks like documenting code, you have to make sure that your documentation styling guidelines are consistent across every request.

To get the most out of LLMs, prompting has to be clear, comprehensive, and consistent. How code prompts fixes these issues:

1. Code prompts will ensure the inherent structure of your code is retained to limit any possible misinterpretation or inferrence required from the LLM.
2. The complete context required for understanding the code is enclosed in a standardized format. This includes the project directory tree and file pathing.
3. By using Handlebar templates, the prompts will be comprehensive and consistent. If your preferred style guide and/or coding practices are updated in the future, there is no need to remember to update all future manually typed prompts. Instead, just update the target Handlebars template once.

## Installation

### Release Binary

To download a release binary, go to the [releases](https://github.com/seankim658/codeprompts/releases) and download the binary for your OS.

### Building From Source

To build from source you will need to have [git](https://git-scm.com/downloads), [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html), and Cargo (will be installed with Rust) installed.

First clone the repository:

```bash
git clone git@github.com:seankim658/codeprompts.git
```

And then compile a release binary:

```bash
cd codeprompts/
cargo build --release
```

## Usage

### Arguments

The code prompts command line tool has the following arguments:

```txt
Usage: codeprompt [OPTIONS] <PATH>

Arguments:
  <PATH>
          Path to project directory

Options:
      --include <INCLUDE>
          Glob patterns to include

      --exclude <EXCLUDE>
          Glob patterns to exclude

      --include-priority
          Pattern priority in case of conflict (True to prioritize Include pattern, False to prioritize exclude pattern). Defaults to True

      --exclude-from-tree
          Whether to exclude files/folders from the source tree based on exclude patterns. Defaults to False

      --gitignore
          Whether to respect the .gitignore file. Defaults to True

  -d, --diff-staged
          Whether to capture the git diff for staged changes only (equivalaent to running `git diff --cached` or `git diff --staged`. Defaults to False

  -u, --diff-unstaged
          Whether to capture the git diff for unstaged changes only (equivalaent to running `git diff`). Defaults to False

      --tokens
          Display approximate token count of the genrated prompt. Defaults to True

  -c, --encoding <ENCODING>
          Tokenizer to use for token count.

          Right now cl100k is the only supported tokenizer.

          [default: cl100k]

  -o, --output <OUTPUT>
          Redirect output to file

  -l, --line-numbers
          Toggle line numbers to source code. Defaults to True

      --no-codeblock
          Disable wrapping code inside markdown code blocks. Defaults to False

      --relative-paths
          Use relative paths instead of absolute paths, including parent directory. Defaults to True

      --no-clipboard
          Disable copying to clipboard. Defaults to False

  -t, --template <TEMPLATE>
          Optional path to Handlebars template

      --spinner
          Whether to render the spinner (incurs some overhead but is nice to look at). Defaults to True

      --json
          Whether to print the output as JSON. Defaults to False

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Templates

The templates use a simple templating language called [Handlebars](https://handlebarsjs.com/guide/). For this project, the Handlebars templates are used to generate markdown. 

The pre-defined templates can be downloaded from the project [releases](https://github.com/seankim658/codeprompts/releases). Download the `templates.zip`.

Currently, the included pre-defined templates are:

| Template Name                                                              | Description                                                                                                                                                                |
| -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`default_template.hbs`](./src/templates/default_template.hbs)             | This is a simple default template that will structure your project path, source tree, and code blocks.                                                                     |
| [`documentation_template.hbs`](./src/templates/documentation_template.hbs) | The documentation template creates a prompt for documenting code. The documentation guidelines are consistent with the HIVE lab guidelines and documentation requirements. |
