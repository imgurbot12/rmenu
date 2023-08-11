use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Terminal(String),
    Run(String),
    Echo(String),
}

impl Method {
    pub fn new(exec: String, terminal: bool) -> Self {
        match terminal {
            true => Self::Terminal(exec),
            false => Self::Run(exec),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub exec: Method,
    pub comment: Option<String>,
}

impl Action {
    pub fn new(exec: &str) -> Self {
        Self {
            name: "main".to_string(),
            exec: Method::Run(exec.to_string()),
            comment: None,
        }
    }
    pub fn echo(echo: &str) -> Self {
        Self {
            name: "main".to_string(),
            exec: Method::Echo(echo.to_string()),
            comment: None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub actions: Vec<Action>,
    pub comment: Option<String>,
    pub icon: Option<String>,
}

impl Entry {
    pub fn new(name: &str, action: &str, comment: Option<&str>) -> Self {
        Self {
            name: name.to_owned(),
            actions: vec![Action::new(action)],
            comment: comment.map(|c| c.to_owned()),
            icon: Default::default(),
        }
    }

    pub fn echo(echo: &str, comment: Option<&str>) -> Self {
        Self {
            name: echo.to_owned(),
            actions: vec![Action::echo(echo)],
            comment: comment.map(|c| c.to_owned()),
            icon: Default::default(),
        }
    }
}
