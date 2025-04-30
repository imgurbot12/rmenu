//! RMenu-Plugin Object Implementations
use serde::{Deserialize, Serialize};

/// Methods allowed to Execute Actions on Selection
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Terminal(String),
    Run(String),
    Echo(String),
}

impl Method {
    /// Generate the Required Method from a Function
    pub fn new(exec: String, terminal: bool) -> Self {
        match terminal {
            true => Self::Terminal(exec),
            false => Self::Run(exec),
        }
    }
}

/// RMenu Entry Action Definition
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub exec: Method,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl Action {
    /// Generate a simple Execution Action
    pub fn exec(exec: &str) -> Self {
        Self {
            name: "main".to_string(),
            exec: Method::Run(exec.to_string()),
            comment: None,
        }
    }
    /// Generate a simple Echo Action
    pub fn echo(echo: &str) -> Self {
        Self {
            name: "main".to_string(),
            exec: Method::Echo(echo.to_string()),
            comment: None,
        }
    }
}

/// RMenu Menu-Entry Implementation
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename = "entry")]
pub struct Entry {
    pub name: String,
    pub actions: Vec<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_alt: Option<String>,
}

impl Entry {
    /// Generate a simplified Exec Action Entry
    pub fn new(name: &str, action: &str, comment: Option<&str>) -> Self {
        Self {
            name: name.to_owned(),
            actions: vec![Action::exec(action)],
            comment: comment.map(|c| c.to_owned()),
            icon: Default::default(),
            icon_alt: Default::default(),
        }
    }
    /// Generate a simplified Echo Action Entry
    pub fn echo(echo: &str, comment: Option<&str>) -> Self {
        Self {
            name: echo.to_owned(),
            actions: vec![Action::echo(echo)],
            comment: comment.map(|c| c.to_owned()),
            icon: Default::default(),
            icon_alt: Default::default(),
        }
    }
}

/// Additional Plugin Option Overrides
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default, tag = "type", rename = "options")]
pub struct Options {
    // base settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_load: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jump_dist: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_select: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_click: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_menu: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_icons: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_comments: Option<bool>,

    // search settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_restrict: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_max_length: Option<usize>,
    // key settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_exec: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_exit: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_move_next: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_move_prev: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_open_menu: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_close_menu: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_jump_next: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_jump_prev: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_mode_next: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_mode_prev: Option<Vec<String>>,
    // window settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decorate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transparent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fullscreen: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_height: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Search {
    pub search: String,
    #[serde(default)]
    pub is_regex: bool,
    #[serde(default)]
    pub ignore_case: bool,
}

/// Valid RMenu Plugin Messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    Entry(Entry),
    Options(Options),
    Stop,
}

/// Retrieve EXE of Self
#[inline]
pub fn self_exe() -> String {
    std::env::current_exe()
        .expect("Cannot Find EXE of Self")
        .to_str()
        .unwrap()
        .to_string()
}
