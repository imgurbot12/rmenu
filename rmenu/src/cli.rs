///! CLI Argument Based Configuration and Application Setup
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::{fmt::Display, fs::read_to_string};

use clap::Parser;
use rmenu_plugin::{Entry, Message};
use thiserror::Error;

use crate::config::{cfg_replace, Config, Keybind};
use crate::gui::Entries;
use crate::{DEFAULT_CONFIG, DEFAULT_THEME, ENV_ACTIVE_PLUGINS, XDG_PREFIX};

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
    /// Limit which plugins are active
    #[arg(short, long)]
    pub show: Vec<String>,
    /// Override default configuration path
    #[arg(short, long, env = "RMENU_CONFIG")]
    config: Option<PathBuf>,
    /// Override base css theme styling
    #[arg(long, env = "RMENU_THEME")]
    theme: Option<PathBuf>,
    /// Include additional css settings
    #[arg(long, env = "RMENU_CSS")]
    css: Option<PathBuf>,

    // root config settings
    /// Override terminal command
    #[arg(long, env = "RMENU_TERMINAL")]
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
    /// Allow Selection by Mouse Hover
    #[arg(long)]
    hover_select: Option<bool>,
    /// Activate Menu Result with Single Click
    #[arg(long)]
    single_click: Option<bool>,
    /// Allow Right Click Context Menu
    #[arg(long)]
    context_menu: Option<bool>,

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
    /// Override jump-next keybind
    #[arg(long)]
    key_jump_next: Option<Vec<Keybind>>,
    /// Override jump-previous keybind
    #[arg(long)]
    key_jump_prev: Option<Vec<Keybind>>,
    /// Override next plugin keybind
    #[arg(long)]
    key_mode_next: Option<Vec<Keybind>>,
    /// Override prev plugin keybind
    #[arg(long)]
    key_mode_prev: Option<Vec<Keybind>>,

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

    // hidden vars
    #[clap(skip)]
    pub threads: Vec<std::thread::JoinHandle<()>>,
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
    #[error("Invalid Keybind Definition")]
    InvalidKeybind(String),
    #[error("Command Runtime Exception")]
    CommandError(Option<ExitStatus>),
    #[error("Invalid JSON Entry Object")]
    InvalidJson(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, RMenuError>;

impl Args {
    /// Find a specifically named file across xdg config paths
    fn find_xdg_file(&self, name: &str, base: &Option<PathBuf>) -> Option<String> {
        return base
            .clone()
            .or_else(|| {
                xdg::BaseDirectories::with_prefix(XDG_PREFIX)
                    .expect("Failed to read xdg base dirs")
                    .find_config_file(name)
            })
            .map(|f| {
                let f = f.to_string_lossy().to_string();
                shellexpand::tilde(&f).to_string()
            });
    }

    /// Load Configuration File
    pub fn get_config(&self) -> Result<Config> {
        let config = self.find_xdg_file(DEFAULT_CONFIG, &self.config);
        if let Some(path) = config {
            log::debug!("loading config: {path:?}");
            let config: Config = match read_to_string(path) {
                Ok(content) => serde_yaml::from_str(&content),
                Err(err) => {
                    log::error!("Failed to Load Config: {err:?}");
                    Ok(Config::default())
                }
            }?;
            return Ok(config);
        }
        log::error!("Failed to Load Config: no file found in xdg config paths");
        Ok(Config::default())
    }

    /// Update Configuration w/ CLI Specified Settings
    pub fn update_config(&self, mut config: Config) -> Config {
        // override basic settings
        config.terminal = self.terminal.clone().or_else(|| config.terminal);
        config.page_size = self.page_size.unwrap_or(config.page_size);
        config.page_load = self.page_load.unwrap_or(config.page_load);
        config.use_icons = self.use_icons.unwrap_or(config.use_icons);
        config.use_comments = self.use_icons.unwrap_or(config.use_comments);
        config.hover_select = self.hover_select.unwrap_or(config.hover_select);
        config.single_click = self.single_click.unwrap_or(config.single_click);
        config.context_menu = self.context_menu.unwrap_or(config.context_menu);
        // override search settings
        cfg_replace!(config.search.restrict, self.search_restrict);
        cfg_replace!(config.search.max_length, self.search_max_length, true);
        cfg_replace!(config.search.use_regex, self.search_regex, true);
        cfg_replace!(config.search.ignore_case, self.ignore_case, true);
        cfg_replace!(config.search.placeholder, self.placeholder);
        // override keybind settings
        cfg_replace!(config.keybinds.exec, self.key_exec, true);
        cfg_replace!(config.keybinds.exit, self.key_exit, true);
        cfg_replace!(config.keybinds.move_next, self.key_move_next, true);
        cfg_replace!(config.keybinds.move_prev, self.key_move_prev, true);
        cfg_replace!(config.keybinds.open_menu, self.key_open_menu, true);
        cfg_replace!(config.keybinds.close_menu, self.key_close_menu, true);
        cfg_replace!(config.keybinds.jump_next, self.key_jump_next, true);
        cfg_replace!(config.keybinds.jump_prev, self.key_jump_prev, true);
        cfg_replace!(config.keybinds.mode_next, self.key_mode_next, true);
        cfg_replace!(config.keybinds.mode_prev, self.key_move_prev, true);
        // override window settings
        cfg_replace!(config.window.title, self.title, true);
        cfg_replace!(config.window.size.width, self.width, true);
        cfg_replace!(config.window.size.height, self.height, true);
        cfg_replace!(config.window.focus, self.focus, true);
        cfg_replace!(config.window.decorate, self.decorate, true);
        cfg_replace!(config.window.transparent, self.transparent, true);
        cfg_replace!(config.window.always_top, self.always_top, true);
        cfg_replace!(config.window.fullscreen, self.fullscreen);
        config
    }

    /// Load CSS Theme or Default
    pub fn get_theme(&self) -> String {
        self.find_xdg_file(DEFAULT_THEME, &self.theme)
            .map(read_to_string)
            .map(|f| {
                f.unwrap_or_else(|err| {
                    log::error!("Failed to load CSS: {err:?}");
                    String::new()
                })
            })
            .unwrap_or_else(String::new)
    }

    /// Load Additional CSS or Default
    pub fn get_css(&self, c: &Config) -> String {
        let css = self
            .css
            .clone()
            .map(|s| s.to_string_lossy().to_string())
            .or(c.css.clone());
        if let Some(path) = css {
            let path = shellexpand::tilde(&path).to_string();
            match read_to_string(&path) {
                Ok(css) => return css,
                Err(err) => log::error!("Failed to load Theme: {err:?}"),
            }
        }
        String::new()
    }

    fn read_entries<T: Read>(
        &mut self,
        r: BufReader<T>,
        v: &mut Vec<Entry>,
        c: &mut Config,
    ) -> Result<()> {
        for line in r.lines().filter_map(|l| l.ok()) {
            match &self.format {
                Format::DMenu => v.push(Entry::echo(line.trim(), None)),
                Format::Json => {
                    let msg: Message = serde_json::from_str(&line)?;
                    match msg {
                        Message::Entry(entry) => v.push(entry),
                        Message::Options(options) => c
                            .update(&options)
                            .map_err(|s| RMenuError::InvalidKeybind(s))?,
                    }
                }
            }
        }
        Ok(())
    }

    /// Read Entries from a Configured Input
    fn load_input(&mut self, input: &str, config: &mut Config) -> Result<Vec<Entry>> {
        // retrieve input file
        let input = if input == "-" { "/dev/stdin" } else { input };
        let fpath = shellexpand::tilde(input).to_string();
        // read entries into iterator and collect
        log::info!("reading from: {fpath:?}");
        let file = File::open(fpath)?;
        let reader = BufReader::new(file);
        let mut entries = vec![];
        self.read_entries(reader, &mut entries, config)?;
        Ok(entries)
    }

    /// Read Entries from a Plugin Source
    fn load_plugins(&mut self, config: &mut Config) -> Result<Entries> {
        let mut entries = Entries::new();
        for name in self.run.clone().into_iter() {
            // retrieve plugin configuration
            log::info!("running plugin: {name:?}");
            let plugin = config
                .plugins
                .get(&name)
                .cloned()
                .ok_or_else(|| RMenuError::NoSuchPlugin(name.to_owned()))?;
            // update config w/ plugin options when available
            if let Some(options) = plugin.options.as_ref() {
                config
                    .update(options)
                    .map_err(|e| RMenuError::InvalidKeybind(e))?;
            }
            // read cache when available
            match crate::cache::read_cache(&name, &plugin) {
                Err(err) => log::error!("cache read failed: {err:?}"),
                Ok(cached) => {
                    log::debug!("plugin {name:?} loaded entries from cache");
                    entries.push((name, cached));
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
            // spawn command
            log::debug!("plugin {name:?} reading from command output");
            let mut command = Command::new(main)
                .args(&args[1..])
                .stdout(Stdio::piped())
                .spawn()?;
            let stdout = command
                .stdout
                .as_mut()
                .ok_or_else(|| RMenuError::CommandError(None))?;
            // parse and read entries into vector of results
            let reader = BufReader::new(stdout);
            let mut entry = vec![];
            self.read_entries(reader, &mut entry, config)?;
            let status = command.wait()?;
            if !status.success() {
                return Err(RMenuError::CommandError(Some(status)));
            }
            // finalize settings and save to cache
            if config.search.placeholder.is_none() {
                config.search.placeholder = plugin.placeholder.clone();
            }
            let plugin_name = name.clone();
            let write_entries = entry.clone();
            self.threads
                .push(std::thread::spawn(move || match crate::cache::write_cache(
                    &name,
                    &plugin,
                    &write_entries,
                ) {
                    Ok(_) => {}
                    Err(err) => log::error!("cache write error: {err:?}"),
                }));
            // write collected entries to main output
            entries.push((plugin_name, entry));
        }
        Ok(entries)
    }

    /// Load Entries from Enabled/Configured Entry-Sources
    pub fn get_entries(&mut self, config: &mut Config) -> Result<Entries> {
        // configure default source if none are given
        let mut input = self.input.clone();
        let mut entries = Entries::new();
        if input.is_none() && self.run.is_empty() {
            input = Some("-".to_owned());
        }
        // load entries
        if let Some(input) = input {
            entries.push(("stdin".to_owned(), self.load_input(&input, config)?));
        }

        entries.extend(self.load_plugins(config)?);
        Ok(entries)
    }

    /// Configure Environment Variables for Multi-Stage Execution
    pub fn set_env(&self) {
        let mut running = self.run.join(",");
        if let Ok(already_running) = std::env::var(ENV_ACTIVE_PLUGINS) {
            running = format!("{running},{already_running}");
        }
        std::env::set_var(ENV_ACTIVE_PLUGINS, running);
    }

    /// Load Settings from Environment Variables for Multi-Stage Execution
    pub fn load_env(&mut self, config: &mut Config) -> Result<()> {
        let env_plugins = std::env::var(ENV_ACTIVE_PLUGINS).unwrap_or_default();
        let active_plugins: Vec<&str> = env_plugins
            .split(",")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        for name in active_plugins {
            // retrieve plugin configuration
            log::info!("reloading plugin configuration for {name:?}");
            let plugin = config
                .plugins
                .get(name)
                .cloned()
                .ok_or_else(|| RMenuError::NoSuchPlugin(name.to_owned()))?;
            // update config w/ plugin options when available
            if let Some(options) = plugin.options.as_ref() {
                config
                    .update(options)
                    .map_err(|e| RMenuError::InvalidKeybind(e))?;
            }
        }
        Ok(())
    }
}
