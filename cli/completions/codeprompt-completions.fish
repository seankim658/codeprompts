# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_codeprompt_global_optspecs
	string join \n include= exclude= exclude-priority exclude-from-tree gitignore d/diff-staged u/diff-unstaged no-tokens c/encoding= o/output= l/no-line-numbers no-codeblock relative-paths no-clipboard t/template= no-spinner json issue= verbose h/help V/version
end

function __fish_codeprompt_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_codeprompt_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_codeprompt_using_subcommand
	set -l cmd (__fish_codeprompt_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c codeprompt -n "__fish_codeprompt_needs_command" -l include -d 'Glob patterns to include' -r
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l exclude -d 'Glob patterns to exclude' -r
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s c -l encoding -d 'Tokenizer to use for token count' -r
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s o -l output -d 'Redirect output to file' -r
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s t -l template -d 'Optional path to Handlebars template' -r -F
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l issue -d 'Fetch a specific Github issue for the repository' -r
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l exclude-priority -d 'Change pattern priority in case of conflict to prioritize the exclusion pattern'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l exclude-from-tree -d 'Eclude files/folders from the source tree based on exclude patterns'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l gitignore -d 'Don\'t respect .gitignore file'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s d -l diff-staged -d 'Capture the git diff for staged changes only (equivalent to running `git diff --cached` or `git diff --staged`'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s u -l diff-unstaged -d 'Capture the git diff for unstaged changes only (equivalent to running `git diff`)'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l no-tokens -d 'Don\'t display approximate token count of the genrated prompt'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s l -l no-line-numbers -d 'Turn off line numbers in source code blocks'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l no-codeblock -d 'Disable wrapping code inside markdown code blocks'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l relative-paths -d 'Use relative paths instead of absolute paths, including parent directory'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l no-clipboard -d 'Disable copying to clipboard'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l no-spinner -d 'Whether to render the spinner'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l json -d 'Whether to print the output as JSON. Defaults to False'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -l verbose -d 'Run in verbose mode to investigate glob pattern matching'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -s V -l version -d 'Print version'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -a "completion" -d 'Generate shell completion scripts.'
complete -c codeprompt -n "__fish_codeprompt_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c codeprompt -n "__fish_codeprompt_using_subcommand completion" -s h -l help -d 'Print help'
complete -c codeprompt -n "__fish_codeprompt_using_subcommand help; and not __fish_seen_subcommand_from completion help" -f -a "completion" -d 'Generate shell completion scripts.'
complete -c codeprompt -n "__fish_codeprompt_using_subcommand help; and not __fish_seen_subcommand_from completion help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
