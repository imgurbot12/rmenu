#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod cache;
mod cli;
mod config;
mod exec;
mod gui;
mod search;
mod server;

use clap::Parser;
use server::ServerBuilder;

static DEFAULT_THEME: &'static str = "style.css";
static DEFAULT_CONFIG: &'static str = "config.yaml";
static XDG_PREFIX: &'static str = "rmenu";

static ENV_BIN: &'static str = "RMENU";
static ENV_ACTIVE_PLUGINS: &'static str = "RMENU_ACTIVE_PLUGINS";

//TODO: remove min-length from search options in rmenu-lib

fn main() -> server::Result<()> {
    env_logger::init();

    // export self to environment for other scripts
    let exe = rmenu_plugin::self_exe();
    std::env::set_var(ENV_BIN, exe);

    // parse cli and retrieve values for app
    let mut cli = cli::Args::parse();
    let mut config = cli.get_config()?;

    // spawn plugin server
    let mut builder = ServerBuilder::default();
    if let Some(input) = cli.input.as_ref() {
        builder = builder.add_input(cli.format, input, &mut config)?;
    }
    if cli.input.is_none() && cli.run.is_empty() {
        builder = builder.add_input(cli.format, "-", &mut config)?;
    }
    builder = builder.add_plugins(cli.run.clone(), &mut config)?;
    let server = builder.build(cli.show.clone())?;

    // update config based on cli-settings and entries
    config = cli.update_config(config);

    // load additional configuration settings from env
    cli.load_env(&mut config)?;

    // configure css theme and css overrides
    let theme = cli.get_theme();

    // set environment variables before running app
    cli.set_env();

    // run gui
    log::debug!("launching gui");
    let context = gui::ContextBuilder::default()
        .with_css(cli.css)
        .with_theme(theme)
        .with_config(config)
        .build(server);
    gui::run(context);

    Ok(())
}
