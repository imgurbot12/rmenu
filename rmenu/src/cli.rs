///! CLI Argument Based Configuration and Application Setup
use std::fs::read_to_string;

use clap::Parser;

use crate::config::{cfg_replace, Config, Format, Keybind};
use crate::server::{RMenuError, Result};
use crate::{DEFAULT_CONFIG, DEFAULT_THEME, ENV_ACTIVE_PLUGINS, XDG_PREFIX};

/// Dynamic Applicaiton-Menu Tool (Built with Rust)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    // simple configuration arguments
    /// Filepath for entry input
    #[arg(short, long)]
    pub input: Option<String>,
    /// Format to accept entries
    #[arg(short, long, default_value_t=Format::Json)]
    pub format: Format,
    /// Plugins to run
    #[arg(short, long)]
    pub run: Vec<String>,
    /// Limit which plugins are active
    #[arg(short, long)]
    pub show: Vec<String>,
    /// Override default configuration path
    #[arg(short, long, env = "RMENU_CONFIG")]
    config: Option<String>,
    /// Override base css theme styling
    #[arg(long, env = "RMENU_THEME")]
    theme: Option<String>,
    /// Include additional css settings
    #[arg(long, env = "RMENU_CSS")]
    pub css: Option<String>,

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
}

impl Args {
    /// Find a specifically named file across xdg config paths
    #[cfg(target_os = "windows")]
    fn find_config(&self, name: &str, base: &Option<String>) -> Option<String> {
        return base.clone().or_else(|| {
            let mut cfg = dirs::config_dir().expect("failed to find windows home directory");
            cfg.push(XDG_PREFIX);
            cfg.push(name);
            Some(cfg.to_string_lossy().to_string())
        });
    }

    /// Find a specifically named file across xdg config paths
    #[cfg(not(target_os = "windows"))]
    fn find_config(&self, name: &str, base: &Option<String>) -> Option<String> {
        return base.clone().or_else(|| {
            xdg::BaseDirectories::with_prefix(XDG_PREFIX)
                .expect("Failed to read xdg base dirs")
                .find_config_file(name)
                .map(|f| f.to_string_lossy().to_string())
        });
    }

    /// Load Configuration File
    pub fn get_config(&self) -> Result<Config> {
        let config = self.find_config(DEFAULT_CONFIG, &self.config);
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
        config.use_comments = self.use_comments.unwrap_or(config.use_comments);
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
    pub fn get_theme(&self) -> Option<String> {
        self.find_config(DEFAULT_THEME, &self.theme)
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
