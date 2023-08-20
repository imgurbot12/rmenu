use std::{fmt::Display, str::FromStr};

use rmenu_plugin::*;

use clap::{Args, Parser, Subcommand};

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

//TODO: add python library to build entries as well

/// Valid Action Modes
#[derive(Debug, Clone)]
enum ActionMode {
    Run,
    Terminal,
    Echo,
}

impl Display for ActionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Run => write!(f, "run"),
            Self::Terminal => write!(f, "terminal"),
            Self::Echo => write!(f, "echo"),
        }
    }
}

impl FromStr for ActionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "run" => Ok(Self::Run),
            "terminal" => Ok(Self::Terminal),
            "echo" => Ok(Self::Echo),
            _ => Err(format!("Invalid Method: {s:?}")),
        }
    }
}

/// Arguents for Action CLI Command
#[derive(Debug, Args)]
struct ActionArgs {
    /// Set Name of Action
    #[arg(short, long, default_value_t=String::from("main"))]
    name: String,
    /// Set Comment of Action
    #[arg(short, long)]
    comment: Option<String>,
    /// Arguments to run As Action Command
    #[clap(required = true, value_delimiter = ' ')]
    args: Vec<String>,
    /// Action Mode
    #[arg(short, long, default_value_t=ActionMode::Run)]
    mode: ActionMode,
}

impl Into<Action> for ActionArgs {
    fn into(self) -> Action {
        let exec = self.args.join(" ");
        Action {
            name: self.name,
            comment: self.comment,
            exec: match self.mode {
                ActionMode::Run => Method::Run(exec),
                ActionMode::Terminal => Method::Terminal(exec),
                ActionMode::Echo => Method::Echo(exec),
            },
        }
    }
}

/// Arguments for Entry CLI Command
#[derive(Debug, Args)]
struct EntryArgs {
    /// Set Name of Entry
    #[arg(short, long, default_value_t=String::from("main"))]
    name: String,
    /// Set Comment of Entry
    #[arg(short, long)]
    comment: Option<String>,
    /// Precomposed Action JSON Objects
    #[arg(short, long, value_parser=parse_action)]
    #[clap(required = true)]
    actions: Vec<Action>,
    /// Icon Image Path
    #[arg(short = 'i', long)]
    icon: Option<String>,
    /// Alternative Image Text/HTML
    #[arg(short = 'I', long)]
    icon_alt: Option<String>,
}

impl Into<Entry> for EntryArgs {
    fn into(self) -> Entry {
        Entry {
            name: self.name,
            comment: self.comment,
            actions: self.actions,
            icon: self.icon,
            icon_alt: self.icon_alt,
        }
    }
}

/// Arguments for Options CLI Command
#[derive(Debug, Args)]
struct OptionArgs {
    /// Override Applicaiton Theme
    #[arg(short = 'C', long)]
    pub css: Option<String>,
    // search settings
    /// Override Default Placeholder
    #[arg(short = 'P', long)]
    pub placeholder: Option<String>,
    /// Override Search Restriction
    #[arg(short = 'r', long)]
    pub search_restrict: Option<String>,
    /// Override Minimum Search Length
    #[arg(short = 'm', long)]
    pub search_min_length: Option<usize>,
    /// Override Maximum Search Length
    #[arg(short = 'M', long)]
    pub search_max_length: Option<usize>,
    // key settings
    /// Override Execution Keybinds
    #[arg(short = 'e', long)]
    pub key_exec: Option<Vec<String>>,
    /// Override Program-Exit Keybinds
    #[arg(short = 'E', long)]
    pub key_exit: Option<Vec<String>>,
    /// Override Move-Next Keybinds
    #[arg(short = 'n', long)]
    pub key_move_next: Option<Vec<String>>,
    /// Override Move-Previous Keybinds
    #[arg(short = 'p', long)]
    pub key_move_prev: Option<Vec<String>>,
    /// Override Open-Menu Keybinds
    #[arg(short = 'o', long)]
    pub key_open_menu: Option<Vec<String>>,
    /// Override Close-Menu Keybinds
    #[arg(short = 'c', long)]
    pub key_close_menu: Option<Vec<String>>,
    /// Override Jump-Next Keybinds
    #[arg(short = 'j', long)]
    pub key_jump_next: Option<Vec<String>>,
    /// Override Jump-Previous Keybinds
    #[arg(short = 'J', long)]
    pub key_jump_prev: Option<Vec<String>>,
    // window settings
    /// Override Window Title
    #[arg(short, long)]
    pub title: Option<String>,
    /// Override Window Deocration Settings
    #[arg(short, long)]
    pub deocorate: Option<bool>,
    /// Override Window Fullscreen Settings
    #[arg(short, long)]
    pub fullscreen: Option<bool>,
    /// Override Window Width
    #[arg(short = 'w', long)]
    pub window_width: Option<f64>,
    /// Override Window Height
    #[arg(short = 'h', long)]
    pub window_height: Option<f64>,
}

impl Into<Options> for OptionArgs {
    fn into(self) -> Options {
        Options {
            css: self.css,
            placeholder: self.placeholder,
            search_restrict: self.search_restrict,
            search_min_length: self.search_min_length,
            search_max_length: self.search_max_length,
            key_exec: self.key_exec,
            key_exit: self.key_exit,
            key_move_next: self.key_move_next,
            key_move_prev: self.key_move_prev,
            key_open_menu: self.key_open_menu,
            key_close_menu: self.key_close_menu,
            key_jump_next: self.key_jump_next,
            key_jump_prev: self.key_jump_prev,
            title: self.title,
            decorate: self.deocorate,
            fullscreen: self.fullscreen,
            window_width: self.window_width,
            window_height: self.window_height,
        }
    }
}

/// Valid CLI Commands and their Arguments
#[derive(Debug, Subcommand)]
enum Command {
    /// Generate Complete RMenu Entry
    Entry(EntryArgs),
    /// Generate RMenu Entry Action Object
    Action(ActionArgs),
    /// Generate RMenu Options Settings
    Options(OptionArgs),
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Generate an Entry/Action Object
    #[clap(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Entry(args) => {
            let entry: Entry = args.into();
            serde_json::to_string(&entry)
        }
        Command::Action(args) => {
            let action: Action = args.into();
            serde_json::to_string(&action)
        }
        Command::Options(args) => {
            let options: Options = args.into();
            serde_json::to_string(&options)
        }
    };
    println!("{}", result.expect("Serialization Failed"));
}
