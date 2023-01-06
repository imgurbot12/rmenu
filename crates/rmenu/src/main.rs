use clap::Parser;

mod config;
mod gui;
mod plugins;

use config::{load_config, PluginConfig};
use gui::launch_gui;
use plugins::Plugins;

/* Types */

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// configuration file to read from
    #[arg(short, long)]
    pub config: Option<String>,
    /// terminal command override
    #[arg(short, long)]
    pub term: Option<String>,
    /// declared and enabled plugin modes
    #[arg(short, long)]
    pub show: Option<Vec<String>>,
}

fn main() {
    // parse cli-args and use it to load the config
    let args = Args::parse();
    let mut config = load_config(args.config);
    // update config based on other cli-args
    if let Some(term) = args.term.as_ref() {
        config.rmenu.terminal = term.to_owned()
    }
    // load relevant plugins based on configured options
    let enabled = args.show.unwrap_or_else(|| vec!["drun".to_owned()]);
    let plugin_configs: Vec<PluginConfig> = config
        .plugins
        .clone()
        .into_iter()
        .filter(|(k, _)| enabled.contains(k))
        .map(|(_, v)| v)
        .collect();
    // error if plugins-list is empty
    if plugin_configs.len() != enabled.len() {
        let missing: Vec<&String> = enabled
            .iter()
            .filter(|p| !config.plugins.contains_key(p.as_str()))
            .collect();
        panic!("no plugin configurations for: {:?}", missing);
    }
    // spawn gui instance w/ config and enabled plugins
    let plugins = Plugins::new(enabled, plugin_configs);
    launch_gui(config, plugins).expect("gui crashed")
}
