#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use anyhow::Result;
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::ListEmoji);
    match command {
        Commands::Copy { emoji } => {
            rmenu_plugin::extra::clipboard::copy_and_exit(&emoji).map_err(anyhow::Error::msg)?;
        }
        Commands::ListEmoji => {
            let exe = rmenu_plugin::extra::get_exe();
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
