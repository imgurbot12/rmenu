use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use rmenu_plugin::Entry;

#[derive(Debug, Subcommand)]
pub enum Commands {
    ListEmoji,
    Copy { emoji: String },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

/// Retrieve Avaialble Copy Command from System
fn get_command() -> Result<Command> {
    if let Ok(path) = which::which("wl-copy") {
        Ok(std::process::Command::new(path))
    } else if let Ok(path) = which::which("xsel") {
        let mut cmd = std::process::Command::new(path);
        cmd.arg("-ib");
        Ok(cmd)
    } else if let Ok(path) = which::which("xclip") {
        let mut cmd = std::process::Command::new(path);
        cmd.arg("-selection");
        cmd.arg("c");
        Ok(cmd)
    } else {
        Err(anyhow!("no copy command available!"))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::ListEmoji);
    match command {
        Commands::Copy { emoji } => {
            let mut command = get_command()?;
            let _ = command.arg(emoji).exec();
        }
        Commands::ListEmoji => {
            let exe = std::env::current_exe()
                .expect("failed to discover self")
                .to_str()
                .expect("invalid executable")
                .to_string();
            for emoji in emojis::iter() {
                let action = format!("{exe} copy '{}'", emoji.as_str());
                let entry = Entry::new(emoji.as_str(), &action, Some(emoji.name()));
                let Ok(json) = serde_json::to_string(&entry) else {
                    continue;
                };
                println!("{json}");
            }
        }
    };
    Ok(())
}
