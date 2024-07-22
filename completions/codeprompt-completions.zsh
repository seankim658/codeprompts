#compdef codeprompt

autoload -U is-at-least

_codeprompt() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'--include=[Glob patterns to include]:INCLUDE: ' \
'--exclude=[Glob patterns to exclude]:EXCLUDE: ' \
'-c+[Tokenizer to use for token count]:ENCODING: ' \
'--encoding=[Tokenizer to use for token count]:ENCODING: ' \
'-o+[Redirect output to file]:OUTPUT: ' \
'--output=[Redirect output to file]:OUTPUT: ' \
'-t+[Optional path to Handlebars template]:TEMPLATE:_files' \
'--template=[Optional path to Handlebars template]:TEMPLATE:_files' \
'--issue=[Fetch a specific Github issue for the repository]:ISSUE: ' \
'--include-priority[Pattern priority in case of conflict (True to prioritize Include pattern, False to prioritize exclude pattern). Defaults to True]' \
'--exclude-from-tree[Whether to exclude files/folders from the source tree based on exclude patterns. Defaults to False]' \
'--gitignore[Whether to respect the .gitignore file. Defaults to True]' \
'-d[Whether to capture the git diff for staged changes only (equivalent to running \`git diff --cached\` or \`git diff --staged\`. Defaults to False]' \
'--diff-staged[Whether to capture the git diff for staged changes only (equivalent to running \`git diff --cached\` or \`git diff --staged\`. Defaults to False]' \
'-u[Whether to capture the git diff for unstaged changes only (equivalent to running \`git diff\`). Defaults to False]' \
'--diff-unstaged[Whether to capture the git diff for unstaged changes only (equivalent to running \`git diff\`). Defaults to False]' \
'--tokens[Display approximate token count of the genrated prompt. Defaults to True]' \
'-l[Toggle line numbers to source code. Defaults to True]' \
'--line-numbers[Toggle line numbers to source code. Defaults to True]' \
'--no-codeblock[Disable wrapping code inside markdown code blocks. Defaults to False]' \
'--relative-paths[Use relative paths instead of absolute paths, including parent directory. Defaults to True]' \
'--no-clipboard[Disable copying to clipboard. Defaults to False]' \
'--spinner[Whether to render the spinner (incurs some overhead but is nice to look at). Defaults to True]' \
'--json[Whether to print the output as JSON. Defaults to False]' \
'--verbose[Run in verbose mode to investigate glob pattern matching. Defaults to False]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
'::path -- Path to project directory:_files' \
":: :_codeprompt_commands" \
"*::: :->codeprompt" \
&& ret=0
    case $state in
    (codeprompt)
        words=($line[2] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:codeprompt-command-$line[2]:"
        case $line[2] in
            (completion)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':shell:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_codeprompt__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:codeprompt-help-command-$line[1]:"
        case $line[1] in
            (completion)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_codeprompt_commands] )) ||
_codeprompt_commands() {
    local commands; commands=(
'completion:Generate shell completion scripts.' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'codeprompt commands' commands "$@"
}
(( $+functions[_codeprompt__completion_commands] )) ||
_codeprompt__completion_commands() {
    local commands; commands=()
    _describe -t commands 'codeprompt completion commands' commands "$@"
}
(( $+functions[_codeprompt__help_commands] )) ||
_codeprompt__help_commands() {
    local commands; commands=(
'completion:Generate shell completion scripts.' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'codeprompt help commands' commands "$@"
}
(( $+functions[_codeprompt__help__completion_commands] )) ||
_codeprompt__help__completion_commands() {
    local commands; commands=()
    _describe -t commands 'codeprompt help completion commands' commands "$@"
}
(( $+functions[_codeprompt__help__help_commands] )) ||
_codeprompt__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'codeprompt help help commands' commands "$@"
}

if [ "$funcstack[1]" = "_codeprompt" ]; then
    _codeprompt "$@"
else
    compdef _codeprompt codeprompt
fi
