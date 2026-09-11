use crate::input::InputState;
use crate::panels::Panel;
use crate::prelude::{handle_input, ui, Config, FileTree, OptionsPanel, TemplatesPanel};
use crate::search::{FileSearch, MarkKind, SearchAction};
use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{event, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;

/// The panels in the TUI
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ActivePanel {
    FileTree,
    Options,
    Templates,
    Buttons,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    Run,
    Print,
    Reset,
    Exit,
}

impl Button {
    pub const ALL: [Button; 4] = [Button::Run, Button::Print, Button::Reset, Button::Exit];

    pub fn label(self) -> &'static str {
        match self {
            Button::Run => "Run",
            Button::Print => "Print",
            Button::Reset => "Reset",
            Button::Exit => "Exit",
        }
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&button| button == self)
            .expect("every Button variant is listed in Button::ALL")
    }

    pub fn next(self) -> Button {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Button {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// Main application state
pub struct App {
    /// Terminaal instance
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// The current active panel
    pub active_panel: ActivePanel,
    /// File tree state
    pub file_tree: FileTree,
    /// Options panel state
    pub options: OptionsPanel,
    /// Templates panel state
    pub templates: TemplatesPanel,
    /// Whether the app should exit
    pub should_exit: bool,
    /// The constructed command if running or submitting
    command: Option<String>,
    /// Application configuration
    config: Config,
    /// Currently focused button
    pub focused_button: Button,
    /// Whether to show the help popup
    show_help: bool,
    /// In-progress vim-style key input
    pub input: InputState,
    /// Fuzzy file finder modal
    pub search: FileSearch,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config) -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            active_panel: ActivePanel::FileTree,
            file_tree: FileTree::new(&config)?,
            options: OptionsPanel::new(&config),
            templates: TemplatesPanel::new(&config)?,
            should_exit: false,
            command: None,
            config,
            focused_button: Button::Run,
            show_help: false,
            input: InputState::default(),
            search: FileSearch::new(),
        })
    }

    /// Start the application event loop
    pub fn run(&mut self) -> Result<Option<String>> {
        // Setup terminal
        enable_raw_mode()?;
        execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )?;

        let result = self.run_app();

        // Cleanup terminal
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        )?;
        disable_raw_mode()?;

        result
    }

    pub fn construct_command_args(&self) -> Vec<String> {
        let mut args = vec![self.config.tui.command.clone(), ".".to_owned()];
        args.extend(self.file_tree.get_command_args());
        args.extend(self.options.get_command_args());
        args.extend(self.templates.get_command_args());
        args
    }

    pub fn construct_command(&self) -> String {
        let args = self.construct_command_args();
        args.iter()
            .enumerate()
            .map(|(i, arg)| {
                if i > 0 && (args[i - 1] == "--include" || args[i - 1] == "--exclude") {
                    format!("\"{}\"", arg)
                } else {
                    arg.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Inner run loop, separated from public `run` to ensure cleanup runs if there is a runtime
    /// error
    fn run_app(&mut self) -> Result<Option<String>> {
        // Main application loop
        while !self.should_exit {
            self.file_tree.set_gitignore(self.options.gitignore());
            self.file_tree
                .set_exclude_priority(self.options.exclude_priority());
            let command = self.construct_command();
            let gutter = self.config.tui.gutter_mode();

            self.terminal.draw(|frame| {
                ui::draw(
                    frame,
                    &command,
                    &mut self.options,
                    &mut self.file_tree,
                    &mut self.templates,
                    self.active_panel,
                    self.focused_button,
                    self.config.config_status,
                    self.show_help,
                    gutter,
                    &self.search,
                );
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    handle_input(self, key)?;
                }
            }
        }

        Ok(self.command.take())
    }

    /// Set the command to return when exiting
    pub fn set_command(&mut self, cmd: String) {
        self.command = Some(cmd);
    }

    /// Reset panel states
    pub fn reset(&mut self) -> Result<()> {
        self.file_tree = FileTree::new(&self.config)?;
        self.options = OptionsPanel::new(&self.config);
        self.templates = TemplatesPanel::new(&self.config)?;
        Ok(())
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn open_search(&mut self) {
        self.show_help = false;
        let candidates = self.file_tree.candidate_paths();
        self.search.open(candidates);
    }

    pub fn handle_search_key(&mut self, key: event::KeyEvent) -> Result<()> {
        if let Some(action) = self.search.handle_key(key) {
            self.apply_search_action(action);
        }
        Ok(())
    }

    fn apply_search_action(&mut self, action: SearchAction) {
        match action {
            SearchAction::Mark { path, kind } => match kind {
                MarkKind::Include => self.file_tree.toggle_include_path(&path),
                MarkKind::Exclude => self.file_tree.toggle_exclude_path(&path),
                MarkKind::Cycle => self.file_tree.cycle_status_path(&path),
            },
            SearchAction::Reveal(_path) => self.search.close(),
            SearchAction::Close => self.search.close(),
        }
    }
}
