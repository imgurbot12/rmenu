use std::env;
use std::os::unix::fs::PermissionsExt;

use rayon::prelude::*;
use rmenu_plugin::Entry;
use walkdir::{DirEntry, WalkDir};

static PATH: &'static str = "PATH";
static DEFAULT_PATH: &'static str = "/bin:/usr/bin:/usr/sbin";
static EXEC_FLAG: u32 = 0o111;

/// Retrieve Search Paths from OS-VAR or Default
fn bin_paths() -> Vec<String> {
    env::var(PATH)
        .unwrap_or_else(|_| DEFAULT_PATH.to_string())
        .split(":")
        .map(|s| s.to_string())
        .collect()
}

/// Ignore Entry if Hidden or Filename contains a `.`
fn should_ignore(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.contains("."))
        .unwrap_or(false)
}

/// Retrieve Binaries for the Specified Paths
fn find_binaries(path: String) -> Vec<Entry> {
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_ignore(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.metadata()
                .map(|m| m.permissions().mode() & EXEC_FLAG != 0)
                .unwrap_or(false)
        })
        .map(|e| {
            let path = e.path().to_string_lossy();
            Entry::new(&e.file_name().to_string_lossy(), &path, Some(&path))
        })
        .collect()
}

fn main() {
    // collect entries for sorting
    let mut entries: Vec<Entry> = bin_paths()
        .into_par_iter()
        .map(find_binaries)
        .flatten()
        .collect();
    // sort entries and render to json
    entries.par_sort_by_cached_key(|e| e.name.clone());
    let _: Vec<()> = entries
        .into_par_iter()
        .map(|e| serde_json::to_string(&e))
        .filter_map(|r| r.ok())
        .map(|s| println!("{}", s))
        .collect();
}
