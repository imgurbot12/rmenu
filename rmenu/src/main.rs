mod cache;
mod cli;
mod config;
mod exec;
mod gui;
mod search;

use clap::Parser;

static DEFAULT_THEME: &'static str = "style.css";
static DEFAULT_CONFIG: &'static str = "config.yaml";
static XDG_PREFIX: &'static str = "rmenu";

static ENV_BIN: &'static str = "RMENU";
static ENV_ACTIVE_PLUGINS: &'static str = "RMENU_ACTIVE_PLUGINS";

//TODO: remove min-length from search options in rmenu-lib

fn main() -> cli::Result<()> {
    env_logger::init();

    // export self to environment for other scripts
    let exe = rmenu_plugin::self_exe();
    std::env::set_var(ENV_BIN, exe);

    // parse cli and retrieve values for app
    let mut cli = cli::Args::parse();
    let mut config = cli.get_config()?;

    let entries = cli.get_entries(&mut config)?;

    // update config based on cli-settings and entries
    config = cli.update_config(config);

    // load additional configuration settings from env
    cli.load_env(&mut config)?;

    // configure css theme and css overrides
    let css = cli.get_css(&config);
    let theme = cli.get_theme();

    // set environment variables before running app
    cli.set_env();

    // run gui
    log::debug!("launching gui");
    let context = gui::ContextBuilder::default()
        .with_css(css)
        .with_theme(theme)
        .with_config(config)
        .with_modes(cli.show)
        .with_entries(entries)
        .with_bg_threads(cli.threads)
        .build();
    gui::run(context);

    Ok(())
}
