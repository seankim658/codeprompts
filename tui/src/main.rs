use anyhow::Result;
use codeprompt_tui::prelude::{App, Config};
use std::process::Command;

fn main() -> Result<()> {
    let config = Config::load()?;

    let mut app = App::new(config)?;
    if let Some(cmd) = app.run()? {
        if cmd.starts_with('!') {
            let cmd = &cmd[1..];
            Command::new("bash")
                .arg("-c")
                .arg(format!("echo '{}' > /dev/tty; stty sane", cmd))
                .spawn()?
                .wait()?;
        } else {
            let parts: Vec<_> = cmd.split_whitespace().collect();
            Command::new(parts[0]).args(&parts[1..]).spawn()?.wait()?;
        }
    }

    Ok(())
}
