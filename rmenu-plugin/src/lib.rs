use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Method {
    Terminal,
    Desktop,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub exec: String,
    pub comment: Option<String>,
}

impl Action {
    pub fn new(exec: &str) -> Self {
        Self {
            name: "main".to_string(),
            exec: exec.to_string(),
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
}
