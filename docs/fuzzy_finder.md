# Fuzzy Finder

The TUI includes a telescope-style fuzzy finder for quickly locating files and directories anywhere
in the project tree and marking them for include/exclude without scrolling the tree by hand.

Press `/` from any panel to open it. It opens as a full-screen modal in **insert mode** with the query
line focused. Type to filter, results are fuzzy-ranked best-first and update as you type. Marks you make
apply to the source tree panel immediately, so the command preview updates live while the finder stays
open.

## Modes

The query line is a small Vim-style editor with insert and normal modes. Type in insert mode to filter.
Switch from insert to normal mode with `Esc` (or your configured [`escape_sequence`](./config_file.md),
e.g. `jj`); from normal mode, `Esc` or `q` closes the finder.

You do not have to leave insert mode to mark results: `Enter` cycles the selected result's mark
(none -> include -> exclude -> none) in **both** modes. Setting a mark directly with `+` / `-` is
normal-mode only.

## Finder Keys

| Key               | Mode   | Action                                               |
| ----------------- | ------ | ---------------------------------------------------- |
| `/`               | —      | Open the finder (from any panel)                     |
| `Enter`           | both   | Cycle the selected mark (none -> include -> exclude) |
| `Tab`             | both   | Reveal the selection in the tree and close           |
| `Ctrl-n` / `Down` | both   | Move selection down                                  |
| `Ctrl-p` / `Up`   | both   | Move selection up                                    |
| `j` / `k`         | normal | Move selection down / up                             |
| `+`               | normal | Mark the selected result **include**                 |
| `-`               | normal | Mark the selected result **exclude**                 |
| `q` / `Esc`       | normal | Close the finder                                     |
| `Ctrl-c`          | both   | Close the finder                                     |

## Query-line Vim Commands

The query line supports a subset of Vim motions. Because it is a single line, line-wise motions 
(`gg`, `G`) don't apply, and in the finder `j` / `k` in normal mode move the result selection rather 
than the cursor.

### Motions (normal mode)

| Key           | Moves the cursor to              |
| ------------- | -------------------------------- |
| `h` / `Left`  | one character left               |
| `l` / `Right` | one character right              |
| `0`           | the first column                 |
| `^`           | the first non-blank character    |
| `$`           | the end of the line              |
| `w`           | the start of the next word       |
| `b`           | the start of the previous word   |
| `e`           | the end of the word              |
| `f{char}`     | the next `{char}`                |
| `t{char}`     | just before the next `{char}`    |
| `F{char}`     | the previous `{char}`            |
| `T{char}`     | just after the previous `{char}` |

### Entering insert mode

| Key      | Action                                             |
| -------- | -------------------------------------------------- |
| `i`      | insert before the cursor                           |
| `a`      | append after the cursor                            |
| `I`, `O` | insert at the first non-blank character            |
| `A`, `o` | append at the end of the line                      |
| `s`      | delete the character under the cursor, then insert |
| `S`      | clear the whole line, then insert                  |

### Editing (normal mode)

| Key           | Action                                                      |
| ------------- | ----------------------------------------------------------- |
| `x`           | delete the character under the cursor                       |
| `D`           | delete from the cursor to the end of the line               |
| `C`           | change from the cursor to the end of the line               |
| `dd`          | delete the whole line                                       |
| `cc`          | change the whole line                                       |
| `d{motion}`   | delete over any motion above (e.g. `dw`, `db`, `d$`, `dfx`) |
| `c{motion}`   | change over any motion above (`cw` acts like `ce`)          |
| `diw` / `daw` | delete the inner word / a word (with surrounding space)     |
| `ciw` / `caw` | change the inner word / a word                              |
