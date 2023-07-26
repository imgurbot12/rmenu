use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs::read_to_string, path::Path};

use freedesktop_desktop_entry::DesktopEntry;
use ini::Ini;
use rmenu_plugin::{Action, Entry};
use walkdir::WalkDir;

static XDG_DATA_ENV: &'static str = "XDG_DATA_DIRS";
static XDG_CONFIG_ENV: &'static str = "XDG_CONFIG_HOME";
static XDG_DATA_DEFAULT: &'static str = "/usr/share:/usr/local/share";
static XDG_CONFIG_DEFAULT: &'static str = "~/.config";
static DEFAULT_THEME: &'static str = "hicolor";

/// Retrieve XDG-CONFIG-HOME Directory
#[inline]
fn config_dir(dir: &str) -> PathBuf {
    let path = std::env::var(XDG_CONFIG_ENV).unwrap_or_else(|_| XDG_CONFIG_DEFAULT.to_string());
    PathBuf::from(shellexpand::tilde(&path).to_string())
}

/// Determine XDG Icon Theme based on Preexisting Configuration Files
fn find_theme(cfgdir: &PathBuf) -> String {
    vec![
        ("kdeglobals", "Icons", "Theme"),
        ("gtk-3.0/settings.ini", "Settings", "gtk-icon-theme-name"),
        ("gtk-4.0/settings.ini", "Settings", "gtk-icon-theme-name"),
    ]
    .into_iter()
    .find_map(|(path, sec, key)| {
        let path = cfgdir.join(path);
        let ini = Ini::load_from_file(path).ok()?;
        ini.get_from(Some(sec), key).map(|s| s.to_string())
    })
    .unwrap_or_else(|| DEFAULT_THEME.to_string())
}

type IconGroup = HashMap<String, PathBuf>;
type Icons = HashMap<String, IconGroup>;

/// Parse and Categorize Icons Within the Specified Path
fn find_icons(path: &PathBuf) -> Icons {
    WalkDir::new(path)
        // collect list of directories of icon subdirs
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            Some((name, e.path().to_owned()))
        })
        // iterate content within subdirs
        .map(|(name, path)| {
            let group = WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| {
                    let name = e.file_name().to_str()?.to_string();
                    if name.ends_with(".png") || name.ends_with(".svg") {
                        let icon = name.rsplit_once(".").map(|(i, _)| i).unwrap_or(&name);
                        return Some((icon.to_owned(), e.path().to_owned()));
                    }
                    None
                })
                .collect();
            (name, group)
        })
        .collect()
}

/// Find Best Icon Match for the Given Name
fn match_icon<'a>(icons: &'a Icons, name: &str, size: usize) -> Option<&'a PathBuf> {
    todo!("implement icon matching to specified name")
}

/// Retrieve XDG-DATA Directories
fn data_dirs(dir: &str) -> Vec<PathBuf> {
    std::env::var(XDG_DATA_ENV)
        .unwrap_or_else(|_| XDG_DATA_DEFAULT.to_string())
        .split(":")
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        .map(|p| p.join(dir.to_owned()))
        .collect()
}

/// Parse XDG Desktop Entry into RMenu Entry
fn parse_desktop(path: &Path, locale: Option<&str>) -> Option<Entry> {
    let bytes = read_to_string(path).ok()?;
    let entry = DesktopEntry::decode(&path, &bytes).ok()?;
    let name = entry.name(locale)?.to_string();
    let icon = entry.icon().map(|s| s.to_string());
    let comment = entry.comment(locale).map(|s| s.to_string());
    let actions: Vec<Action> = entry
        .actions()?
        .split(";")
        .into_iter()
        .filter(|a| a.len() > 0)
        .filter_map(|a| {
            let name = entry.action_name(a, locale)?;
            let exec = entry.action_exec(a)?;
            Some(Action {
                name: name.to_string(),
                exec: exec.to_string(),
                comment: None,
            })
        })
        .collect();
    Some(Entry {
        name,
        actions,
        comment,
        icon,
    })
}

/// Iterate Path and Parse All `.desktop` files into Entries
fn find_desktops(path: PathBuf, locale: Option<&str>) -> Vec<Entry> {
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".desktop"))
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| parse_desktop(e.path(), locale))
        .collect()
}

fn main() {
    let path = PathBuf::from("/usr/share/icons/hicolor");
    let icons = find_icons(&path);
    icons
        .into_iter()
        .map(|(k, v)| {
            println!("category: {k:?}");
            v.into_iter()
                .map(|(name, path)| {
                    println!(" - {name:?}");
                })
                .last()
        })
        .last();

    // data_dirs("applications")
    //     .into_iter()
    //     .map(|p| find_desktops(p, locale))
    //     .flatten()
    //     .filter_map(|e| serde_json::to_string(&e).ok())
    //     .map(|s| println!("{}", s))
    //     .last();
}
