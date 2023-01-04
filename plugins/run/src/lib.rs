/*!
 *  Binary/Executable App Search Module
 */

use std::env;
use std::io::Result;
use std::path::PathBuf;

use abi_stable::std_types::{RString, RVec};
use regex::RegexBuilder;
use rmenu_plugin::{cache::*, *};

mod run;
use run::find_executables;

/* Variables */

static NAME: &str = "run";
static PREFIX: &str = "exec";

static PATH_VAR: &str = "PATH";

/* Functions */

// parse path string into separate path entries
#[inline]
fn parse_config_paths(paths: &str) -> Vec<String> {
    env::split_paths(paths)
        .map(|s| s.to_str().expect("invalid path").to_owned())
        .collect()
}

// get all paths listed in $PATH env variable
#[inline]
fn get_paths() -> Vec<String> {
    parse_config_paths(&env::var(PATH_VAR).expect("Unable to read $PATH"))
}

/* Module */

struct Settings {
    paths: Vec<String>,
    cache_path: PathBuf,
    cache_mode: CacheSetting,
    ignore_case: bool,
}

struct Run {
    cache: Cache,
    settings: Settings,
}

impl Run {
    pub fn new(cfg: &ModuleConfig) -> Self {
        let settings = Settings {
            ignore_case: cfg
                .get("ignore_case")
                .unwrap_or(&RString::from("true"))
                .parse()
                .unwrap_or(true),
            paths: match cfg.get("paths") {
                Some(paths) => parse_config_paths(paths.as_str()),
                None => get_paths(),
            },
            cache_path: get_cache_dir_setting(cfg),
            cache_mode: get_cache_setting(cfg, CacheSetting::After(30)),
        };
        Run {
            cache: Cache::new(settings.cache_path.clone()),
            settings,
        }
    }

    fn load(&mut self) -> Result<Entries> {
        self.cache.wrap(NAME, &self.settings.cache_mode, || {
            RVec::from(find_executables(&self.settings.paths))
        })
    }
}

impl Module for Run {
    extern "C" fn name(&self) -> RString {
        RString::from(NAME)
    }
    extern "C" fn prefix(&self) -> RString {
        RString::from(PREFIX)
    }
    extern "C" fn search(&mut self, search: RString) -> Entries {
        // compile regex expression for the given search
        let mut matches = RVec::new();
        let Ok(rgx) = RegexBuilder::new(search.as_str())
            .case_insensitive(self.settings.ignore_case)
            .build() else { return matches };
        // load entries and evaluate matches
        let entries = self.load().expect("failed to parse through $PATH");
        for entry in entries.into_iter() {
            if rgx.is_match(entry.name.as_str()) {
                matches.push(entry);
            }
        }
        matches
    }
}

export_plugin!(Run);
