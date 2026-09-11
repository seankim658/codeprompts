//! A single-line, Vim-style text editor for the fuzzy-finder query bar.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Insert,
    Normal,
}

/// What a key did to the editor, so the embedder knows how to react.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOutcome {
    /// The buffer text changed, so anything derived from it should
    /// be recomputed.
    TextChanged,
    /// The key was handled but the text did not change (a motion
    /// or mode change). Nothing derived needs to be recomputed.
    Consumed,
    /// The editor did not claim this key.
    Ignored,
}

// A key sequence awaiting its next key.
#[derive(Debug, Clone, Copy)]
enum Pending {
    None,
    /// A find-char motion (f/t/F/T)
    Find(FindKind),
    /// An operator awaiting a motion (d/c)
    Operator(Op),
    /// An operator find motion (e.g. df) awaiting its target character
    OperatorFind(Op, FindKind),
    /// AN operator text object (e.g. di) awaiting the object type (w)
    OperatorObject(Op, Scope),
}

/// An operator that acts on a range of text.
#[derive(Debug, Clone, Copy)]
enum Op {
    /// d
    Delete,
    /// c
    Change,
}

/// The scope of a text object.
#[derive(Debug, Clone, Copy)]
enum Scope {
    /// i
    Inner,
    /// a
    A,
}

/// Which find-char motion is pending.
#[derive(Debug, Clone, Copy)]
enum FindKind {
    /// `f`: forward, land on target
    ForwardTo,
    /// `t`: forward, land just before the target
    ForwardTill,
    /// `F`: backward, land on target
    BackwardTo,
    /// `T`: backward, land just before the target
    BackwardTill,
}

/// Vim's character classes for word motions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Whitespace,
    Word,
    Punct,
}

/// Classifies a character for word motions.
fn char_class(c: char) -> CharClass {
    if c.is_whitespace() {
        CharClass::Whitespace
    } else if c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else {
        CharClass::Punct
    }
}

/// A single line of text edited with a subset of Vim motions and commands.
pub struct PromptEditor {
    /// The buffer, one entry per character.
    buffer: Vec<char>,
    /// Cursor position as a character index.
    ///
    /// In insert mode it ranges over `0..=len`.
    /// In normal mdoe it is clamped to `0..=len-1`.
    cursor: usize,
    /// The current editing mode.
    mode: Mode,
    /// A key sequence.
    pending: Pending,
}

