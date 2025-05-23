///! RMenu Plugin Result Cache
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use rmenu_plugin::Entry;
use thiserror::Error;

use crate::config::{CacheSetting, PluginConfig};
use crate::XDG_PREFIX;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache Not Available")]
    NotAvailable,
    #[error("Cache Invalid")]
    InvalidCache,
    #[error("Cache Expired")]
    CacheExpired,
    #[error("Cache File Error")]
    FileError(#[from] std::io::Error),
    #[error("Encoding Error")]
    EncodingError(#[from] serde_json::Error),
}

#[cfg(target_os = "windows")]
#[inline]
fn cache_file(name: &str) -> PathBuf {
    let mut cache = dirs::cache_dir().expect("failed to find windows cache dir");
    cache.push(&format!("{}", XDG_PREFIX));
    if !cache.exists() {
        std::fs::create_dir_all(&cache).expect("failed to write windows cache dirs");
    }
    cache.push(name);
    cache
}

#[cfg(not(target_os = "windows"))]
#[inline]
fn cache_file(name: &str) -> PathBuf {
    xdg::BaseDirectories::with_prefix(XDG_PREFIX)
        .expect("Failed to read xdg base dirs")
        .place_cache_file(format!("{name}.cache"))
        .expect("Failed to write xdg cache dirs")
}

/// Read Entries from Cache (if Valid and Available)
pub fn read_cache(name: &str, cfg: &PluginConfig) -> Result<Vec<Entry>, CacheError> {
    // confirm cache exists
    let path = cache_file(name);
    if !path.exists() {
        return Err(CacheError::NotAvailable);
    }
    // get file modified date
    let meta = path.metadata()?;
    let modified = meta.modified()?;
    // confirm cache is not expired
    match cfg.cache {
        CacheSetting::NoCache => return Err(CacheError::InvalidCache),
        CacheSetting::Never => {}
        CacheSetting::OnLogin => {
            if let Ok(record) = lastlog::search_self() {
                if let Some(last) = record.last_login.into() {
                    if modified <= last {
                        return Err(CacheError::CacheExpired);
                    }
                }
            }
        }
        CacheSetting::AfterSeconds(secs) => {
            let now = SystemTime::now();
            let duration = Duration::from_secs(secs as u64);
            let diff = now
                .duration_since(modified)
                .unwrap_or_else(|_| Duration::from_secs(0));
            if diff >= duration {
                return Err(CacheError::CacheExpired);
            }
        }
    }
    // attempt to read content
    let data = fs::read(path)?;
    let results: Vec<Entry> = serde_json::from_slice(&data)?;
    Ok(results)
}

/// Write Results to Cache (if Allowed)
pub fn write_cache(name: &str, cfg: &PluginConfig, entries: &Vec<Entry>) -> Result<(), CacheError> {
    // write cache if allowed
    match cfg.cache {
        CacheSetting::NoCache => {}
        _ => {
            log::debug!("{name:?} writing {} entries", entries.len());
            let path = cache_file(name);
            let f = fs::File::create(path)?;
            serde_json::to_writer(f, entries)?;
        }
    }
    Ok(())
}
