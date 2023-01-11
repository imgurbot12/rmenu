use std::collections::HashMap;
use std::fs;

use abi_stable::std_types::{RNone, ROption, RSome, RString, RVec};
use freedesktop_entry_parser::parse_entry;
use walkdir::{DirEntry, WalkDir};

use rmenu_plugin::*;

/* Types */

#[derive(Debug)]
struct IconFile {
    name: String,
    path: String,
    size: u64,
}

/* Functions */

// filter out invalid icon entries
fn is_icon(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
        || entry
            .file_name()
            .to_str()
            .map(|s| s.ends_with(".svg") || s.ends_with(".png"))
            .unwrap_or(false)
}

// filter out invalid desktop entries
fn is_desktop(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
        || entry
            .file_name()
            .to_str()
            .map(|s| s.ends_with(".desktop"))
            .unwrap_or(false)
}

// correlate name w/ best matched icon
#[inline]
fn match_icon(name: &str, icons: &Vec<IconFile>) -> Option<String> {
    for icon in icons.iter() {
        if icon.name == name {
            return Some(icon.path.to_owned());
        }
        let Some((fname, _)) = icon.name.rsplit_once('.') else { continue };
        if fname == name {
            return Some(icon.path.to_owned());
        }
    }
    None
}

// correlate name w/ best matched icon and read into valid entry
#[inline]
fn read_icon(name: &str, icons: &Vec<IconFile>) -> Option<Icon> {
    let path = match_icon(name, icons)?;
    let Ok(data) = fs::read(&path) else { return None };
    Some(Icon {
        name: RString::from(name),
        path: RString::from(path),
        data: RVec::from(data),
    })
}

// retrieve master-list of all possible xdg-application entries from filesystem
pub fn load_entries(app_paths: &Vec<String>, icon_paths: &Vec<String>) -> Vec<Entry> {
    // iterate and collect all existing icon paths
    let mut imap: HashMap<String, IconFile> = HashMap::new();
    for path in icon_paths.into_iter() {
        let walker = WalkDir::new(path).into_iter();
        for entry in walker.filter_entry(is_icon) {
            let Ok(dir)    = entry else { continue };
            let Ok(meta)   = dir.metadata() else { continue };
            let Some(name) = dir.file_name().to_str() else { continue };
            let Some(path) = dir.path().to_str() else { continue; };
            let size = meta.len();
            // find the biggest icon file w/ the same name
            let pathstr = path.to_owned();
            if let Some(icon) = imap.get_mut(name) {
                if icon.size < size {
                    icon.path = pathstr;
                    icon.size = size;
                }
                continue;
            }
            imap.insert(
                name.to_owned(),
                IconFile {
                    name: name.to_owned(),
                    path: pathstr,
                    size,
                },
            );
        }
    }
    // parse application entries
    let icons = imap.into_values().collect();
    let mut entries = vec![];
    for path in app_paths.into_iter() {
        let walker = WalkDir::new(path).into_iter();
        for entry in walker.filter_entry(is_desktop) {
            let Ok(dir)    = entry else { continue };
            let Ok(file)   = parse_entry(dir.path()) else { continue };
            let desktop = file.section("Desktop Entry");
            let Some(name) = desktop.attr("Name") else { continue };
            let Some(exec) = desktop.attr("Exec") else { continue };
            let terminal = desktop.attr("Terminal").unwrap_or("") == "true";
            // parse icon
            let icon = match desktop.attr("Icon") {
                Some(name) => ROption::from(read_icon(name, &icons)),
                None => RNone,
            };
            // parse comment
            let comment = match desktop.attr("Comment") {
                Some(attr) => RSome(RString::from(attr)),
                None => RNone,
            };
            // convert exec/terminal into command
            let command = match terminal {
                true => Exec::Terminal(RString::from(exec)),
                false => Exec::Command(RString::from(exec)),
            };
            // generate entry
            entries.push(Entry {
                name: RString::from(name),
                exec: command,
                comment,
                icon,
            });
        }
    }
    entries
}
