mod cache;
mod cli;
mod config;
mod exec;
mod gui;
mod image;
mod search;
mod state;

use clap::Parser;
use rmenu_plugin::{self_exe, Entry};

static DEFAULT_THEME: &'static str = "style.css";
static DEFAULT_CONFIG: &'static str = "config.yaml";
static XDG_PREFIX: &'static str = "rmenu";
static DEFAULT_CSS_CONTENT: &'static str = include_str!("../public/default.css");

static ENV_BIN: &'static str = "RMENU";
static ENV_ACTIVE_PLUGINS: &'static str = "RMENU_ACTIVE_PLUGINS";

/// Application State for GUI
#[derive(Debug, PartialEq)]
pub struct App {
    css: String,
    name: String,
    theme: String,
    entries: Vec<Entry>,
    config: config::Config,
}

//TODO: how should scripting work?
//  - need a better mechanism for rmenu and another executable to go back and forth
//  - need some way to preserve settings between executions of rmenu
//  - need some way for plugins to customize configuration according to preference

fn main() -> cli::Result<()> {
    // export self to environment for other scripts
    let exe = self_exe();
    std::env::set_var(ENV_BIN, exe);

    // enable log and set default level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // parse cli and retrieve values for app
    let mut cli = cli::Args::parse();
    let mut config = cli.get_config()?;
    let entries = cli.get_entries(&mut config)?;

    // update config based on cli-settings and entries
    config = cli.update_config(config);
    config.use_icons = config.use_icons
        && entries
            .iter()
            .any(|e| e.icon.is_some() || e.icon_alt.is_some());
    config.use_comments = config.use_comments && entries.iter().any(|e| e.comment.is_some());

    // load additional configuration settings from env
    cli.load_env(&mut config)?;

    // configure css theme and css overrides
    let theme = cli.get_theme();
    let css = cli.get_css(&config);

    // set environment variables before running app
    cli.set_env();

    // genrate app context and run gui
    gui::run(App {
        name: "rmenu".to_owned(),
        css,
        theme,
        entries,
        config,
    });

    Ok(())
}
