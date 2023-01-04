use std::env;
use std::io::Result;
use std::path::{Path, PathBuf};

use abi_stable::std_types::*;
use regex::{Regex, RegexBuilder};
use rmenu_plugin::{cache::*, *};

mod desktop;
use desktop::load_entries;

/* Variables */

static NAME: &str = "drun";
static PREFIX: &str = "app";

static XDG_DATA_DIRS: &str = "XDG_DATA_DIRS";

static DEFAULT_XDG_PATHS: &str = "/usr/share/";
static DEFAULT_APP_PATHS: &str = "";
static DEFAULT_ICON_PATHS: &str = "/usr/share/pixmaps/";

/* Functions */

// parse path string into separate path entries
#[inline]
fn parse_config_paths(paths: String) -> Vec<String> {
    env::split_paths(&paths)
        .map(|s| s.to_str().expect("invalid path").to_owned())
        .collect()
}

// retrieve default xdg-paths using xdg environment variable when possible
#[inline]
fn default_xdg_paths() -> String {
    if let Ok(paths) = env::var(XDG_DATA_DIRS) {
        return paths.to_owned();
    }
    DEFAULT_XDG_PATHS.to_owned()
}

// append joined xdg-paths to app/icon path results
#[inline]
fn apply_paths(join: &str, paths: &Vec<String>) -> Vec<String> {
    paths
        .iter()
        .map(|s| {
            Path::new(s)
                .join(join)
                .to_str()
                .expect("Unable to join PATH")
                .to_owned()
        })
        .collect()
}

// regex validate if the following entry matches the given regex expression
#[inline]
pub fn is_match(entry: &Entry, search: &Regex) -> bool {
    if search.is_match(&entry.name) {
        return true;
    };
    if let RSome(comment) = entry.comment.as_ref() {
        return search.is_match(&comment);
    }
    false
}

/* Macros */

macro_rules! pathify {
    ($cfg:expr, $key:expr, $other:expr) => {
        parse_config_paths(match $cfg.get($key) {
            Some(path) => path.as_str().to_owned(),
            None => $other,
        })
    };
}

/* Plugin */

struct Settings {
    xdg_paths: Vec<String>,
    app_paths: Vec<String>,
    icon_paths: Vec<String>,
    cache_path: PathBuf,
    cache_mode: CacheSetting,
    ignore_case: bool,
}

struct DesktopRun {
    cache: Cache,
    settings: Settings,
}

impl DesktopRun {
    pub fn new(cfg: &ModuleConfig) -> Self {
        let settings = Settings {
            xdg_paths: pathify!(cfg, "xdg_paths", default_xdg_paths()),
            app_paths: pathify!(cfg, "app_paths", DEFAULT_APP_PATHS.to_owned()),
            icon_paths: pathify!(cfg, "icon_paths", DEFAULT_ICON_PATHS.to_owned()),
            ignore_case: cfg
                .get("ignore_case")
                .unwrap_or(&RString::from("true"))
                .parse()
                .unwrap_or(true),
            cache_path: get_cache_dir_setting(cfg),
            cache_mode: get_cache_setting(cfg, CacheSetting::OnLogin),
        };
        Self {
            cache: Cache::new(settings.cache_path.clone()),
            settings,
        }
    }

    fn load(&mut self) -> Result<Entries> {
        self.cache.wrap(NAME, &self.settings.cache_mode, || {
            // configure paths w/ xdg-paths
            let mut app_paths = apply_paths("applications", &self.settings.xdg_paths);
            let mut icon_paths = apply_paths("icons", &self.settings.xdg_paths);
            app_paths.append(&mut self.settings.app_paths.clone());
            icon_paths.append(&mut self.settings.icon_paths.clone());
            // generate search results
            let mut entries = load_entries(&app_paths, &icon_paths);
            entries.sort_by_cached_key(|s| s.name.to_owned());
            RVec::from(entries)
        })
    }
}

impl Module for DesktopRun {
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
        // retrieve entries based on declared modes
        let Ok(entries) = self.load() else { return matches };
        // search existing entries for matching regex expr
        for entry in entries.into_iter() {
            if is_match(&entry, &rgx) {
                matches.push(entry);
            }
        }
        matches
    }
}

export_plugin!(DesktopRun);
