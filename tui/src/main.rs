mod app;
mod event;
mod types;
mod ui;

use anyhow::Result;
use app::App;
// use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::handler::handle_events;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io;
use ui::draw::draw;

/// Sets up the terminal for TUI rendering.
///
/// This function:
/// 1. Enables raw mode (terminal receives input without waiting for <CR>)
/// 2. Creates a new terminal output handle
/// 3. Enters the alternate screen (saves current terminal context)
/// 4. Enables mouse capture for mouse event handling
///
/// Returns a configured Terminal instance ready for TUI rendering
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

/// Restores the terminal to its original state.
///
/// This function:
/// 1. Disables raw mode
/// 2. Leaves the alternate screen (restores original terminal content)
/// 3. Disables mouse capture
/// 4. Shows the cursor again
fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Main application loop.
fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;

    // Get current directory as default path
    let path = std::env::current_dir()?;
    let mut app = App::new(path)?;

    // Main app loop
    loop {
        // Draw UI
        terminal.draw(|f| draw(f, &app))?;

        // Hanle input events
        if !handle_events(&mut app)? {
            break;
        }
    }

    // Cleanup and restore terminal
    restore_terminal(terminal)?;

    Ok(())
}

fn main() -> Result<()> {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
    }
    Ok(())
}
