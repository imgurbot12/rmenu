mod action;

use std::fmt::Display;

use clap::{Parser, Subcommand};
use rmenu_plugin::{self_exe, Method};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Subcommand)]
pub enum Command {
    ListActions { no_confirm: bool },
    Shutdown,
    Reboot,
    Suspend,
    Logout,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::ListActions { .. } => write!(f, "list-actions"),
            Command::Shutdown => write!(f, "shutdown"),
            Command::Reboot => write!(f, "reboot"),
            Command::Suspend => write!(f, "suspend"),
            Command::Logout => write!(f, "logout"),
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
}

fn main() {
    let cli = Cli::parse();
    let exe = self_exe();

    let actions = action::list_actions();
    let command = cli
        .command
        .unwrap_or(Command::ListActions { no_confirm: false });
    match command {
        Command::ListActions { no_confirm } => {
            for (command, mut entry) in actions {
                if !no_confirm {
                    let exec = format!("{exe} {command}");
                    entry.actions[0].exec = Method::Run(exec);
                }
                println!("{}", serde_json::to_string(&entry).unwrap());
            }
        }
        command => {
            action::confirm(command, &actions);
        }
    }
}
