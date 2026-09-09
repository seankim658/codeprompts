use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

/// Accent color for the focused panel border, cursor bar, and active title.
pub const ACCENT: Color = Color::Cyan;
/// Foreground painted on top of the accent (selection bar, focused button).
/// Black stays legible against the bright accent on any terminal.
pub const ACCENT_TEXT: Color = Color::Black;
/// Color for an entry the user has marked to include.
pub const INCLUDED: Color = Color::Green;
/// Color for an entry the user has marked to exclude.
pub const EXCLUDED: Color = Color::Red;
/// Border and text color for panels that do not hold focus.
pub const INACTIVE: Color = Color::DarkGray;
/// Secondary text: hints, the line-number gutter, status-bar details.
pub const MUTED: Color = Color::Gray;

/// Border glyphs used by every panel. Rounded corners read as more modern than
/// the default square border.
pub const BORDER_TYPE: BorderType = BorderType::Rounded;

/// Border style for a panel, brightened and bolded on focus.
pub fn border(is_active: bool) -> Style {
    if is_active {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(INACTIVE)
    }
}

/// Title style for a panel.
pub fn title(is_active: bool) -> Style {
    if is_active {
        Style::default()
            .fg(ACCENT_TEXT)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(INACTIVE)
    }
}

/// Cursor row-style.
pub fn selection(is_active: bool) -> Style {
    if is_active {
        Style::default()
            .fg(ACCENT_TEXT)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(INACTIVE)
    }
}

/// Style for a button in the bottom row.
pub fn button(is_focused: bool) -> Style {
    if is_focused {
        Style::default()
            .fg(ACCENT_TEXT)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED)
    }
}

/// Dim style for secondary text.
pub fn muted() -> Style {
    Style::default().fg(MUTED)
}
