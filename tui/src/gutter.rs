//! Line-number gutter shared by the TUI panels.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GutterMode {
    /// No gutter is drawn.
    Off,
    /// The absolute line number on every row.
    Absolute,
    /// The distance from the cursor on every row.
    Relative,
    /// The absolute number on the cursor row, relative distances elsewhere.
    Hybrid,
}

impl GutterMode {
    /// Resolves the mode from the `line_numbers` and `relative_line_numbers`
    /// config flags. Both off is [`GutterMode::Off`], both on is the vim-style
    /// [`GutterMode::Hybrid`].
    pub fn from_flags(line_numbers: bool, relative_line_numbers: bool) -> Self {
        match (line_numbers, relative_line_numbers) {
            (false, false) => Self::Off,
            (true, false) => Self::Absolute,
            (false, true) => Self::Relative,
            (true, true) => Self::Hybrid,
        }
    }

    pub fn is_visible(self) -> bool {
        !matches!(self, Self::Off)
    }
}

pub fn number_width(total: usize) -> usize {
    total.max(1).to_string().len()
}

pub fn number(mode: GutterMode, index: usize, cursor: usize) -> Option<usize> {
    match mode {
        GutterMode::Off => None,
        GutterMode::Absolute => Some(index + 1),
        GutterMode::Relative => Some(index.abs_diff(cursor)),
        GutterMode::Hybrid if index == cursor => Some(index + 1),
        GutterMode::Hybrid => Some(index.abs_diff(cursor)),
    }
}

pub fn cell(mode: GutterMode, index: usize, cursor: usize, width: usize) -> String {
    let Some(n) = number(mode, index, cursor) else {
        return String::new();
    };
    if mode == GutterMode::Hybrid && index == cursor {
        format!("{n:<width$} ")
    } else {
        format!("{n:>width$} ")
    }
}
