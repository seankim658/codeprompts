//! Input handling for the global TUI application. If necessary, passes on the key handling to the
//! responsible panel.

use crate::input::InputState;
use crate::prelude::{ActivePanel, App, Button, Panel};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub const MOVE_LEFT: char = 'h';
pub const MOVE_RIGHT: char = 'l';
pub const MOVE_DOWN: char = 'j';
pub const MOVE_UP: char = 'k';
const EXIT: char = 'q';
const RUN_KEY: char = 'r';
const PRINT_KEY: char = 'p';
const RESET_KEY: char = 't';
const HELP_KEY: char = '?';
const OPEN_SEARCH_KEY: char = '/';

/// Entry point for handling generic application keyboard input events.
pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.search.is_active() {
        return app.handle_search_key(key);
    }

    // A numeric prefix builds the pending count (e.g. `12j`).
    if let Some(digit) = count_digit(&app.input, key) {
        app.input.push_digit(digit);
        return Ok(());
    }

    // A lone `g` waits for a second to complete `gg`.
    if key.code == KeyCode::Char('g') && key.modifiers == KeyModifiers::NONE {
        if app.input.register_g() {
            if let Some(panel) = active_panel_mut(app) {
                panel.jump_to_top();
            }
            app.input.reset();
        }
        return Ok(());
    }

    // Every remaining key ends the sequence; resolve the count before clearing.
    let count = app.input.take_count();
    app.input.reset();

    match (key.code, key.modifiers) {
        // Count-aware navigation within the active panel
        (KeyCode::Char(MOVE_DOWN), KeyModifiers::NONE) => move_active_down(app, count),
        (KeyCode::Char(MOVE_UP), KeyModifiers::NONE) => move_active_up(app, count),
        (KeyCode::Char('G'), _) => jump_active_bottom(app),

        // Global key handlers
        (KeyCode::Char(EXIT), KeyModifiers::NONE) => app.should_exit = true,
        (KeyCode::Char(RUN_KEY), KeyModifiers::NONE) => run_button_action(app, Button::Run)?,
        (KeyCode::Char(PRINT_KEY), KeyModifiers::NONE) => run_button_action(app, Button::Print)?,
        (KeyCode::Char(RESET_KEY), KeyModifiers::NONE) => run_button_action(app, Button::Reset)?,
        (KeyCode::Char(HELP_KEY), KeyModifiers::NONE) => app.toggle_help(),
        (KeyCode::Char(OPEN_SEARCH_KEY), KeyModifiers::NONE) => app.open_search(),

        // Panel navigation
        (KeyCode::Char(MOVE_LEFT), KeyModifiers::CONTROL) => navigate_left(app),
        (KeyCode::Char(MOVE_RIGHT), KeyModifiers::CONTROL) => navigate_right(app),
        (KeyCode::Char(MOVE_DOWN), KeyModifiers::CONTROL) => navigate_down(app),
        (KeyCode::Char(MOVE_UP), KeyModifiers::CONTROL) => navigate_up(app),

        // Fallback to delegate to active panel handling
        _ => handle_panel_input(app, key)?,
    }

    Ok(())
}

/// Interprets `key` as a count digit, if it is one. A leading `0` is not the
/// start of a count.
fn count_digit(input: &InputState, key: KeyEvent) -> Option<u32> {
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    let KeyCode::Char(c) = key.code else {
        return None;
    };
    let digit = c.to_digit(10)?;
    if digit == 0 && input.pending_count().is_none() {
        return None;
    }
    Some(digit)
}

/// The active panel as a trait object, or `None` when the button row is focused
/// (it has no row selection).
fn active_panel_mut(app: &mut App) -> Option<&mut dyn Panel> {
    match app.active_panel {
        ActivePanel::FileTree => Some(&mut app.file_tree),
        ActivePanel::Options => Some(&mut app.options),
        ActivePanel::Templates => Some(&mut app.templates),
        ActivePanel::Buttons => None,
    }
}

fn move_active_down(app: &mut App, count: usize) {
    if let Some(panel) = active_panel_mut(app) {
        panel.move_down(count);
    }
}

fn move_active_up(app: &mut App, count: usize) {
    if let Some(panel) = active_panel_mut(app) {
        panel.move_up(count);
    }
}

fn jump_active_bottom(app: &mut App) {
    if let Some(panel) = active_panel_mut(app) {
        panel.jump_to_bottom();
    }
}

/// Handle input depending on currently active panel
fn handle_panel_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.active_panel {
        ActivePanel::FileTree => app.file_tree.handle_input(key)?,
        ActivePanel::Options => app.options.handle_input(key)?,
        ActivePanel::Templates => app.templates.handle_input(key)?,
        ActivePanel::Buttons => handle_button_input(app, key)?,
    }
    Ok(())
}

/// Hanndle input when button row is focused
fn handle_button_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Enter => {
            let button = app.focused_button;
            run_button_action(app, button)?;
        }
        KeyCode::Char(MOVE_LEFT) => {
            focus_prev_button(app);
        }
        KeyCode::Char(MOVE_RIGHT) => {
            focus_next_button(app);
        }
        _ => {}
    }
    Ok(())
}

/// Panel Navigation

fn navigate_left(app: &mut App) {
    app.active_panel = match app.active_panel {
        ActivePanel::Options | ActivePanel::Templates => ActivePanel::FileTree,
        _ => app.active_panel,
    };
}

fn navigate_right(app: &mut App) {
    // TODO : probably shouldn't hardcode a right move to always go to the options panel, maybe
    // depending on vertical screen position it will go to the templates panel?
    app.active_panel = match app.active_panel {
        ActivePanel::FileTree => ActivePanel::Options,
        _ => app.active_panel,
    };
}

fn navigate_up(app: &mut App) {
    // TODO : same considerations as in navigate_right
    app.active_panel = match app.active_panel {
        ActivePanel::Templates => ActivePanel::Options,
        ActivePanel::Buttons => ActivePanel::Templates,
        _ => app.active_panel,
    };
}

fn navigate_down(app: &mut App) {
    app.active_panel = match app.active_panel {
        ActivePanel::Options => ActivePanel::Templates,
        ActivePanel::FileTree | ActivePanel::Templates => ActivePanel::Buttons,
        _ => app.active_panel,
    }
}

/// Button Interaction

fn focus_next_button(app: &mut App) {
    app.focused_button = app.focused_button.next();
}

fn focus_prev_button(app: &mut App) {
    app.focused_button = app.focused_button.prev();
}

pub fn run_button_action(app: &mut App, button: Button) -> Result<()> {
    match button {
        Button::Run => {
            app.should_exit = true;
            let cmd = app.construct_command();
            app.set_command(cmd);
        }
        Button::Print => {
            app.should_exit = true;
            let cmd = app.construct_command();
            app.set_command(format!("!{}", cmd));
        }
        Button::Reset => {
            app.reset()?;
        }
        Button::Exit => {
            app.should_exit = true;
        }
    }
    Ok(())
}
