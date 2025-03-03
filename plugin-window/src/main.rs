use std::fmt::Debug;

use anyhow::Result;
use clap::{Parser, Subcommand};
use rmenu_plugin::Entry;

#[cfg(feature = "sway")]
mod sway;

#[cfg(feature = "hyprland")]
mod hyprland;

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
fn get_impl() -> Box<dyn WindowManager> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    if cfg!(feature = "sway") && desktop == "sway" {
        return Box::new(sway::SwayManager {});
    } else if cfg!(feature = "hyprland") && desktop == "Hyprland" {
        return Box::new(hyprland::HyprlandManager {});
    }
    panic!("Desktop environment {desktop:?} not supported.");
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
