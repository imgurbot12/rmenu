use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;

use abi_stable::std_types::{RNone, RString};
use rmenu_plugin::{Entry, Exec};
use walkdir::{DirEntry, WalkDir};

/* Functions */

// check if file is executable
fn is_exec(entry: &DirEntry) -> bool {
    if entry.file_type().is_dir() {
        return true;
    }
    let Ok(meta) = entry.metadata() else { return false };
    meta.permissions().mode() & 0o111 != 0
}

// find all executables within the given paths
#[inline]
pub fn find_executables(paths: &Vec<String>) -> Vec<Entry> {
    let mut execs: HashMap<String, Entry> = HashMap::new();
    for path in paths.iter() {
        let walker = WalkDir::new(path).into_iter();
        for entry in walker.filter_entry(is_exec) {
            let Ok(dir)    = entry else { continue };
            let Some(name) = dir.file_name().to_str() else { continue };
            let Some(path) = dir.path().to_str() else { continue; };
            // check if entry already exists but replace on longer path
            if let Some(entry) = execs.get(name) {
                if let Exec::Terminal(ref exec) = entry.exec {
                    if exec.len() >= path.len() {
                        continue;
                    }
                }
            }
            execs.insert(
                name.to_owned(),
                Entry {
                    name: RString::from(name),
                    exec: Exec::Terminal(RString::from(path)),
                    comment: RNone,
                    icon: RNone,
                },
            );
        }
    }
    execs.into_values().collect()
}

/* Module */

// pub struct RunModule {}
//
// impl Module for RunModule {
//     fn name(&self) -> &str {
//         "run"
//     }
//
//     fn mode(&self) -> Mode {
//         Mode::Run
//     }
//
//     fn cache_setting(&self, settings: &Settings) -> Cache {
//         match settings.run.cache.as_ref() {
//             Some(cache) => cache.clone(),
//             None => Cache::After(Duration::new(30, 0)),
//         }
//     }
//
//     fn load(&self, settings: &Settings) -> Vec<Entry> {
//         let cfg = &settings.run;
//         let paths = match cfg.paths.as_ref() {
//             Some(paths) => paths.clone(),
//             None => get_paths(),
//         };
//         find_executables(&paths)
//     }
// }
