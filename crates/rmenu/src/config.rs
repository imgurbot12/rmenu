use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use rmenu_plugin::ModuleConfig;
use serde::{Deserialize, Serialize};
use shellexpand::tilde;

/* Variables */

static HOME: &str = "HOME";
static XDG_CONIFG_HOME: &str = "XDG_CONIFG_HOME";

/* Types */

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub prefix: String,
    pub path: String,
    pub config: ModuleConfig,
}

#[derive(Serialize, Deserialize)]
pub struct RMenuConfig {
    pub terminal: String,
    pub icon_size: f32,
    pub centered: Option<bool>,
    pub window_pos: Option<[f32; 2]>,
    pub window_size: Option<[f32; 2]>,
    pub result_size: Option<usize>,
    pub decorate_window: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub rmenu: RMenuConfig,
    pub plugins: HashMap<String, PluginConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rmenu: RMenuConfig {
                terminal: "foot".to_owned(),
                icon_size: 20.0,
                centered: Some(true),
                window_pos: None,
                window_size: Some([500.0, 300.0]),
                result_size: Some(15),
                decorate_window: false,
            },
            plugins: HashMap::new(),
        }
    }
}

/* Functions */

#[inline]
fn get_config_dir() -> PathBuf {
    if let Ok(config) = env::var(XDG_CONIFG_HOME) {
        return Path::new(&config).join("rmenu").to_path_buf();
    }
    if let Ok(home) = env::var(HOME) {
        return Path::new(&home).join(".config").join("rmenu").to_path_buf();
    }
    panic!("cannot find config directory!")
}

pub fn load_config(path: Option<String>) -> Config {
    // determine path based on arguments
    let fpath = match path.clone() {
        Some(path) => Path::new(&tilde(&path).to_string()).to_path_buf(),
        None => get_config_dir().join("config.toml"),
    };
    // read existing file or write default and read it back
    let mut config = match fpath.exists() {
        false => {
            // write default config to standard location
            let config = Config::default();
            if path.is_none() {
                let dir = get_config_dir();
                if !dir.exists() {
                    fs::create_dir(dir).expect("failed to make config dir");
                }
                let default = toml::to_string(&config).unwrap();
                fs::write(fpath, default).expect("failed to write default config");
            }
            config
        }
        true => {
            let config = fs::read_to_string(fpath).expect("unable to read config");
            toml::from_str(&config).expect("broken config")
        }
    };
    // expand plugin paths
    for plugin in config.plugins.values_mut() {
        plugin.path = tilde(&plugin.path).to_string();
    }
    config
}
