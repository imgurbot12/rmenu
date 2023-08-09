use std::collections::VecDeque;
use std::fmt::Display;
use std::fs::{read_to_string, File};
use std::io::{self, prelude::*, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;

mod cache;
mod config;
mod exec;
mod gui;
mod image;
mod search;
mod state;

use clap::Parser;
use rmenu_plugin::Entry;
use thiserror::Error;

static CONFIG_DIR: &'static str = "~/.config/rmenu/";
static DEFAULT_CSS: &'static str = "~/.config/rmenu/style.css";
static DEFAULT_CONFIG: &'static str = "~/.config/rmenu/config.yaml";
static DEFAULT_CSS_CONTENT: &'static str = include_str!("../public/default.css");

#[derive(Debug, Clone)]
pub enum Format {
    Json,
    MsgPack,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "json" => Ok(Format::Json),
            "msgpack" => Ok(Format::MsgPack),
            _ => Err("No Such Format".to_owned()),
        }
    }
}

#[derive(Error, Debug)]
pub enum RMenuError {
    #[error("$HOME not found")]
    HomeNotFound,
    #[error("Invalid Config")]
    InvalidConfig(#[from] serde_yaml::Error),
    #[error("File Error")]
    FileError(#[from] io::Error),
    #[error("No Such Plugin")]
    NoSuchPlugin(String),
    #[error("Invalid Plugin Specified")]
    InvalidPlugin(String),
    #[error("Command Runtime Exception")]
    CommandError(Vec<String>, Option<ExitStatus>),
    #[error("Invalid JSON Entry Object")]
    InvalidJson(#[from] serde_json::Error),
}

/// Application State for GUI
#[derive(Debug, PartialEq)]
pub struct App {
    css: String,
    name: String,
    entries: Vec<Entry>,
    config: config::Config,
}

/// Rofi Clone (Built with Rust)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[arg(short, long, default_value_t=String::from("-"))]
    input: String,
    #[arg(short, long, default_value_t=Format::Json)]
    format: Format,
    #[arg(short, long)]
    run: Vec<String>,
    #[arg(long)]
    regex: Option<bool>,
    #[arg(short, long)]
    config: Option<String>,
    #[arg(long)]
    css: Option<String>,
}

impl Args {
    /// Load Config based on CLI Settings
    fn config(&self) -> Result<config::Config, RMenuError> {
        let path = match &self.config {
            Some(path) => path.to_owned(),
            None => shellexpand::tilde(DEFAULT_CONFIG).to_string(),
        };
        log::debug!("loading config from {path:?}");
        let cfg = match read_to_string(path) {
            Ok(cfg) => cfg,
            Err(err) => {
                log::error!("failed to load config: {err:?}");
                return Ok(config::Config::default());
            }
        };
        serde_yaml::from_str(&cfg).map_err(|e| RMenuError::InvalidConfig(e))
    }

    /// Read single entry from incoming line object
    fn readentry(&self, cfg: &config::Config, line: &str) -> Result<Entry, RMenuError> {
        let mut entry = match self.format {
            Format::Json => serde_json::from_str::<Entry>(line)?,
            Format::MsgPack => todo!(),
        };
        if !cfg.use_icons {
            entry.icon = None;
        }
        Ok(entry)
    }

    /// Load Entries From Input (Stdin by Default)
    fn load_default(&self, cfg: &config::Config) -> Result<Vec<Entry>, RMenuError> {
        let fpath = match self.input.as_str() {
            "-" => "/dev/stdin",
            _ => &self.input,
        };
        log::info!("reading from {fpath:?}");
        let file = File::open(fpath).map_err(|e| RMenuError::FileError(e))?;
        let reader = BufReader::new(file);
        let mut entries = vec![];
        for line in reader.lines() {
            let entry = self.readentry(cfg, &line?)?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Load Entries From Specified Sources
    fn load_sources(&self, cfg: &config::Config) -> Result<Vec<Entry>, RMenuError> {
        log::debug!("config: {cfg:?}");
        // execute commands to get a list of entries
        let mut entries = vec![];
        for name in self.run.iter() {
            log::debug!("running plugin: {name}");
            // retrieve plugin command arguments
            let plugin = cfg
                .plugins
                .get(name)
                .ok_or_else(|| RMenuError::NoSuchPlugin(name.to_owned()))?;
            // attempt to read cache rather than run command
            match cache::read_cache(name, plugin) {
                Ok(cached) => {
                    entries.extend(cached);
                    continue;
                }
                Err(err) => log::error!("cache read error: {err:?}"),
            }
            // build command
            let mut cmdargs: VecDeque<String> = plugin
                .exec
                .iter()
                .map(|arg| shellexpand::tilde(arg).to_string())
                .collect();
            let main = cmdargs
                .pop_front()
                .ok_or_else(|| RMenuError::InvalidPlugin(name.to_owned()))?;
            let mut cmd = Command::new(main);
            for arg in cmdargs.iter() {
                cmd.arg(arg);
            }
            // spawn command
            let mut proc = cmd.stdout(Stdio::piped()).spawn()?;
            let stdout = proc
                .stdout
                .as_mut()
                .ok_or_else(|| RMenuError::CommandError(plugin.exec.clone().into(), None))?;
            let reader = BufReader::new(stdout);
            // read output line by line and parse content
            for line in reader.lines() {
                let entry = self.readentry(cfg, &line?)?;
                entries.push(entry);
            }
            // check status of command on exit
            let status = proc.wait()?;
            if !status.success() {
                return Err(RMenuError::CommandError(
                    plugin.exec.clone().into(),
                    Some(status.clone()),
                ));
            }
            // write cache for entries collected
            match cache::write_cache(name, plugin, &entries) {
                Ok(_) => {}
                Err(err) => log::error!("cache write error: {err:?}"),
            };
        }
        Ok(entries)
    }

    /// Load Application
    pub fn parse_app() -> Result<App, RMenuError> {
        let args = Self::parse();
        let mut config = args.config()?;
        // load css files from settings
        let csspath = args.css.clone().unwrap_or_else(|| DEFAULT_CSS.to_owned());
        let csspath = shellexpand::tilde(&csspath).to_string();
        let css = match read_to_string(csspath) {
            Ok(css) => css,
            Err(err) => {
                log::error!("failed to load css: {err:?}");
                "".to_owned()
            }
        };
        // load entries from configured sources
        let entries = match args.run.len() > 0 {
            true => args.load_sources(&config)?,
            false => args.load_default(&config)?,
        };
        // update configuration based on cli
        config.use_icons = config.use_icons && entries.iter().any(|e| e.icon.is_some());
        config.search_regex = args.regex.unwrap_or(config.search_regex);
        // generate app object
        return Ok(App {
            css,
            name: "rmenu".to_owned(),
            entries,
            config,
        });
    }
}

//TODO: improve search w/ modes?
//TODO: improve looks and css

fn main() -> Result<(), RMenuError> {
    // enable log and set default level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    // parse cli / config / application-settings
    let app = Args::parse_app()?;
    // change directory to configuration dir
    let cfgdir = shellexpand::tilde(CONFIG_DIR).to_string();
    if let Err(err) = std::env::set_current_dir(&cfgdir) {
        log::error!("failed to change directory: {err:?}");
    }
    // run gui
    gui::run(app);
    Ok(())
}
