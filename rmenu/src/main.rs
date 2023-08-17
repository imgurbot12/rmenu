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

static CONFIG_DIR: &'static str = "~/.config/rmenu/";
static DEFAULT_CSS: &'static str = "~/.config/rmenu/style.css";
static DEFAULT_CONFIG: &'static str = "~/.config/rmenu/config.yaml";
static DEFAULT_CSS_CONTENT: &'static str = include_str!("../public/default.css");

/// Application State for GUI
#[derive(Debug, PartialEq)]
pub struct App {
    css: String,
    name: String,
    entries: Vec<Entry>,
    config: config::Config,
    theme: String,
}

//TODO: how should scripting work?
//  - need a better mechanism for rmenu and another executable to go back and forth
//  - need some way to preserve settings between executions of rmenu
//  - need some way for plugins to customize configuration according to preference

fn main() -> cli::Result<()> {
    // export self to environment for other scripts
    let exe = self_exe();
    std::env::set_var("RMENU", exe);

    // enable log and set default level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // parse cli and retrieve values for app
    let cli = cli::Args::parse();
    let mut config = cli.get_config()?;
    let css = cli.get_css();
    let theme = cli.get_theme();
    let entries = cli.get_entries(&mut config)?;

    // update config based on entries
    config.use_icons = config.use_icons
        && entries
            .iter()
            .any(|e| e.icon.is_some() || e.icon_alt.is_some());
    config.use_comments = config.use_comments && entries.iter().any(|e| e.comment.is_some());

    // change directory to config folder
    let cfgdir = shellexpand::tilde(CONFIG_DIR).to_string();
    if let Err(err) = std::env::set_current_dir(&cfgdir) {
        log::error!("failed to change directory: {err:?}");
    }

    // genrate app context and run gui
    gui::run(App {
        name: "rmenu".to_owned(),
        css,
        entries,
        config,
        theme,
    });

    Ok(())
}
