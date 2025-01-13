use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

pub mod file_tree;
pub mod options_panel;
pub mod templates_panel;

pub trait Panel {
    fn handle_input(&mut self, key: KeyEvent) -> Result<()>;

    fn draw(&mut self, frame: &mut Frame, area: Rect, is_active: bool);

    fn get_command_args(&self) -> Vec<String>;
}
