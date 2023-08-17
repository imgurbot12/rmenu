use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::{fmt::Display, fs::read_to_string};

use clap::Parser;
use rmenu_plugin::Entry;
use thiserror::Error;

use crate::config::{Config, Keybind};
use crate::{DEFAULT_CONFIG, DEFAULT_CSS};

/// Allowed Formats for Entry Ingestion
#[derive(Debug, Clone)]
pub enum Format {
    Json,
    DMenu,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "json" => Ok(Format::Json),
            "dmenu" => Ok(Format::DMenu),
            _ => Err("No Such Format".to_owned()),
        }
    }
}

/// Dynamic Applicaiton-Menu Tool (Built with Rust)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    // simple configuration arguments
    /// Filepath for entry input
    #[arg(short, long)]
    input: Option<String>,
    /// Format to accept entries
    #[arg(short, long, default_value_t=Format::Json)]
    format: Format,
    /// Plugins to run
    #[arg(short, long)]
    run: Vec<String>,
    /// Override default configuration path
    #[arg(short, long)]
    config: Option<String>,
    /// Override base css styling
    #[arg(long, default_value_t=String::from(DEFAULT_CSS))]
    css: String,
    /// Include additional css settings for themeing
    #[arg(long)]
    theme: Option<String>,

    // root config settings
    /// Override terminal command
    #[arg(long)]
    terminal: Option<String>,
    /// Number of results to include for each page
    #[arg(long)]
    page_size: Option<usize>,
    /// Control ratio on when to load next page
    #[arg(long)]
    page_load: Option<f64>,
    /// Force enable/disable comments
    #[arg(long)]
    use_icons: Option<bool>,
    /// Force enable/disable comments
    #[arg(long)]
    use_comments: Option<bool>,

    // search settings
    /// Enforce Regex Pattern on Search
    #[arg(long)]
    search_restrict: Option<String>,
    /// Enforce Minimum Length on Search
    #[arg(long)]
    search_min_length: Option<usize>,
    /// Enforce Maximum Length on Search
    #[arg(long)]
    search_max_length: Option<usize>,
    /// Force enable/disable regex in search
    #[arg(long)]
    search_regex: Option<bool>,
    /// Force enable/disable ignore-case in search
    #[arg(long)]
    ignore_case: Option<bool>,
    /// Override placeholder in searchbar
    #[arg(short, long)]
    placeholder: Option<String>,

    // keybinding settings
    /// Override exec keybind
    #[arg(long)]
    key_exec: Option<Vec<Keybind>>,
    /// Override exit keybind
    #[arg(long)]
    key_exit: Option<Vec<Keybind>>,
    /// Override move-next keybind
    #[arg(long)]
    key_move_next: Option<Vec<Keybind>>,
    /// Override move-previous keybind
    #[arg(long)]
    key_move_prev: Option<Vec<Keybind>>,
    /// Override open-menu keybind
    #[arg(long)]
    key_open_menu: Option<Vec<Keybind>>,
    /// Override close-menu keybind
    #[arg(long)]
    key_close_menu: Option<Vec<Keybind>>,

    //window settings
    /// Override Window Title
    #[arg(long)]
    title: Option<String>,
    /// Override Window Width
    #[arg(long)]
    width: Option<f64>,
    /// Override Window Height
    #[arg(long)]
    height: Option<f64>,
    /// Override Window X Position
    #[arg(long)]
    xpos: Option<f64>,
    /// Override Window Y Position
    #[arg(long)]
    ypos: Option<f64>,
    /// Override Window Focus on Startup
    #[arg(long)]
    focus: Option<bool>,
    /// Override Window Decoration
    #[arg(long)]
    decorate: Option<bool>,
    /// Override Window Transparent
    #[arg(long)]
    transparent: Option<bool>,
    /// Override Window Always-On-Top
    #[arg(long)]
    always_top: Option<bool>,
    /// Override Fullscreen Settings
    #[arg(long)]
    fullscreen: Option<bool>,
}

