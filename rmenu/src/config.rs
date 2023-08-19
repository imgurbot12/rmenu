//! RMENU Configuration Implementations
use heck::AsPascalCase;
use keyboard_types::{Code, Modifiers};
use serde::{de::Error, Deserialize};
use std::collections::BTreeMap;
use std::str::FromStr;

use dioxus_desktop::tao::{
    dpi::{LogicalPosition, LogicalSize},
    window::Fullscreen,
};

// parse supported modifiers from string
fn mod_from_str(s: &str) -> Option<Modifiers> {
    match s.to_lowercase().as_str() {
        "alt" => Some(Modifiers::ALT),
        "ctrl" => Some(Modifiers::CONTROL),
        "shift" => Some(Modifiers::SHIFT),
        "super" => Some(Modifiers::SUPER),
        _ => None,
    }
}

/// Single GUI Keybind for Configuration
#[derive(Debug, Clone, PartialEq)]
pub struct Keybind {
    pub mods: Modifiers,
    pub key: Code,
}

impl Keybind {
    fn new(key: Code) -> Self {
        Self {
            mods: Modifiers::empty(),
            key,
        }
    }
}

impl FromStr for Keybind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // parse modifiers/keys from string
        let mut mods = vec![];
        let mut keys = vec![];
        for item in s.split("+") {
            let camel = format!("{}", AsPascalCase(item));
            match Code::from_str(&camel) {
                Ok(key) => keys.push(key),
                Err(_) => match mod_from_str(item) {
                    Some(keymod) => mods.push(keymod),
                    None => return Err(format!("invalid key/modifier: {item}")),
                },
            }
        }
        // generate final keybind
        let kmod = mods.into_iter().fold(Modifiers::empty(), |m1, m2| m1 | m2);
        match keys.len() {
            0 => Err(format!("no keys specified")),
            1 => Ok(Keybind {
                mods: kmod,
                key: keys.pop().unwrap(),
            }),
            _ => Err(format!("too many keys: {keys:?}")),
        }
    }
}

impl<'de> Deserialize<'de> for Keybind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Keybind::from_str(s).map_err(D::Error::custom)
    }
}

/// Global GUI Keybind Settings Options
#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct KeyConfig {
    pub exec: Vec<Keybind>,
    pub exit: Vec<Keybind>,
    pub move_next: Vec<Keybind>,
    pub move_prev: Vec<Keybind>,
    pub open_menu: Vec<Keybind>,
    pub close_menu: Vec<Keybind>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        return Self {
            exec: vec![Keybind::new(Code::Enter)],
            exit: vec![Keybind::new(Code::Escape)],
            move_next: vec![Keybind::new(Code::ArrowUp)],
            move_prev: vec![Keybind::new(Code::ArrowDown)],
            open_menu: vec![],
            close_menu: vec![],
        };
    }
}

/// GUI Desktop Window Configuration Settings
#[derive(Debug, PartialEq, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub size: LogicalSize<f64>,
    pub position: LogicalPosition<f64>,
    #[serde(default = "_true")]
    pub focus: bool,
    pub decorate: bool,
    pub transparent: bool,
    #[serde(default = "_true")]
    pub always_top: bool,
    pub fullscreen: Option<bool>,
    pub dark_mode: Option<bool>,
}

impl WindowConfig {
    /// Retrieve Desktop Compatabible Fullscreen Settings
    pub fn get_fullscreen(&self) -> Option<Fullscreen> {
        self.fullscreen.and_then(|fs| match fs {
            true => Some(Fullscreen::Borderless(None)),
            false => None,
        })
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "RMenu - App Launcher".to_owned(),
            size: LogicalSize {
                width: 700.0,
                height: 400.0,
            },
            position: LogicalPosition { x: 100.0, y: 100.0 },
            focus: true,
            decorate: false,
            transparent: false,
            always_top: true,
            fullscreen: None,
            dark_mode: None,
        }
    }
}

/// Cache Settings for Configured RMenu Plugins
#[derive(Debug, Clone, PartialEq)]
pub enum CacheSetting {
    NoCache,
    Never,
    OnLogin,
    AfterSeconds(usize),
}

impl FromStr for CacheSetting {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(Self::Never),
            "false" | "disable" | "disabled" => Ok(Self::NoCache),
            "true" | "login" | "onlogin" => Ok(Self::OnLogin),
            _ => {
                let secs: usize = s
                    .parse()
                    .map_err(|_| format!("Invalid Cache Setting: {s:?}"))?;
                Ok(Self::AfterSeconds(secs))
            }
        }
    }
}

impl<'de> Deserialize<'de> for CacheSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        CacheSetting::from_str(s).map_err(D::Error::custom)
    }
}

impl Default for CacheSetting {
    fn default() -> Self {
        Self::NoCache
    }
}

/// RMenu Data-Source Plugin Configuration
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PluginConfig {
    pub exec: Vec<String>,
    #[serde(default)]
    pub cache: CacheSetting,
    #[serde(default)]
    pub placeholder: Option<String>,
}

#[inline]
fn _true() -> bool {
    true
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub restrict: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub placeholder: Option<String>,
    #[serde(default = "_true")]
    pub use_regex: bool,
    #[serde(default = "_true")]
    pub ignore_case: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            restrict: Default::default(),
            min_length: Default::default(),
            max_length: Default::default(),
            placeholder: Default::default(),
            use_regex: true,
            ignore_case: true,
        }
    }
}

/// Global RMenu Complete Configuration
#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    pub page_size: usize,
    pub page_load: f64,
    #[serde(default = "_true")]
    pub use_icons: bool,
    #[serde(default = "_true")]
    pub use_comments: bool,
    pub search: SearchConfig,
    pub plugins: BTreeMap<String, PluginConfig>,
    pub keybinds: KeyConfig,
    pub window: WindowConfig,
    pub terminal: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            page_size: 50,
            page_load: 0.8,
            use_icons: true,
            use_comments: true,
            search: Default::default(),
            plugins: Default::default(),
            keybinds: Default::default(),
            window: Default::default(),
            terminal: Default::default(),
        }
    }
}