impl Default for PromptEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptEditor {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
            mode: Mode::Insert,
            pending: Pending::None,
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.mode = Mode::Insert;
        self.pending = Pending::None;
    }

    pub fn text(&self) -> String {
        self.buffer.iter().collect()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Handles one key event, dispatching by the current mode.
    ///
    /// ### Arguments
    ///
    /// - `key`: The key event to interpret.
    ///
    /// ### Returns
    ///
    /// - `KeyOutcome`: Whether the text changed, the key
    ///   was handled, or the key was left for the embedder
    ///   to interpret.
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyOutcome {
        match self.pending {
            Pending::Find(kind) => return self.complete_find(kind, key),
            Pending::Operator(op) => return self.complete_operator(op, key),
            Pending::OperatorFind(op, kind) => return self.complete_operator_find(op, kind, key),
            Pending::OperatorObject(op, scope) => {
                return self.complete_operator_object(op, scope, key)
            }
            Pending::None => {}
        }
        match self.mode {
            Mode::Insert => self.handle_insert_key(key),
            Mode::Normal => self.handle_normal_key(key),
        }
    }

    /// Handles a key while in insert mode.
    fn handle_insert_key(&mut self, key: KeyEvent) -> KeyOutcome {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::ALT)
                {
                    return KeyOutcome::Consumed;
                }
                self.insert_char(c);
                KeyOutcome::TextChanged
            }
            KeyCode::Backspace => self.delete_before_cursor(),
            KeyCode::Left => {
                self.move_left();
                KeyOutcome::Consumed
            }
            KeyCode::Right => {
                self.move_right_insert();
                KeyOutcome::Consumed
            }
            KeyCode::Esc => {
                self.enter_normal_mode();
                KeyOutcome::Consumed
            }
            _ => KeyOutcome::Consumed,
        }
    }

    /// Handles a key while in normal mode.
    fn handle_normal_key(&mut self, key: KeyEvent) -> KeyOutcome {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            || key.modifiers.contains(KeyModifiers::ALT)
        {
            return KeyOutcome::Ignored;
        }

        match key.code {
            // Motions
            KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
                self.move_left();
                KeyOutcome::Consumed
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.move_right_normal();
                KeyOutcome::Consumed
            }
            KeyCode::Char('0') => {
                self.cursor = 0;
                KeyOutcome::Consumed
            }
            KeyCode::Char('$') => {
                self.cursor = self.last_normal_index();
                KeyOutcome::Consumed
            }
            KeyCode::Char('^') => {
                self.cursor = self.first_non_blank();
                KeyOutcome::Consumed
            }
            KeyCode::Char('w') => {
                self.cursor = self.next_word_start();
                KeyOutcome::Consumed
            }
            KeyCode::Char('b') => {
                self.cursor = self.prev_word_start();
                KeyOutcome::Consumed
            }
            KeyCode::Char('e') => {
                self.cursor = self.word_end();
                KeyOutcome::Consumed
            }
            KeyCode::Char('f') => {
                self.pending = Pending::Find(FindKind::ForwardTo);
                KeyOutcome::Consumed
            }
            KeyCode::Char('t') => {
                self.pending = Pending::Find(FindKind::ForwardTill);
                KeyOutcome::Consumed
            }
            KeyCode::Char('F') => {
                self.pending = Pending::Find(FindKind::BackwardTo);
                KeyOutcome::Consumed
            }
            KeyCode::Char('T') => {
                self.pending = Pending::Find(FindKind::BackwardTill);
                KeyOutcome::Consumed
            }

            // Enter insert mode
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                KeyOutcome::Consumed
            }
            KeyCode::Char('a') => {
                self.move_right_insert();
                self.mode = Mode::Insert;
                KeyOutcome::Consumed
            }
            KeyCode::Char('I') | KeyCode::Char('O') => {
                self.cursor = self.first_non_blank();
                self.mode = Mode::Insert;
                KeyOutcome::Consumed
            }
            KeyCode::Char('A') | KeyCode::Char('o') => {
                self.cursor = self.len();
                self.mode = Mode::Insert;
                KeyOutcome::Consumed
            }

            // Simple edit
            KeyCode::Char('x') => self.delete_under_cursor(),

            // Operators
            KeyCode::Char('d') => {
                self.pending = Pending::Operator(Op::Delete);
                KeyOutcome::Consumed
            }
            KeyCode::Char('c') => {
                self.pending = Pending::Operator(Op::Change);
                KeyOutcome::Consumed
            }
            KeyCode::Char('D') => self.delete_to_line_end(Op::Delete),
            KeyCode::Char('C') => self.delete_to_line_end(Op::Change),
            KeyCode::Char('s') => self.substitute_char(),
            KeyCode::Char('S') => self.substitute_line(),

            // Not an editor command
            _ => KeyOutcome::Ignored,
        }
    }

    // --- Buffer Helpers ---

    /// The number of characters in the buffer
    fn len(&self) -> usize {
        self.buffer.len()
    }

    // The largest valid cursor index in normal mode (0 when empty)
    fn last_normal_index(&self) -> usize {
        self.buffer.len().saturating_sub(1)
    }

    /// The index of the first non-whitespace character
    fn first_non_blank(&self) -> usize {
        self.buffer
            .iter()
            .position(|c| !c.is_whitespace())
            .unwrap_or_else(|| self.last_normal_index())
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Moves the cursor one character to the right, allowing the
    /// past-the-end position used by insert mode.
    fn move_right_insert(&mut self) {
        self.cursor = (self.cursor + 1).min(self.len());
    }

    /// Moves the cursor one character right, clamping on len() - 1
    /// for normal mode.
    fn move_right_normal(&mut self) {
        self.cursor = (self.cursor + 1).min(self.last_normal_index());
    }

    /// Inserts a character at the cursor and advances past it.
    fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Deletes the character before the cursor (backspace).
    fn delete_before_cursor(&mut self) -> KeyOutcome {
        if self.cursor == 0 {
            return KeyOutcome::Consumed;
        }
        self.cursor -= 1;
        self.buffer.remove(self.cursor);
        KeyOutcome::TextChanged
    }

    /// Deletes the character under the cursor (x), keeping the cursor
    /// on the character index.
    fn delete_under_cursor(&mut self) -> KeyOutcome {
        if self.buffer.is_empty() {
            return KeyOutcome::Consumed;
        }
        self.buffer.remove(self.cursor);
        self.cursor = self.cursor.min(self.last_normal_index());
        KeyOutcome::TextChanged
    }

    fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
        self.cursor = self.cursor.saturating_sub(1);
        self.cursor = self.cursor.min(self.last_normal_index());
    }

    // --- Word Motions ---

    /// The index of the start of the next word (w).
    fn next_word_start(&self) -> usize {
        let len = self.len();
        if len == 0 {
            return 0;
        }
        let mut i = self.cursor;
        let start_class = char_class(self.buffer[i]);
        if start_class != CharClass::Whitespace {
            while i < len && char_class(self.buffer[i]) == start_class {
                i += 1;
            }
        }
        while i < len && char_class(self.buffer[i]) == CharClass::Whitespace {
            i += 1;
        }
        i.min(self.last_normal_index())
    }

    /// The index of the start of the previous word (b).
    fn prev_word_start(&self) -> usize {
        if self.cursor == 0 {
            return 0;
        }
        let mut i = self.cursor - 1;
        while i > 0 && char_class(self.buffer[i]) == CharClass::Whitespace {
            i -= 1;
        }
        if char_class(self.buffer[i]) == CharClass::Whitespace {
            return 0;
        }
        let class = char_class(self.buffer[i]);
        while i > 0 && char_class(self.buffer[i - 1]) == class {
            i -= 1;
        }
        i
    }

    /// The index of the end of the current or next word (e).
    fn word_end(&self) -> usize {
        let last = self.last_normal_index();
        if self.len() == 0 || self.cursor >= last {
            return last;
        }
        let mut i = self.cursor + 1;
        while i < last && char_class(self.buffer[i]) == CharClass::Whitespace {
            i += 1;
        }
        let class = char_class(self.buffer[i]);
        while i < last && char_class(self.buffer[i + 1]) == class {
            i += 1;
        }
        i
    }

    // --- Find Motions ---

    // Completes a pending find motion.
    fn complete_find(&mut self, kind: FindKind, key: KeyEvent) -> KeyOutcome {
        self.pending = Pending::None;
        let target = match key.code {
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                c
            }
            _ => return KeyOutcome::Consumed,
        };
        if let Some(dest) = self.find_target(kind, target) {
            self.cursor = dest;
        }
        KeyOutcome::Consumed
    }

    /// Computes the destination for a find motion, or `None` when the target
    /// character is not present in the search direction.
    fn find_target(&self, kind: FindKind, target: char) -> Option<usize> {
        match kind {
            FindKind::ForwardTo => {
                ((self.cursor + 1)..self.len()).find(|&i| self.buffer[i] == target)
            }
            FindKind::ForwardTill => ((self.cursor + 1)..self.len())
                .find(|&i| self.buffer[i] == target)
                .map(|i| i - 1),
            FindKind::BackwardTo => (0..self.cursor).rev().find(|&i| self.buffer[i] == target),
            FindKind::BackwardTill => (0..self.cursor)
                .rev()
                .find(|&i| self.buffer[i] == target)
                .map(|i| i + 1),
        }
    }

    // --- Operators ---

    /// Resolves a key following a `d` or `c` operator into an edit.
    fn complete_operator(&mut self, op: Op, key: KeyEvent) -> KeyOutcome {
        self.pending = Pending::None;
        if key.modifiers.contains(KeyModifiers::CONTROL)
            || key.modifiers.contains(KeyModifiers::ALT)
        {
            return KeyOutcome::Consumed;
        }
        let KeyCode::Char(c) = key.code else {
            return KeyOutcome::Consumed;
        };
        match c {
            'd' if matches!(op, Op::Delete) => self.apply_operator(op, 0, self.len()),
            'c' if matches!(op, Op::Change) => self.apply_operator(op, 0, self.len()),
            // Find-char motions need a further key
            'f' => {
                self.pending = Pending::OperatorFind(op, FindKind::ForwardTo);
                KeyOutcome::Consumed
            }
            't' => {
                self.pending = Pending::OperatorFind(op, FindKind::ForwardTill);
                KeyOutcome::Consumed
            }
            'F' => {
                self.pending = Pending::OperatorFind(op, FindKind::BackwardTo);
                KeyOutcome::Consumed
            }
            'T' => {
                self.pending = Pending::OperatorFind(op, FindKind::BackwardTill);
                KeyOutcome::Consumed
            }
            'i' => {
                self.pending = Pending::OperatorObject(op, Scope::Inner);
                KeyOutcome::Consumed
            }
            'a' => {
                self.pending = Pending::OperatorObject(op, Scope::A);
                KeyOutcome::Consumed
            }
            _ => match self.motion_range(op, c) {
                Some((start, end)) => self.apply_operator(op, start, end),
                None => KeyOutcome::Consumed,
            },
        }
    }

    /// Resolves the target character following an operator find motion.
    fn complete_operator_find(&mut self, op: Op, kind: FindKind, key: KeyEvent) -> KeyOutcome {
        self.pending = Pending::None;
        let target = match key.code {
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                c
            }
            _ => return KeyOutcome::Consumed,
        };
        match self.operator_find_range(kind, target) {
            Some((start, end)) => self.apply_operator(op, start, end),
            None => KeyOutcome::Consumed,
        }
    }

    /// The range an operator find motion coveres.
    fn operator_find_range(&self, kind: FindKind, target: char) -> Option<(usize, usize)> {
        match kind {
            FindKind::ForwardTo => {
                let found = self.find_target(FindKind::ForwardTo, target)?;
                Some((self.cursor, found + 1))
            }
            FindKind::ForwardTill => {
                let stop = self.find_target(FindKind::ForwardTill, target)?;
                Some((self.cursor, stop + 1))
            }
            FindKind::BackwardTo => {
                let found = self.find_target(FindKind::BackwardTo, target)?;
                Some((found, self.cursor))
            }
            FindKind::BackwardTill => {
                let stop = self.find_target(FindKind::BackwardTill, target)?;
                Some((stop, self.cursor))
            }
        }
    }

    /// Computes the half-open `[start, end)` range an operator should act on for
    /// a cimple motion, or `None` when the motion produces an empty range.
    fn motion_range(&self, op: Op, motion: char) -> Option<(usize, usize)> {
        let cursor = self.cursor;
        let len = self.len();
        let (start, end) = match motion {
            'h' => (cursor.saturating_sub(1), cursor),
            'l' => (cursor, (cursor + 1).min(len)),
            '0' => (0, cursor),
            '^' => {
                let target = self.first_non_blank_index();
                (target.min(cursor), target.max(cursor))
            }
            '$' => (cursor, len),
            'e' => (cursor, self.word_end() + 1),
            'b' => (self.prev_word_start(), cursor),
            'w' => {
                let on_word =
                    cursor < len && char_class(self.buffer[cursor]) != CharClass::Whitespace;
                if matches!(op, Op::Change) && on_word {
                    // Vim special case: cw acts like ce.
                    (cursor, self.word_end() + 1)
                } else {
                    (cursor, self.next_word_target())
                }
            }
            _ => return None,
        };
        if start >= end {
            None
        } else {
            Some((start, end))
        }
    }

    /// Deletes the half-open range `[start, end)` and repositions the cursor.
    fn apply_operator(&mut self, op: Op, start: usize, end: usize) -> KeyOutcome {
        let start = start.min(self.len());
        let end = end.min(self.len());
        if start >= end {
            return KeyOutcome::Consumed;
        }
        self.buffer.drain(start..end);
        self.cursor = start;
        match op {
            Op::Delete => {
                self.cursor = self.cursor.min(self.last_normal_index());
                KeyOutcome::TextChanged
            }
            Op::Change => {
                self.mode = Mode::Insert;
                KeyOutcome::TextChanged
            }
        }
    }

    /// `D` / `C`: delete (or change) from the cursor to the end of the line.
    fn delete_to_line_end(&mut self, op: Op) -> KeyOutcome {
        if self.cursor >= self.len() {
            if matches!(op, Op::Change) {
                self.mode = Mode::Insert;
            }
            return KeyOutcome::Consumed;
        }
        self.apply_operator(op, self.cursor, self.len())
    }

    /// `s`: delete the character under the cursor and enter insert mode.
    fn substitute_char(&mut self) -> KeyOutcome {
        if self.buffer.is_empty() {
            self.mode = Mode::Insert;
            return KeyOutcome::Consumed;
        }
        self.apply_operator(Op::Change, self.cursor, (self.cursor + 1).min(self.len()))
    }

    /// `S`: clear the whole line and enter insert mode.
    fn substitute_line(&mut self) -> KeyOutcome {
        if self.buffer.is_empty() {
            self.mode = Mode::Insert;
            return KeyOutcome::Consumed;
        }
        self.apply_operator(Op::Change, 0, self.len())
    }

    /// The index of the start of the next word, or the line length when the
    /// cursor is on the last word.
    fn next_word_target(&self) -> usize {
        let len = self.len();
        let mut i = self.cursor;
        if i >= len {
            return len;
        }
        let start_class = char_class(self.buffer[i]);
        if start_class != CharClass::Whitespace {
            while i < len && char_class(self.buffer[i]) == start_class {
                i += 1;
            }
        }
        while i < len && char_class(self.buffer[i]) == CharClass::Whitespace {
            i += 1;
        }
        i
    }

    /// The index of the first non-whitespace character, or 0 when the line is
    /// empty or blank.
    fn first_non_blank_index(&self) -> usize {
        self.buffer
            .iter()
            .position(|c| !c.is_whitespace())
            .unwrap_or(0)
    }

    // --- Text Objects (iw / aw) ---

    /// Resolvse the object type following `di`/`ci`/`da`/`ca` into an edit.
    fn complete_operator_object(&mut self, op: Op, scope: Scope, key: KeyEvent) -> KeyOutcome {
        self.pending = Pending::None;
        let KeyCode::Char('w') = key.code else {
            return KeyOutcome::Consumed;
        };
        let range = match scope {
            Scope::Inner => self.inner_word_range(),
            Scope::A => self.a_word_range(),
        };
        match range {
            Some((start, end)) => self.apply_operator(op, start, end),
            None => KeyOutcome::Consumed,
        }
    }

    /// The `[start, end)` range of the inner word object (`iw`) under the cursor.
    fn inner_word_range(&self) -> Option<(usize, usize)> {
        let len = self.len();
        if len == 0 {
            return None;
        }
        let cursor = self.cursor.min(len - 1);
        let class = char_class(self.buffer[cursor]);

        let mut start = cursor;
        while start > 0 && char_class(self.buffer[start - 1]) == class {
            start -= 1;
        }
        let mut end = cursor + 1;
        while end < len && char_class(self.buffer[end]) == class {
            end += 1;
        }
        Some((start, end))
    }

    /// The `[start, end)` range of the "a word" object (`aw`) under the cursor.
    fn a_word_range(&self) -> Option<(usize, usize)> {
        let len = self.len();
        let (start, end) = self.inner_word_range()?;
        let inner_is_whitespace = char_class(self.buffer[start]) == CharClass::Whitespace;

        if inner_is_whitespace {
            let mut new_end = end;
            if new_end < len {
                let next_class = char_class(self.buffer[new_end]);
                while new_end < len && char_class(self.buffer[new_end]) == next_class {
                    new_end += 1;
                }
            }
            return Some((start, new_end));
        }

        // Include trailing whitespace when present.
        let mut new_end = end;
        while new_end < len && char_class(self.buffer[new_end]) == CharClass::Whitespace {
            new_end += 1;
        }
        if new_end > end {
            return Some((start, new_end));
        }

        // Otherwise include leading whitespace.
        let mut new_start = start;
        while new_start > 0 && char_class(self.buffer[new_start - 1]) == CharClass::Whitespace {
            new_start -= 1;
        }
        Some((new_start, end))
    }
}
