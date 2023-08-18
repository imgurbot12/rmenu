use rmenu_plugin::*;

use clap::{Parser, Subcommand};

/// Parse Action from JSON
fn parse_action(action: &str) -> Result<Action, serde_json::Error> {
    serde_json::from_str(action)
}

//TODO: add options struct object that allows for further
// dynamic customization of the cli settings.
// last instance overwrites previous entries
// properties do not override cli-specified values
//
// priority order:
//   1. cli flags
//   2. plugin/source latest merged options
//   3. configuration settings

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate Complete RMenu Entry
    Entry {
        /// Precomposed Action JSON Objects
        #[arg(short, long, value_parser=parse_action)]
        #[clap(required = true)]
        actions: Vec<Action>,
        /// Icon Image Path
        #[arg(short, long)]
        icon: Option<String>,
        /// Alternative Image Text/HTML
        #[arg(short = 'o', long)]
        icon_alt: Option<String>,
    },
    /// Generate RMenu Entry Action Object
    Action {
        /// Arguments to run As Action Command
        #[clap(required = true, value_delimiter = ' ')]
        args: Vec<String>,
        /// Run in New Terminal Session if Active
        #[arg(short, long)]
        terminal: bool,
    },
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Set Name of Entry/Action
    #[arg(short, long, default_value_t=String::from("main"))]
    name: String,
    /// Set Comment of Entry/Action
    #[arg(short, long)]
    comment: Option<String>,
    /// Generate an Entry/Action Object
    #[clap(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Entry {
            actions,
            icon,
            icon_alt,
        } => serde_json::to_string(&Entry {
            name: cli.name,
            comment: cli.comment,
            actions,
            icon,
            icon_alt,
        }),
        Command::Action { args, terminal } => serde_json::to_string(&Action {
            name: cli.name,
            exec: Method::new(args.join(" "), terminal),
            comment: cli.comment,
        }),
    };
    println!("{}", result.expect("Serialization Failed"));
}
