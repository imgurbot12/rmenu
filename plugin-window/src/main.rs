use std::fmt::Debug;

use anyhow::Result;
use clap::{Parser, Subcommand};
use rmenu_plugin::Entry;

#[cfg(feature = "sway")]
mod sway;

/// Trait To Implement for Window Focus
pub trait WindowManager: Debug {
    fn focus(&self, id: &str) -> Result<()>;
    fn entries(&self) -> Result<Vec<Entry>>;
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    ListWindow,
    Focus { id: String },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

/// Retrieve WindowManager Implementation
#[allow(unreachable_code)]
fn get_impl() -> impl WindowManager {
    #[cfg(feature = "sway")]
    return sway::SwayManager {};
    // if no features are enabled for some reason?
    panic!("No Implementations Available")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let windows = get_impl();
    let command = cli.command.unwrap_or(Commands::ListWindow);
    match command {
        Commands::Focus { id } => windows.focus(&id)?,
        Commands::ListWindow => {
            for entry in windows.entries()? {
                println!("{}", serde_json::to_string(&entry).unwrap());
            }
        }
    }
    Ok(())
}
