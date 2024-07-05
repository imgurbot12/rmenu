use std::str::FromStr;

use dioxus::events::{Code, Modifiers};
use serde::de::Error;
use serde::Deserialize;

#[inline]
fn _true() -> bool {
    true
}

/// Global RMenu Complete Configuration
#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    pub page_size: usize,
    pub page_load: f64,
    pub jump_dist: usize,
    #[serde(default = "_true")]
    pub use_icons: bool,
    #[serde(default = "_true")]
    pub use_comments: bool,
    pub hover_select: bool,
    pub single_click: bool,
    pub search: SearchConfig,
    pub window: WindowConfig,
    pub keybinds: KeyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            page_size: 50,
            page_load: 0.8,
            jump_dist: 5,
            use_icons: true,
            use_comments: true,
            hover_select: false,
            single_click: false,
            search: Default::default(),
            window: Default::default(),
            keybinds: Default::default(),
        }
    }
}

#[inline]
fn _maxlen() -> usize {
    999
}

/// Search Configuration Settings
#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub restrict: Option<String>,
    #[serde(default = "_maxlen")]
    pub max_length: usize,
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
            max_length: 999,
            placeholder: Default::default(),
            use_regex: true,
            ignore_case: true,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct WindowSize {
    width: f64,
    height: f64,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 400.0,
        }
    }
}

/// Window Configuration Settings
#[derive(Debug, PartialEq, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub size: WindowSize,
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
    pub fn logical_size(&self) -> dioxus_desktop::LogicalSize<f64> {
        dioxus_desktop::LogicalSize {
            width: self.size.width,
            height: self.size.height,
        }
    }
    pub fn get_fullscreen(&self) -> Option<dioxus_desktop::tao::window::Fullscreen> {
        self.fullscreen.and_then(|fs| match fs {
            true => Some(dioxus_desktop::tao::window::Fullscreen::Borderless(None)),
            false => None,
        })
    }
    pub fn get_theme(&self) -> Option<dioxus_desktop::tao::window::Theme> {
        match self.dark_mode {
            Some(dark) => match dark {
                true => Some(dioxus_desktop::tao::window::Theme::Dark),
                false => Some(dioxus_desktop::tao::window::Theme::Light),
            },
            None => None,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "RMenu - Application Launcher".to_owned(),
            size: Default::default(),
            focus: true,
            decorate: false,
            transparent: false,
            always_top: true,
            fullscreen: None,
            dark_mode: None,
        }
    }
}

/// GUI Keybind Settings Options
#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct KeyConfig {
    pub exec: Vec<Keybind>,
    pub exit: Vec<Keybind>,
    pub move_next: Vec<Keybind>,
    pub move_prev: Vec<Keybind>,
    pub open_menu: Vec<Keybind>,
    pub close_menu: Vec<Keybind>,
    pub jump_next: Vec<Keybind>,
    pub jump_prev: Vec<Keybind>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        return Self {
            exec: vec![Keybind::new(Code::Enter)],
            exit: vec![Keybind::new(Code::Escape)],
            move_next: vec![Keybind::new(Code::ArrowDown)],
            move_prev: vec![Keybind::new(Code::ArrowUp)],
            open_menu: vec![],
            close_menu: vec![],
            jump_next: vec![Keybind::new(Code::PageDown)],
            jump_prev: vec![Keybind::new(Code::PageUp)],
        };
    }
}

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
        // parse keys & modifiers from string
        let mut mods = vec![];
        let mut keys = vec![];
        for item in s.split("+") {
            let camel = format!("{}", heck::AsPascalCase(item));
            match Code::from_str(&camel) {
                Ok(key) => keys.push(key),
                Err(_) => match mod_from_str(item) {
                    Some(keymod) => mods.push(keymod),
                    None => return Err(format!("Invalid key/modifier: {item}")),
                },
            }
        }
        // generate final keybind
        let kmod = mods.into_iter().fold(Modifiers::empty(), |m1, m2| m1 | m2);
        match keys.len() {
            0 => Err(format!("No keys specified")),
            1 => Ok(Keybind {
                mods: kmod,
                key: keys.pop().unwrap(),
            }),
            _ => Err(format!("Too many keys: {keys:?}")),
        }
    }
}

macro_rules! de_fromstr {
    ($s:ident) => {
        impl<'de> Deserialize<'de> for $s {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s: &str = Deserialize::deserialize(deserializer)?;
                $s::from_str(s).map_err(D::Error::custom)
            }
        }
    };
}

// implement `Deserialize` using `FromStr`
de_fromstr!(Keybind);
