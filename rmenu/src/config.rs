use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub css: Vec<String>,
    pub use_icons: bool,
    pub search_regex: bool,
    pub ignore_case: bool,
    pub plugins: BTreeMap<String, VecDeque<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: vec![],
            use_icons: true,
            search_regex: false,
            ignore_case: true,
            plugins: Default::default(),
        }
    }
}
