use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub regex: bool,
    pub ignore_case: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            regex: true,
            ignore_case: true,
        }
    }
}
