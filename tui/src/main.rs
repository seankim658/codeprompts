use anyhow::Result;
use codeprompt_tui::prelude::{App, Config};
use std::process::Command;

fn main() -> Result<()> {
    let config = Config::load()?;

    let mut app = App::new(config)?;
    if let Some(cmd) = app.run()? {
        if cmd.starts_with('!') {
            println!("{}", &cmd[1..]);
        } else {
            let args = app.construct_command_args();
            println!("{:?}", args);
            if let Some((program, args)) = args.split_first() {
                Command::new(program).args(args).spawn()?.wait()?;
            }
        }
    }

    Ok(())
}
