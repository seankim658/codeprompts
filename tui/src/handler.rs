//! Input handling for the global TUI application. If necessary, passes on the key handling to the
//! responsible panel.

use crate::prelude::{ActivePanel, App, Panel};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub const MOVE_LEFT: char = 'h';
pub const MOVE_RIGHT: char = 'l';
pub const MOVE_DOWN: char = 'j';
pub const MOVE_UP: char = 'k';
const EXIT: char = 'q';

/// Entry point point for handling generic application keyboard input events
pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match (key.code, key.modifiers) {
        // Global key handlers
        (KeyCode::Char(EXIT), KeyModifiers::NONE) => {
            app.should_exit = true;
        }

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
            handle_button_action(app)?;
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
    app.focused_button = (app.focused_button + 1) % 4;
}

fn focus_prev_button(app: &mut App) {
    app.focused_button = (app.focused_button + 3) % 4;
}

pub fn handle_button_action(app: &mut App) -> Result<()> {
    match app.focused_button {
        0 => {
            // Run
            app.should_exit = true;
            let cmd = app.construct_command();
            app.set_command(cmd);
        }
        1 => {
            // Submit
            app.should_exit = true;
            let cmd = app.construct_command();
            app.set_command(format!("!{}", cmd));
        }
        2 => {
            // Reset
            app.reset()?;
        }
        3 => {
            // Exit
            app.should_exit = true;
        }
        _ => {}
    }
    Ok(())
}
