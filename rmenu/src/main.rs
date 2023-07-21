use std::collections::VecDeque;
use std::fmt::Display;
use std::fs::{read_to_string, File};
use std::io::{self, prelude::*, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;

mod config;
mod exec;
mod gui;
mod search;
mod state;

use clap::Parser;
use rmenu_plugin::Entry;
use thiserror::Error;

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
    #[arg(short, long)]
    config: Option<String>,
    #[arg(long)]
    css: Vec<String>,
}

impl Args {
    /// Load Config based on CLI Settings
    fn config(&self) -> Result<config::Config, RMenuError> {
        let path = match &self.config {
            Some(path) => path.to_owned(),
            None => match dirs::config_dir() {
                Some(mut dir) => {
                    dir.push("rmenu");
                    dir.push("config.yaml");
                    dir.to_string_lossy().to_string()
                }
                None => {
                    return Err(RMenuError::HomeNotFound);
                }
            },
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
        println!("{cfg:?}");
        // execute commands to get a list of entries
        let mut entries = vec![];
        for plugin in self.run.iter() {
            log::debug!("running plugin: {plugin}");
            // retrieve plugin command arguments
            let Some(args) = cfg.plugins.get(plugin) else {
                return Err(RMenuError::NoSuchPlugin(plugin.to_owned()));
            };
            // build command
            let mut cmdargs: VecDeque<String> = args
                .iter()
                .map(|arg| shellexpand::tilde(arg).to_string())
                .collect();
            let Some(main) = cmdargs.pop_front() else {
                return Err(RMenuError::InvalidPlugin(plugin.to_owned()));
            };
            let mut cmd = Command::new(main);
            for arg in cmdargs.iter() {
                cmd.arg(arg);
            }
            // spawn command
            let mut proc = cmd.stdout(Stdio::piped()).spawn()?;
            let stdout = proc
                .stdout
                .as_mut()
                .ok_or_else(|| RMenuError::CommandError(args.clone().into(), None))?;
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
                    args.clone().into(),
                    Some(status.clone()),
                ));
            }
        }
        Ok(entries)
    }

    /// Load Application
    pub fn parse_app() -> Result<App, RMenuError> {
        let args = Self::parse();
        let mut config = args.config()?;
        // load css files from settings
        config.css.extend(args.css.clone());
        let mut css = vec![];
        for path in config.css.iter() {
            let path = shellexpand::tilde(path).to_string();
            let src = read_to_string(path)?;
            css.push(src);
        }
        // load entries from configured sources
        let entries = match args.run.len() > 0 {
            true => args.load_sources(&config)?,
            false => args.load_default(&config)?,
        };
        // generate app object
        return Ok(App {
            css: css.join("\n"),
            name: "rmenu".to_owned(),
            entries,
            config,
        });
    }
}

//TODO: improve search w/ modes?
//TODO: improve looks and css

//TODO: config
//  - default and cli accessable modules (instead of piped in)
//  - should resolve arguments/paths with home expansion

//TODO: add exit key (Esc by default?) - part of keybindings

fn main() -> Result<(), RMenuError> {
    // parse cli / config / application-settings
    let app = Args::parse_app()?;
    gui::run(app);
    Ok(())
}