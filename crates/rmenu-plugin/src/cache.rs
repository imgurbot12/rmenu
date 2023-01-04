/*
 * Plugin Optional Cache Implementation for Convenient Storage
 */
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::{env, fs};

use abi_stable::{std_types::RString, StableAbi};
use lastlog::search_self;

use super::{Entries, ModuleConfig};

/* Variables */

static HOME: &str = "HOME";
static XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";

/* Functions */

/// Retrieve xdg-cache directory or get default
pub fn get_cache_dir() -> std::result::Result<String, String> {
    let path = if let Ok(xdg) = env::var(XDG_CACHE_HOME) {
        Path::new(&xdg).to_path_buf()
    } else if let Ok(home) = env::var(HOME) {
        Path::new(&home).join(".cache")
    } else {
        Path::new("/tmp/").to_path_buf()
    };
    match path.join("rmenu").to_str() {
        Some(s) => Ok(s.to_owned()),
        None => Err(format!("Failed to read $XDG_CACHE_DIR or $HOME")),
    }
}

/// Retrieve standard cache-path variable from config or set to default
#[inline]
pub fn get_cache_dir_setting(cfg: &ModuleConfig) -> PathBuf {
    Path::new(
        match cfg.get("cache_path") {
            Some(path) => RString::from(path.as_str()),
            None => RString::from(get_cache_dir().unwrap()),
        }
        .as_str(),
    )
    .to_path_buf()
}

/// Retrieve standard cache-mode variable from config or set to default
#[inline]
pub fn get_cache_setting(cfg: &ModuleConfig, default: CacheSetting) -> CacheSetting {
    cfg.get("cache_mode")
        .unwrap_or(&RString::from(""))
        .parse::<CacheSetting>()
        .unwrap_or(default)
}

/* Module */

/// Configured Module Cache settings
#[repr(C)]
#[derive(Debug, Clone, PartialEq, StableAbi)]
pub enum CacheSetting {
    Never,
    Forever,
    OnLogin,
    After(u64),
}

impl std::str::FromStr for CacheSetting {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.chars().all(char::is_numeric) {
            let i = s.parse::<u64>().map_err(|e| format!("{e}"))?;
            return Ok(CacheSetting::After(i));
        };
        match s {
            "never" => Ok(CacheSetting::Never),
            "forever" => Ok(CacheSetting::Forever),
            "login" => Ok(CacheSetting::OnLogin),
            _ => Err(format!("invalid value: {:?}", s)),
        }
    }
}

/// Simple Entry cache implementation
pub struct Cache {
    path: PathBuf,
    cache: HashMap<String, Entries>,
}

impl Cache {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            cache: HashMap::new(),
        }
    }

    // attempt to read given cache entry from disk
    fn load(&self, name: &str, valid: &CacheSetting) -> Result<Entries> {
        // skip all checks if caching is disabled
        let expr_err = Error::new(ErrorKind::InvalidData, "cache expired");
        if valid == &CacheSetting::Never {
            return Err(expr_err);
        }
        // check if the given path exists
        let path = self.path.join(name);
        if !path.is_file() {
            return Err(Error::new(ErrorKind::NotFound, "no such file"));
        }
        // get last-modified date of file
        let meta = path.metadata()?;
        let modified = meta.modified()?;
        // handle expiration based on cache-setting
        match valid {
            CacheSetting::Never => return Err(expr_err),
            CacheSetting::Forever => {}
            CacheSetting::OnLogin => {
                // expire content if it was last modified before last login
                if let Ok(record) = search_self() {
                    if let Some(lastlog) = record.last_login.into() {
                        if modified <= lastlog {
                            return Err(expr_err);
                        }
                    }
                }
            }
            CacheSetting::After(secs) => {
                // expire content if it was last modified longer than duration
                let now = SystemTime::now();
                let duration = Duration::from_secs(*secs);
                if now - duration >= modified {
                    return Err(expr_err);
                }
            }
        };
        // load entries from bincode
        let data = fs::read(path)?;
        let entries: Entries = bincode::deserialize(&data).map_err(|_| ErrorKind::InvalidData)?;
        Ok(entries)
    }

    /// Returns true if the given key was found in cache memory
    pub fn contains_key(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }

    /// Read all cached entries associated w/ the specified key
    pub fn read(&mut self, name: &str, valid: &CacheSetting) -> Result<&Entries> {
        // return cache entry if already cached
        if self.cache.contains_key(name) {
            return Ok(self.cache.get(name).expect("read failed cache grab"));
        }
        // load entries from cache on disk
        let entries = self.load(name, valid)?;
        self.cache.insert(name.to_owned(), entries);
        Ok(self.cache.get(name).expect("cached entries are missing?"))
    }

    /// Write all entries associated w/ the specified key to cache
    pub fn write(&mut self, name: &str, valid: &CacheSetting, entries: Entries) -> Result<()> {
        // write to runtime cache reguardless of cache settings
        self.cache.insert(name.to_owned(), entries);
        // skip caching if disabled
        if valid == &CacheSetting::Never {
            return Ok(());
        }
        // retrieve entries passed to cache
        let entries = self.cache.get(name).expect("write failed cache grab");
        // serialize entries and write to cache
        let path = self.path.join(name);
        let mut f = fs::File::create(path)?;
        let data = bincode::serialize(entries).map_err(|_| ErrorKind::InvalidInput)?;
        f.write_all(&data)?;
        Ok(())
    }

    /// easy functional wrapper that ensures data is always loaded once and then cached
    pub fn wrap<F>(&mut self, name: &str, valid: &CacheSetting, func: F) -> Result<Entries>
    where
        F: FnOnce() -> Entries,
    {
        let entries = match self.read(name, valid) {
            Ok(entries) => entries,
            Err(_) => {
                self.write(name, valid, func())?;
                self.read(name, valid)?
            }
        };
        Ok(entries.clone())
    }
}
