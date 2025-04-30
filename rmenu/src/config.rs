///! File Based Configuration for RMenu
use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

use rmenu_plugin::Options;

use dioxus::events::{Code, Modifiers};
use serde::de::Error;
use serde::Deserialize;

#[inline]
fn _true() -> bool {
    true
}

/// Global RMenu Complete Configuration
#[derive(Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub css: Option<String>,
    pub terminal: Option<String>,
    pub page_size: usize,
    pub page_load: f64,
    pub jump_dist: usize,
    #[serde(default = "_true")]
    pub use_icons: bool,
    #[serde(default = "_true")]
    pub use_comments: bool,
    pub hover_select: bool,
    pub single_click: bool,
    pub context_menu: bool,
    pub search: SearchConfig,
    pub window: WindowConfig,
    pub keybinds: KeyConfig,
    pub plugins: BTreeMap<String, PluginConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: None,
            terminal: None,
            page_size: 50,
            page_load: 0.8,
            jump_dist: 5,
            use_icons: true,
            use_comments: true,
            hover_select: false,
            single_click: false,
            context_menu: false,
            search: Default::default(),
            window: Default::default(),
            keybinds: Default::default(),
            plugins: Default::default(),
        }
    }
}

impl Config {
    /// Update Configuration from Options Object
    pub fn update(&mut self, options: &Options) -> Result<(), String> {
        cfg_replace!(self.css, options.css);
        cfg_replace!(self.page_size, options.page_size, true);
        cfg_replace!(self.page_load, options.page_load, true);
        cfg_replace!(self.jump_dist, options.jump_dist, true);
        cfg_replace!(self.hover_select, options.hover_select, true);
        cfg_replace!(self.single_click, options.single_click, true);
        cfg_replace!(self.context_menu, options.context_menu, true);
        cfg_replace!(self.use_icons, options.use_icons, true);
        cfg_replace!(self.use_comments, options.use_comments, true);
        // search settings
        cfg_replace!(self.search.placeholder, options.placeholder);
        cfg_replace!(self.search.restrict, options.search_restrict);
        cfg_replace!(self.search.max_length, options.search_max_length, true);
        // keybind settings
        cfg_keybind!(self.keybinds.exec, options.key_exec);
        cfg_keybind!(self.keybinds.exit, options.key_exit);
        cfg_keybind!(self.keybinds.move_next, options.key_move_next);
        cfg_keybind!(self.keybinds.move_prev, options.key_move_prev);
        cfg_keybind!(self.keybinds.open_menu, options.key_open_menu);
        cfg_keybind!(self.keybinds.close_menu, options.key_close_menu);
        cfg_keybind!(self.keybinds.jump_next, options.key_jump_next);
        cfg_keybind!(self.keybinds.jump_prev, options.key_jump_prev);
        cfg_keybind!(self.keybinds.mode_next, options.key_mode_next);
        cfg_keybind!(self.keybinds.mode_prev, options.key_mode_prev);
        // window settings
        cfg_replace!(self.window.title, options.title, true);
        cfg_replace!(self.window.decorate, options.decorate, true);
        cfg_replace!(self.window.fullscreen, options.fullscreen);
        cfg_replace!(self.window.transparent, options.transparent, true);
        cfg_replace!(self.window.size.width, options.window_width, true);
        cfg_replace!(self.window.size.height, options.window_height, true);
        Ok(())
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
    pub width: f64,
    pub height: f64,
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

/// Cache Settings for Configured RMenu Plugins
#[derive(Debug, Clone, PartialEq)]
pub enum CacheSetting {
    NoCache,
    Never,
    OnLogin,
    AfterSeconds(usize),
}

impl Default for CacheSetting {
    fn default() -> Self {
        Self::NoCache
    }
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

/// RMenu Data-Source Plugin Configuration
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PluginConfig {
    pub exec: Vec<String>,
    #[serde(default)]
    pub format: Format,
    #[serde(default)]
    pub cache: CacheSetting,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub options: Option<Options>,
}

/// Allowed Formats for Entry Ingestion
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Format {
    #[default]
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
    pub mode_next: Vec<Keybind>,
    pub mode_prev: Vec<Keybind>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        return Self {
            exec: vec![Keybind::new(Code::Enter)],
            exit: vec![Keybind::new(Code::Escape)],
            move_next: vec![Keybind::new(Code::ArrowDown)],
            move_prev: vec![Keybind::new(Code::ArrowUp)],
            open_menu: vec![Keybind::new(Code::ArrowRight)],
            close_menu: vec![Keybind::new(Code::ArrowLeft)],
            jump_next: vec![Keybind::new(Code::PageDown)],
            jump_prev: vec![Keybind::new(Code::PageUp)],
            mode_next: vec![Keybind::new(Code::Tab)],
            mode_prev: vec![Keybind {
                mods: Modifiers::SHIFT,
                key: Code::Tab,
            }],
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
de_fromstr!(Format);
de_fromstr!(CacheSetting);
de_fromstr!(Keybind);

macro_rules! cfg_replace {
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

macro_rules! cfg_keybind {
    ($key:expr, $repl:expr) => {
        if let Some(bind_strings) = $repl.as_ref() {
            let mut keybinds = vec![];
            for bind_str in bind_strings.iter() {
                let bind = Keybind::from_str(bind_str)?;
                keybinds.push(bind);
            }
            $key = keybinds;
        }
    };
}

pub(crate) use cfg_keybind;
pub(crate) use cfg_replace;
pub(crate) use de_fromstr;
