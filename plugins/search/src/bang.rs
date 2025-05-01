//! Bang Data Implementations and Utilities

use std::collections::HashMap;

use serde::Deserialize;

const BANGS: &'static str = include_str!("../external/bangs.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Bang {
    #[serde(alias = "s")]
    pub name: String,
    #[serde(alias = "t")]
    pub bang: String,
    #[serde(alias = "u")]
    pub url: String,
}

impl Bang {
    pub fn bangs() -> HashMap<String, Self> {
        let bangs: Vec<Bang> = serde_json::from_str(BANGS).expect("failed to parse bangs");
        bangs.into_iter().map(|s| (s.bang.to_owned(), s)).collect()
    }
}
