//! RMENU Configuration Implementations
use heck::AsPascalCase;
use keyboard_types::{Code, Modifiers};
use serde::{de::Error, Deserialize};
use std::collections::BTreeMap;
use std::str::FromStr;

use dioxus_desktop::tao::dpi::{LogicalPosition, LogicalSize};

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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Deserialize)]
pub struct KeyConfig {
    pub exec: Vec<Keybind>,
    pub exit: Vec<Keybind>,
    pub move_up: Vec<Keybind>,
    pub move_down: Vec<Keybind>,
    #[serde(default)]
    pub open_menu: Vec<Keybind>,
    #[serde(default)]
    pub close_menu: Vec<Keybind>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        return Self {
            exec: vec![Keybind::new(Code::Enter)],
            exit: vec![Keybind::new(Code::Escape)],
            move_up: vec![Keybind::new(Code::ArrowUp)],
            move_down: vec![Keybind::new(Code::ArrowDown)],
            open_menu: vec![],
            close_menu: vec![],
        };
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub size: LogicalSize<f64>,
    pub position: LogicalPosition<f64>,
    pub focus: bool,
    pub decorate: bool,
    pub transparent: bool,
    pub always_top: bool,
    pub dark_mode: Option<bool>,
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
            dark_mode: None,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub css: Vec<String>,
    pub use_icons: bool,
    pub search_regex: bool,
    pub ignore_case: bool,
    #[serde(default)]
    pub plugins: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub keybinds: KeyConfig,
    #[serde(default)]
    pub window: WindowConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: vec![],
            use_icons: true,
            search_regex: false,
            ignore_case: true,
            plugins: Default::default(),
            keybinds: Default::default(),
            window: Default::default(),
        }
    }
}
