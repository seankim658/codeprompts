use crate::app::App;
use crate::ui::Focus;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

pub fn handle_events(app: &mut App) -> Result<bool> {
    if let Event::Key(key) = event::read()? {
        match app.focus {
            Focus::FileTree => match key.code {
                // Panel switching
                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.focus = Focus::Options;
                    app.selected_option = Some(0);
                }
                KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.focus = Focus::Buttons;
                    app.selected_button = 0;
                }

                // Navigation
                KeyCode::Char('j') => app.select_next(),
                KeyCode::Char('k') => app.select_previous(),
                KeyCode::Char('h') => app.move_to_parent(),

                // Expand/collapse directory
                KeyCode::Enter => app.toggle_expand(),

                // Include/exclude files
                KeyCode::Char('y') => app.toggle_selected(),
                KeyCode::Char('x') => {
                    if let Some(path) = &app.selected_path {
                        app.toggle_node_by_path(path.clone());
                        app.update_command();
                    }
                }

                KeyCode::Esc => return Ok(false),
                _ => {}
            },

            Focus::Options => match key.code {
                // Panel switching
                KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.focus = Focus::FileTree;
                    app.selected_option = None;
                }
                KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.focus = Focus::Buttons;
                    app.selected_button = 0;
                }

                // Navigation
                KeyCode::Char('j') => {
                    if let Some(idx) = app.selected_option {
                        if idx < app.available_templates.len() + 12 {
                            app.selected_option = Some(idx + 1);
                        }
                    }
                }
                KeyCode::Char('k') => {
                    if let Some(idx) = app.selected_option {
                        if idx > 0 {
                            app.selected_option = Some(idx - 1);
                        }
                    }
                }

                // Toggles option selection
                KeyCode::Enter => app.toggle_current_option(),

                KeyCode::Esc => return Ok(false),
                _ => {}
            },

            Focus::Buttons => match key.code {
                KeyCode::Char('h') => {
                    if app.selected_button > 0 {
                        app.selected_button -= 1;
                    }
                }
                KeyCode::Char('l') => {
                    if app.selected_button > 0 {
                        app.selected_button += 1;
                    }
                }
                KeyCode::Enter => match app.selected_button {
                    0 => app.execute_command()?,
                    1 => app.show_command_editor = true,
                    2 => app.reset(),
                    3 => return Ok(false),
                    _ => {}
                },

                // Panel switching
                KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if app.selected_button <= 1 {
                        app.focus = Focus::FileTree;
                    } else {
                        app.focus = Focus::Options;
                        app.selected_option = Some(0);
                    }
                }

                KeyCode::Esc => return Ok(false),
                _ => {}
            },

            Focus::CommandEditor => match key.code {
                KeyCode::Enter => {
                    app.show_command_editor = false;
                    app.execute_command()?;
                }
                KeyCode::Esc => app.show_command_editor = false,
                _ => {}
            },
        }
    }
    Ok(true)
}
