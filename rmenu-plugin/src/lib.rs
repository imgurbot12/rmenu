use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Method {
    Terminal,
    Desktop,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub exec: String,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub actions: BTreeMap<String, Action>,
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
