use serde::{Deserialize, Serialize};
use std::ffi::OsString;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub css: Vec<OsString>,
    pub use_icons: bool,
    pub search_regex: bool,
    pub ignore_case: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: vec![],
            use_icons: true,
            search_regex: false,
            ignore_case: true,
        }
    }
}
