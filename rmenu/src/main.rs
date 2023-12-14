mod cache;
mod cli;
mod config;
mod exec;
mod gui;
mod search;

use clap::Parser;
use config::{CacheSetting, PluginConfig};
use rmenu_plugin::{self_exe, Entry};

static CONFIG_DIR: &'static str = "~/.config/rmenu/";
static DEFAULT_CSS: &'static str = "~/.config/rmenu/style.css";
static DEFAULT_CONFIG: &'static str = "~/.config/rmenu/config.yaml";

/// Application State for GUI
#[derive(Debug, PartialEq)]
pub struct AppData {
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
    std::env::set_var("RMENU", exe);

    // enable log and set default level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // parse cli and retrieve values for app
    let mut cli = cli::Args::parse();
    // let mut config = cli.get_config()?;
    let mut config = crate::config::Config::default();
    config.plugins.insert(
        "run".to_owned(),
        PluginConfig {
            exec: vec!["/home/andrew/.config/rmenu/rmenu-run".to_owned()],
            cache: CacheSetting::OnLogin,
            placeholder: None,
            options: None,
        },
    );
    let entries = cli.get_entries(&mut config)?;
    let css = cli.get_css(&config);
    let theme = cli.get_theme();

    // update config based on cli-settings and entries
    config = cli.update_config(config);
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
    gui::run(AppData {
        name: "rmenu".to_owned(),
        css,
        theme,
        entries,
        config,
    });

    Ok(())
}
