
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'codeprompt' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'codeprompt'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'codeprompt' {
            [CompletionResult]::new('--include', 'include', [CompletionResultType]::ParameterName, 'Glob patterns to include')
            [CompletionResult]::new('--exclude', 'exclude', [CompletionResultType]::ParameterName, 'Glob patterns to exclude')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Tokenizer to use for token count')
            [CompletionResult]::new('--encoding', 'encoding', [CompletionResultType]::ParameterName, 'Tokenizer to use for token count')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Redirect output to file')
            [CompletionResult]::new('--output', 'output', [CompletionResultType]::ParameterName, 'Redirect output to file')
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Optional path to Handlebars template')
            [CompletionResult]::new('--template', 'template', [CompletionResultType]::ParameterName, 'Optional path to Handlebars template')
            [CompletionResult]::new('--issue', 'issue', [CompletionResultType]::ParameterName, 'Fetch a specific Github issue for the repository')
            [CompletionResult]::new('--include-priority', 'include-priority', [CompletionResultType]::ParameterName, 'Pattern priority in case of conflict (True to prioritize Include pattern, False to prioritize exclude pattern). Defaults to True')
            [CompletionResult]::new('--exclude-from-tree', 'exclude-from-tree', [CompletionResultType]::ParameterName, 'Whether to exclude files/folders from the source tree based on exclude patterns. Defaults to False')
            [CompletionResult]::new('--gitignore', 'gitignore', [CompletionResultType]::ParameterName, 'Whether to respect the .gitignore file. Defaults to True')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Whether to capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`. Defaults to False')
            [CompletionResult]::new('--diff-staged', 'diff-staged', [CompletionResultType]::ParameterName, 'Whether to capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`. Defaults to False')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Whether to capture the git diff for unstaged changes only (equivalent to running `git diff`). Defaults to False')
            [CompletionResult]::new('--diff-unstaged', 'diff-unstaged', [CompletionResultType]::ParameterName, 'Whether to capture the git diff for unstaged changes only (equivalent to running `git diff`). Defaults to False')
            [CompletionResult]::new('--tokens', 'tokens', [CompletionResultType]::ParameterName, 'Display approximate token count of the genrated prompt. Defaults to True')
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'Toggle line numbers to source code. Defaults to True')
            [CompletionResult]::new('--line-numbers', 'line-numbers', [CompletionResultType]::ParameterName, 'Toggle line numbers to source code. Defaults to True')
            [CompletionResult]::new('--no-codeblock', 'no-codeblock', [CompletionResultType]::ParameterName, 'Disable wrapping code inside markdown code blocks. Defaults to False')
            [CompletionResult]::new('--relative-paths', 'relative-paths', [CompletionResultType]::ParameterName, 'Use relative paths instead of absolute paths, including parent directory. Defaults to True')
            [CompletionResult]::new('--no-clipboard', 'no-clipboard', [CompletionResultType]::ParameterName, 'Disable copying to clipboard. Defaults to False')
            [CompletionResult]::new('--spinner', 'spinner', [CompletionResultType]::ParameterName, 'Whether to render the spinner (incurs some overhead but is nice to look at). Defaults to True')
            [CompletionResult]::new('--json', 'json', [CompletionResultType]::ParameterName, 'Whether to print the output as JSON. Defaults to False')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Run in verbose mode to investigate glob pattern matching. Defaults to False')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('completion', 'completion', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'codeprompt;completion' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'codeprompt;help' {
            [CompletionResult]::new('completion', 'completion', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'codeprompt;help;completion' {
            break
        }
        'codeprompt;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
