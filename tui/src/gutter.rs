//! Line-number gutter shared by the TUI panels.

use crate::theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

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

const MIN_GUTTER_DIGITS: usize = 2;
fn gutter_digits(total: usize) -> usize {
    number_width(total).max(MIN_GUTTER_DIGITS)
}

pub fn column_width(total: usize) -> usize {
    gutter_digits(total) + 1
}

/// A synced gutter column, painted beside a list or tree in its own screen
/// column.
pub struct GutterColumn {
    /// The active gutter mode.
    pub mode: GutterMode,
    /// Index of the first visible row (the widget's scroll offset).
    pub offset: usize,
    /// Index of the cursor row, anchoring the relative/hybrid numbers.
    pub cursor: usize,
    /// Total number of rows, used to clamp painting to real content.
    pub total: usize,
}

impl GutterColumn {
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let digits = gutter_digits(self.total);
        let height = area.height as usize;
        let last = (self.offset + height).min(self.total);

        let lines: Vec<Line> = (self.offset..last)
            .map(|index| {
                let style = if index == self.cursor {
                    theme::gutter_current()
                } else {
                    theme::gutter()
                };
                Line::from(Span::styled(
                    cell(self.mode, index, self.cursor, digits),
                    style,
                ))
            })
            .collect();

        frame.render_widget(Paragraph::new(lines), area);
    }
}
