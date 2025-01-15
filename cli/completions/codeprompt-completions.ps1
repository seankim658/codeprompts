
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
            [CompletionResult]::new('--exclude-priority', 'exclude-priority', [CompletionResultType]::ParameterName, 'Change pattern priority in case of conflict to prioritize the exclusion pattern')
            [CompletionResult]::new('--exclude-from-tree', 'exclude-from-tree', [CompletionResultType]::ParameterName, 'Eclude files/folders from the source tree based on exclude patterns')
            [CompletionResult]::new('--gitignore', 'gitignore', [CompletionResultType]::ParameterName, 'Don''t respect .gitignore file')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`')
            [CompletionResult]::new('--diff-staged', 'diff-staged', [CompletionResultType]::ParameterName, 'Capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Capture the git diff for unstaged changes only (equivalent to running `git diff`)')
            [CompletionResult]::new('--diff-unstaged', 'diff-unstaged', [CompletionResultType]::ParameterName, 'Capture the git diff for unstaged changes only (equivalent to running `git diff`)')
            [CompletionResult]::new('--no-tokens', 'no-tokens', [CompletionResultType]::ParameterName, 'Don''t display approximate token count of the genrated prompt')
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'Turn off line numbers in source code blocks')
            [CompletionResult]::new('--no-line-numbers', 'no-line-numbers', [CompletionResultType]::ParameterName, 'Turn off line numbers in source code blocks')
            [CompletionResult]::new('--no-codeblock', 'no-codeblock', [CompletionResultType]::ParameterName, 'Disable wrapping code inside markdown code blocks')
            [CompletionResult]::new('--relative-paths', 'relative-paths', [CompletionResultType]::ParameterName, 'Use relative paths instead of absolute paths, including parent directory')
            [CompletionResult]::new('--no-clipboard', 'no-clipboard', [CompletionResultType]::ParameterName, 'Disable copying to clipboard')
            [CompletionResult]::new('--no-spinner', 'no-spinner', [CompletionResultType]::ParameterName, 'Whether to render the spinner')
            [CompletionResult]::new('--json', 'json', [CompletionResultType]::ParameterName, 'Whether to print the output as JSON. Defaults to False')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Run in verbose mode to investigate glob pattern matching')
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
