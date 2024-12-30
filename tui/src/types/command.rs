use anyhow::Result;
use std::process::Command as ProcessCommand;

/// Wrapper around `std::process::Command` for executing CLI commands
pub struct Command {
    inner: ProcessCommand,
}

impl Command {
    pub fn new(program: &str) -> Self {
        Self {
            inner: ProcessCommand::new(program),
        }
    }

    /// Add arguments to the command
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// Spawn command as a child process
    pub fn spawn(&mut self) -> Result<std::process::Child> {
        Ok(self.inner.spawn()?)
    }

    /// Wait for command to complete
    #[allow(dead_code)]
    pub fn wait(&mut self) -> Result<std::process::ExitStatus> {
        let mut child = self.spawn()?;
        Ok(child.wait()?)
    }
}