#[derive(Error, Debug)]
pub enum RMenuError {
    #[error("Invalid Config")]
    InvalidConfig(#[from] serde_yaml::Error),
    #[error("File Error")]
    FileError(#[from] std::io::Error),
    #[error("No Such Plugin")]
    NoSuchPlugin(String),
    #[error("Invalid Plugin Specified")]
    InvalidPlugin(String),
    #[error("Command Runtime Exception")]
    CommandError(Option<ExitStatus>),
    #[error("Invalid JSON Entry Object")]
    InvalidJson(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, RMenuError>;
type MaybeEntry = Result<Entry>;

macro_rules! cli_replace {
    ($key:expr, $repl:expr) => {
        if $repl.is_some() {
            $key = $repl.clone();
        }
    };
    ($key:expr, $repl:expr, true) => {
        if let Some(value) = $repl.as_ref() {
            $key = value.to_owned();
        }
    };
}

impl Args {
    /// Load Configuration File and Update w/ Argument Overrides
    pub fn get_config(&self) -> Result<Config> {
        // read configuration
        let path = self
            .config
            .as_ref()
            .map(|v| v.as_str())
            .unwrap_or(DEFAULT_CONFIG);
        let path = shellexpand::tilde(path).to_string();
        let mut config: Config = match read_to_string(path) {
            Ok(content) => serde_yaml::from_str(&content),
            Err(err) => {
                log::error!("Failed to Load Config: {err:?}");
                Ok(Config::default())
            }
        }?;
        // override basic settings
        config.terminal = self.terminal.clone().or_else(|| config.terminal);
        config.page_size = self.page_size.unwrap_or(config.page_size);
        config.page_load = self.page_load.unwrap_or(config.page_load);
        config.use_icons = self.use_icons.unwrap_or(config.use_icons);
        config.use_comments = self.use_icons.unwrap_or(config.use_comments);
        // override search settings
        cli_replace!(config.search.restrict, self.search_restrict);
        cli_replace!(config.search.min_length, self.search_min_length);
        cli_replace!(config.search.max_length, self.search_max_length);
        cli_replace!(config.search.use_regex, self.search_regex, true);
        cli_replace!(config.search.ignore_case, self.ignore_case, true);
        cli_replace!(config.search.placeholder, self.placeholder);
        // override keybind settings
        cli_replace!(config.keybinds.exec, self.key_exec, true);
        cli_replace!(config.keybinds.exit, self.key_exit, true);
        cli_replace!(config.keybinds.move_next, self.key_move_next, true);
        cli_replace!(config.keybinds.move_prev, self.key_move_prev, true);
        cli_replace!(config.keybinds.open_menu, self.key_open_menu, true);
        cli_replace!(config.keybinds.close_menu, self.key_close_menu, true);
        // override window settings
        cli_replace!(config.window.title, self.title, true);
        cli_replace!(config.window.size.width, self.width, true);
        cli_replace!(config.window.size.height, self.height, true);
        cli_replace!(config.window.position.x, self.xpos, true);
        cli_replace!(config.window.position.y, self.ypos, true);
        cli_replace!(config.window.focus, self.focus, true);
        cli_replace!(config.window.decorate, self.decorate, true);
        cli_replace!(config.window.transparent, self.transparent, true);
        cli_replace!(config.window.always_top, self.always_top, true);
        cli_replace!(config.window.fullscreen, self.fullscreen);
        Ok(config)
    }

    /// Load CSS or Default
    pub fn get_css(&self) -> String {
        let path = shellexpand::tilde(&self.css).to_string();
        match read_to_string(&path) {
            Ok(css) => css,
            Err(err) => {
                log::error!("Failed to load CSS: {err:?}");
                String::new()
            }
        }
    }

    /// Load CSS Theme or Default
    pub fn get_theme(&self) -> String {
        if let Some(theme) = self.theme.as_ref() {
            let path = shellexpand::tilde(&theme).to_string();
            match read_to_string(&path) {
                Ok(theme) => return theme,
                Err(err) => log::error!("Failed to load Theme: {err:?}"),
            }
        }
        String::new()
    }

    /// Read Entries Contained within the Given Reader
    fn read_entries<T: Read>(&self, reader: BufReader<T>) -> impl Iterator<Item = MaybeEntry> {
        let format = self.format.clone();
        reader
            .lines()
            .filter_map(|l| l.ok())
            .map(move |l| match format {
                Format::Json => serde_json::from_str(&l).map_err(|e| RMenuError::InvalidJson(e)),
                Format::DMenu => Ok(Entry::echo(l.trim(), None)),
            })
    }

    /// Read Entries from a Configured Input
    fn load_input(&self, input: &str) -> Result<Vec<Entry>> {
        // retrieve input file
        let input = if input == "-" { "/dev/stdin" } else { input };
        let fpath = shellexpand::tilde(input).to_string();
        // read entries into iterator and collect
        log::info!("reading from: {fpath:?}");
        let file = File::open(fpath)?;
        let reader = BufReader::new(file);
        let mut entries = vec![];
        for entry in self.read_entries(reader) {
            entries.push(entry?);
        }
        Ok(entries)
    }

    /// Read Entries from a Plugin Source
    fn load_plugins(&self, config: &mut Config) -> Result<Vec<Entry>> {
        let mut entries = vec![];
        for name in self.run.iter() {
            // retrieve plugin configuration
            log::info!("running plugin: {name:?}");
            let plugin = config
                .plugins
                .get(name)
                .ok_or_else(|| RMenuError::NoSuchPlugin(name.to_owned()))?;
            // read cache when available
            match crate::cache::read_cache(name, plugin) {
                Err(err) => log::error!("cache read failed: {err:?}"),
                Ok(cached) => {
                    entries.extend(cached);
                    continue;
                }
            }
            // build command arguments
            let args: Vec<String> = plugin
                .exec
                .iter()
                .map(|s| shellexpand::tilde(s).to_string())
                .collect();
            let main = args
                .get(0)
                .ok_or_else(|| RMenuError::InvalidPlugin(name.to_owned()))?;
            // spawn command and handle command entries
            let mut command = Command::new(main)
                .args(&args[1..])
                .stdout(Stdio::piped())
                .spawn()?;
            let stdout = command
                .stdout
                .as_mut()
                .ok_or_else(|| RMenuError::CommandError(None))?;
            let reader = BufReader::new(stdout);
            for entry in self.read_entries(reader) {
                entries.push(entry?);
            }
            let status = command.wait()?;
            if !status.success() {
                return Err(RMenuError::CommandError(Some(status)));
            }
            // finalize settings and save to cache
            if config.search.placeholder.is_none() {
                config.search.placeholder = plugin.placeholder.clone();
            }
            match crate::cache::write_cache(name, plugin, &entries) {
                Ok(_) => {}
                Err(err) => log::error!("cache write error: {err:?}"),
            }
        }
        Ok(entries)
    }

    /// Load Entries from Enabled/Configured Entry-Sources
    pub fn get_entries(&self, config: &mut Config) -> Result<Vec<Entry>> {
        // configure default source if none are given
        let mut input = self.input.clone();
        let mut entries = vec![];
        if input.is_none() && self.run.is_empty() {
            input = Some("-".to_owned());
        }
        // load entries
        if let Some(input) = input {
            entries.extend(self.load_input(&input)?);
        }
        entries.extend(self.load_plugins(config)?);
        Ok(entries)
    }
}
