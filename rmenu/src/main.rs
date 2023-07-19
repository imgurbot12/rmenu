use std::ffi::OsString;
use std::fs::{read_to_string, File};
use std::io::{prelude::*, BufReader, Error, ErrorKind};

mod config;
mod gui;
mod search;
mod state;

use clap::*;
use rmenu_plugin::Entry;

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
    #[arg(short, long)]
    json: bool,
    #[arg(short, long)]
    msgpack: bool,
    #[arg(short, long)]
    run: Vec<String>,
    #[arg(short, long)]
    config: Option<OsString>,
    #[arg(long)]
    css: Vec<OsString>,
}

impl Args {
    /// Load Config based on CLI Settings
    fn config(&self) -> Result<config::Config, Error> {
        let path = match &self.config {
            Some(path) => path.to_owned(),
            None => match dirs::config_dir() {
                Some(mut dir) => {
                    dir.push("rmenu");
                    dir.push("config.toml");
                    dir.into()
                }
                None => {
                    return Err(Error::new(ErrorKind::NotFound, "$HOME not found"));
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
        toml::from_str(&cfg).map_err(|e| Error::new(ErrorKind::InvalidInput, format!("{e}")))
    }

    /// Load Entries From Input (Stdin by Default)
    fn load_default(&self) -> Result<Vec<Entry>, Error> {
        let fpath = match self.input.as_str() {
            "-" => "/dev/stdin",
            _ => &self.input,
        };
        let file = File::open(fpath)?;
        let reader = BufReader::new(file);
        let mut entries = vec![];
        for line in reader.lines() {
            let entry = serde_json::from_str::<Entry>(&line?)?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Load Entries From Specified Sources
    fn load_sources(&self, cfg: &config::Config) -> Result<Vec<Entry>, Error> {
        todo!()
    }

    /// Load Application
    pub fn parse_app() -> Result<App, Error> {
        let args = Self::parse();
        let mut config = args.config()?;
        // load css files from settings
        config.css.extend(args.css.clone());
        let mut css = vec![];
        for path in config.css.iter() {
            let src = read_to_string(path)?;
            css.push(src);
        }
        // load entries from configured sources
        let entries = match args.run.len() > 0 {
            true => args.load_sources(&config)?,
            false => args.load_default()?,
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

//TODO: add better errors with `thiserror` to add context
//TODO: improve search w/ options for regex/case-insensivity/modes?
//TODO: add secondary menu for sub-actions aside from the main action
//TODO: improve looks and css

//TODO: config
//  - default and cli accessable modules (instead of piped in)
//  - allow/disable icons (also available via CLI)
//  - custom keybindings (some available via CLI?)

fn main() -> Result<(), Error> {
    // parse cli / config / application-settings
    let app = Args::parse_app()?;
    gui::run(app);
    Ok(())
}
