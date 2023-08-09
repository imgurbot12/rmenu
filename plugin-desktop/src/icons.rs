use std::collections::{BTreeMap, HashMap};
use std::fs::{read_dir, read_to_string};
use std::path::PathBuf;

use freedesktop_desktop_entry::DesktopEntry;
use ini::Ini;
use once_cell::sync::Lazy;
use thiserror::Error;
use walkdir::WalkDir;

type ThemeSource<'a> = (&'a str, &'a str, &'a str);

static INDEX_MAIN: &'static str = "Icon Theme";
static INDEX_NAME: &'static str = "Name";
static INDEX_SIZE: &'static str = "Size";
static INDEX_DIRS: &'static str = "Directories";
static INDEX_FILE: &'static str = "index.theme";

static DEFAULT_INDEX: &'static str = "default/index.theme";
static DEFAULT_THEME: &'static str = "Hicolor";

static PIXMAPS: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("/usr/share/pixmaps/"));
static THEME_SOURCES: Lazy<Vec<ThemeSource>> = Lazy::new(|| {
    vec![
        ("kdeglobals", "Icons", "Theme"),
        ("gtk-4.0/settings.ini", "Settings", "gtk-icon-theme-name"),
        ("gtk-3.0/settings.ini", "Settings", "gtk-icon-theme-name"),
    ]
});

/// Title String
#[inline]
fn title(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Collect Theme Definitions in Common GUI Configurations
fn theme_inis(cfgdir: &PathBuf) -> Vec<String> {
    THEME_SOURCES
        .iter()
        .filter_map(|(path, sec, key)| {
            let path = cfgdir.join(path);
            let ini = Ini::load_from_file(path).ok()?;
            ini.get_from(Some(sec.to_owned()), key).map(|s| title(s))
        })
        .collect()
}

/// Parse FreeDesktop Theme-Name from Index File
fn get_theme_name(path: &PathBuf) -> Option<String> {
    let content = read_to_string(path).ok()?;
    let config = DesktopEntry::decode(&path, &content).ok()?;
    config
        .groups
        .get(INDEX_MAIN)
        .and_then(|g| g.get(INDEX_NAME))
        .map(|key| key.0.to_owned())
}

/// Determine XDG Icon Theme based on Preexisting Configuration Files
pub fn active_themes(cfgdir: &PathBuf, icondirs: &Vec<PathBuf>) -> Vec<String> {
    let mut themes: Vec<String> = icondirs
        .iter()
        .map(|d| d.join(DEFAULT_INDEX))
        .filter(|p| p.exists())
        .filter_map(|p| get_theme_name(&p))
        .collect();
    themes.extend(theme_inis(cfgdir));
    let default = DEFAULT_THEME.to_string();
    if !themes.contains(&default) {
        themes.push(default);
    }
    themes
}

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("Failed to Read Index")]
    FileError(#[from] std::io::Error),
    #[error("Failed to Parse Index")]
    IndexError(#[from] freedesktop_desktop_entry::DecodeError),
    #[error("No Such Group")]
    NoSuchGroup(&'static str),
    #[error("No Such Key")]
    NoSuchKey(&'static str),
    #[error("Unselected Theme")]
    UnselectedTheme,
    #[error("Invalid Path Name")]
    BadPathName(PathBuf),
}

/// Track Paths and their Priority according to Sizes preference
struct PathPriority {
    path: PathBuf,
    priority: usize,
}

impl PathPriority {
    fn new(path: PathBuf, priority: usize) -> Self {
        Self { path, priority }
    }
}

/// Track Theme Information w/ Name/Priority/SubPaths
struct ThemeInfo {
    name: String,
    priority: usize,
    paths: Vec<PathBuf>,
}

/// Single Theme Specification
struct ThemeSpec<'a> {
    root: &'a PathBuf,
    themes: &'a Vec<String>,
    sizes: &'a Vec<String>,
}

impl<'a> ThemeSpec<'a> {
    fn new(root: &'a PathBuf, themes: &'a Vec<String>, sizes: &'a Vec<String>) -> Self {
        Self {
            root,
            themes,
            sizes,
        }
    }
}

/// Sort Theme Directories by Priority, Append Root, and Collect Names Only
#[inline]
fn sort_dirs(dirs: &mut Vec<PathPriority>) -> Vec<PathBuf> {
    dirs.sort_by_key(|p| p.priority);
    dirs.push(PathPriority::new("".into(), 0));
    dirs.into_iter().map(|p| p.path.to_owned()).collect()
}

/// Parse Theme Index and Sort Directories based on Size Preference
fn parse_index(spec: &ThemeSpec) -> Result<ThemeInfo, ThemeError> {
    // parse file content
    let index = spec.root.join(INDEX_FILE);
    let content = read_to_string(&index)?;
    let config = DesktopEntry::decode(&index, &content)?;
    let main = config
        .groups
        .get(INDEX_MAIN)
        .ok_or_else(|| ThemeError::NoSuchGroup(INDEX_MAIN))?;
    // retrieve name and directories
    let name = main
        .get(INDEX_NAME)
        .ok_or_else(|| ThemeError::NoSuchKey(INDEX_NAME))?
        .0;
    // check if name in supported themes
    let index = spec
        .themes
        .iter()
        .position(|t| t == &name)
        .ok_or_else(|| ThemeError::UnselectedTheme)?;
    // sort directories based on size preference
    let mut directories = main
        .get(INDEX_DIRS)
        .ok_or_else(|| ThemeError::NoSuchKey(INDEX_DIRS))?
        .0
        .split(',')
        .into_iter()
        .filter_map(|dir| {
            let group = config.groups.get(dir)?;
            let size = group
                .get(INDEX_SIZE)
                .and_then(|e| Some(e.0.to_owned()))
                .and_then(|s| spec.sizes.iter().position(|is| &s == is));
            Some(match size {
                Some(num) => PathPriority::new(spec.root.join(dir), num),
                None => PathPriority::new(spec.root.join(dir), 99),
            })
        })
        .collect();
    Ok(ThemeInfo {
        priority: index,
        name: name.to_owned(),
        paths: sort_dirs(&mut directories),
    })
}

/// Guess Theme when Index is Missing
fn guess_index(spec: &ThemeSpec) -> Result<ThemeInfo, ThemeError> {
    // parse name and confirm active theme
    let name = title(
        spec.root
            .file_name()
            .ok_or_else(|| ThemeError::BadPathName(spec.root.to_owned()))?
            .to_str()
            .ok_or_else(|| ThemeError::BadPathName(spec.root.to_owned()))?,
    );
    let index = spec
        .themes
        .iter()
        .position(|t| t == &name)
        .ok_or_else(|| ThemeError::UnselectedTheme)?;
    // retrieve directories and include priority
    let mut directories: Vec<PathPriority> = read_dir(spec.root)?
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_str().map(|n| n.to_owned())?;
            Some(match name.split_once("x") {
                Some((size, _)) => {
                    let index = spec.sizes.iter().position(|is| &size == is);
                    PathPriority::new(e.path(), index.unwrap_or(99))
                }
                None => PathPriority::new(e.path(), 99),
            })
        })
        .collect();
    // sort by priorty and only include matches
    Ok(ThemeInfo {
        name,
        priority: index,
        paths: sort_dirs(&mut directories),
    })
}

