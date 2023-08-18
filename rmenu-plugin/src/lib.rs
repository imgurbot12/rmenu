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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub actions: Vec<Action>,
    pub comment: Option<String>,
    pub icon: Option<String>,
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

/// Retrieve EXE of Self
#[inline]
pub fn self_exe() -> String {
    std::env::current_exe()
        .expect("Cannot Find EXE of Self")
        .to_str()
        .unwrap()
        .to_string()
}
