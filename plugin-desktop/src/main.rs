use std::collections::{HashMap, HashSet};
use std::fs::FileType;
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
fn config_dir() -> PathBuf {
    let path = std::env::var(XDG_CONFIG_ENV).unwrap_or_else(|_| XDG_CONFIG_DEFAULT.to_string());
    PathBuf::from(shellexpand::tilde(&path).to_string())
}

/// Determine XDG Icon Theme based on Preexisting Configuration Files
fn find_theme(cfgdir: &PathBuf) -> Vec<String> {
    let mut themes: Vec<String> = vec![
        ("kdeglobals", "Icons", "Theme"),
        ("gtk-4.0/settings.ini", "Settings", "gtk-icon-theme-name"),
        ("gtk-3.0/settings.ini", "Settings", "gtk-icon-theme-name"),
    ]
    .into_iter()
    .filter_map(|(path, sec, key)| {
        let path = cfgdir.join(path);
        let ini = Ini::load_from_file(path).ok()?;
        ini.get_from(Some(sec), key).map(|s| s.to_string())
    })
    .collect();
    let default = DEFAULT_THEME.to_string();
    if !themes.contains(&default) {
        themes.push(default);
    }
    themes
}

type IconGroup = HashMap<String, PathBuf>;
type Icons = HashMap<String, IconGroup>;

/// Precalculate prefferred sizes folders
fn calculate_sizes(range: (usize, usize, usize)) -> HashSet<String> {
    let (min, preffered, max) = range;
    let mut size = preffered.clone();
    let mut sizes = HashSet::new();
    while size < max {
        sizes.insert(format!("{size}x{size}"));
        sizes.insert(format!("{size}x{size}@2"));
        size *= 2;
    }
    // attempt to match sizes down to lowest minimum
    let mut size = preffered.clone();
    while size > min {
        sizes.insert(format!("{size}x{size}"));
        sizes.insert(format!("{size}x{size}@2"));
        size /= 2;
    }
    sizes
}

#[inline(always)]
fn is_valid_icon(name: &str) -> bool {
    name.ends_with(".png") || name.ends_with(".svg")
}

/// Parse Icon-Name from Filename
#[inline]
fn icon_name(name: &str) -> String {
    name.rsplit_once(".")
        .map(|(i, _)| i)
        .unwrap_or(&name)
        .to_owned()
}

/// Parse and Categorize Icons Within the Specified Path
fn find_icons(path: &PathBuf, sizes: (usize, usize, usize)) -> Vec<IconGroup> {
    let sizes = calculate_sizes(sizes);
    let mut extras = IconGroup::new();
    let icons: Icons = WalkDir::new(path)
        // collect list of directories of icon subdirs
        .max_depth(1)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_str()?;
            let path = e.path().to_owned();
            match e.file_type().is_dir() {
                true => Some((name.to_owned(), path)),
                false => {
                    if is_valid_icon(name) {
                        extras.insert(icon_name(name), path);
                    }
                    None
                }
            }
        })
        // iterate content within subdirs
        .map(|(name, path)| {
            let group = WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| {
                    let name = e.file_name().to_str()?;
                    if is_valid_icon(name) {
                        return Some((icon_name(name), e.path().to_owned()));
                    }
                    None
                })
                .collect();
            (name, group)
        })
        .collect();
    // organize icon groups according to prefference
    let mut priority = vec![];
    let mut others = vec![];
    icons
        .into_iter()
        .map(|(folder, group)| match sizes.contains(&folder) {
            true => priority.push(group),
            false => match folder.contains("x") {
                false => others.push(group),
                _ => {}
            },
        })
        .last();
    priority.append(&mut others);
    priority.push(extras);
    priority
}

/// Retrieve Extras in Base Icon Directories
fn find_icon_extras(path: &PathBuf) -> IconGroup {
    WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let name = e.file_name().to_str()?;
            if is_valid_icon(name) {
                return Some((icon_name(&name), e.path().to_owned()));
            }
            None
        })
        .collect()
}

/// Retrieve XDG-DATA Directories
fn data_dirs(dir: &str) -> Vec<PathBuf> {
    std::env::var(XDG_DATA_ENV)
        .unwrap_or_else(|_| XDG_DATA_DEFAULT.to_string())
        .split(":")
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        .map(|p| p.join(dir.to_owned()))
        .filter(|p| p.exists())
        .collect()
}

/// Parse XDG Desktop Entry into RMenu Entry
fn parse_desktop(path: &Path, locale: Option<&str>) -> Option<Entry> {
    let bytes = read_to_string(path).ok()?;
    let entry = DesktopEntry::decode(&path, &bytes).ok()?;
    let name = entry.name(locale)?.to_string();
    let icon = entry.icon().map(|s| s.to_string());
    let comment = entry.comment(locale).map(|s| s.to_string());
    let mut actions = match entry.exec() {
        Some(exec) => vec![Action {
            name: "main".to_string(),
            exec: exec.to_string(),
            comment: None,
        }],
        None => vec![],
    };
    actions.extend(
        entry
            .actions()
            .unwrap_or("")
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
            }),
    );
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

/// Find and Assign Icons from Icon-Cache when Possible
fn assign_icons(icons: &Vec<IconGroup>, mut e: Entry) -> Entry {
    if let Some(name) = e.icon.as_ref() {
        if !name.contains("/") {
            if let Some(path) = icons.iter().find_map(|i| i.get(name)) {
                if let Some(fpath) = path.to_str() {
                    e.icon = Some(fpath.to_owned());
                }
            }
        }
    }
    e
}

fn main() {
    let locale = Some("en");
    let sizes = (32, 64, 128);
    // build a collection of icons for configured themes
    let cfgdir = config_dir();
    let themes = find_theme(&cfgdir);
    let icon_paths = data_dirs("icons");
    let mut icons: Vec<IconGroup> = icon_paths
        // generate list of icon-paths that exist
        .iter()
        .map(|d| themes.iter().map(|t| d.join(t)))
        .flatten()
        .filter(|t| t.exists())
        // append icon-paths within supported themes
        .map(|t| find_icons(&t, sizes))
        .flatten()
        .collect();
    // add extra icons found in base folders
    icons.extend(icon_paths.iter().map(|p| find_icon_extras(p)));
    // retrieve desktop applications and assign icons before printing results
    data_dirs("applications")
        .into_iter()
        .map(|p| find_desktops(p, locale))
        .flatten()
        .map(|e| assign_icons(&icons, e))
        .filter_map(|e| serde_json::to_string(&e).ok())
        .map(|s| println!("{}", s))
        .last();
}