/// Specification for a Single Theme Path
pub struct IconSpec {
    paths: Vec<PathBuf>,
    themes: Vec<String>,
    sizes: Vec<String>,
}

impl IconSpec {
    pub fn new(paths: Vec<PathBuf>, themes: Vec<String>, sizes: Vec<usize>) -> Self {
        Self {
            paths,
            themes,
            sizes: sizes.into_iter().map(|i| i.to_string()).collect(),
        }
    }

    pub fn standard(cfg: &PathBuf, sizes: Vec<usize>) -> Self {
        let icon_paths = crate::data_dirs("icons");
        let themes = active_themes(cfg, &icon_paths);
        Self::new(icon_paths, themes, sizes)
    }
}

/// Parse and Collect a list of Directories to Find Icons in Order of Preference
fn parse_themes(icons: IconSpec) -> Vec<PathBuf> {
    // retrieve supported theme information
    let mut infos: Vec<ThemeInfo> = icons
        .paths
        // retrieve icon directories within main icon data paths
        .into_iter()
        .filter_map(|p| Some(read_dir(&p).ok()?.into_iter().filter_map(|d| d.ok())))
        .flatten()
        .map(|readdir| readdir.path())
        // parse or guess index themes
        .filter_map(|icondir| {
            let spec = ThemeSpec::new(&icondir, &icons.themes, &icons.sizes);
            parse_index(&spec)
                .map(|r| Ok(r))
                .unwrap_or_else(|_| guess_index(&spec))
                .ok()
        })
        .collect();
    // sort results by theme index
    infos.sort_by_key(|i| i.priority);
    // combine results from multiple directories for the same theme
    let mut map = BTreeMap::new();
    for info in infos.into_iter() {
        map.entry(info.name).or_insert(vec![]).extend(info.paths);
    }
    // finalize results from values
    map.insert("pixmaps".to_owned(), vec![PIXMAPS.to_owned()]);
    map.into_values().flatten().collect()
}

pub type IconMap = HashMap<String, PathBuf>;

#[inline]
fn is_icon(fname: &str) -> bool {
    fname.ends_with("png") || fname.ends_with("svg") || fname.ends_with("xpm")
}

/// Collect Unique Icon Map based on Preffered Paths
pub fn collect_icons(spec: IconSpec) -> IconMap {
    let mut map = HashMap::new();
    for path in parse_themes(spec).into_iter() {
        let icons = WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());
        for icon in icons {
            let Some(fname) = icon.file_name().to_str() else { continue };
            if !is_icon(&fname) {
                continue;
            }
            let Some((name, _)) = fname.rsplit_once(".") else { continue };
            map.entry(name.to_owned())
                .or_insert_with(|| icon.path().to_owned());
        }
    }
    map
}
