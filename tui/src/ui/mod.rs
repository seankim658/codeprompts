pub mod draw;
pub mod style;

/// Different UI elements
#[derive(PartialEq)]
pub enum Focus {
    FileTree,
    Options,
    Buttons,
    CommandEditor,
}
