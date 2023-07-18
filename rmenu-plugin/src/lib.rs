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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub actions: Vec<Action>,
    pub comment: Option<String>,
    pub icon: Option<String>,
}

impl Entry {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            actions: Default::default(),
            comment: Default::default(),
            icon: Default::default(),
        }
    }
}
