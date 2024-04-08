use std::fs::read_to_string;
use std::path::PathBuf;

use freedesktop_desktop_entry::{DesktopEntry, Iter};
use once_cell::sync::Lazy;
use regex::Regex;
use rmenu_plugin::{Action, Entry, Method};

mod icons;

static XDG_HOME_ENV: &'static str = "XDG_DATA_HOME";
static XDG_DATA_ENV: &'static str = "XDG_DATA_DIRS";
static XDG_CONFIG_ENV: &'static str = "XDG_CONFIG_HOME";

static XDG_HOME_DEFAULT: &'static str = "~/.local/share";
static XDG_DATA_DEFAULT: &'static str = "/usr/share:/usr/local/share";
static XDG_CONFIG_DEFAULT: &'static str = "~/.config";

static EXEC_RGX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"%\w").expect("Failed to Build Exec Regex"));

/// Retrieve XDG-CONFIG-HOME Directory
#[inline]
fn config_dir() -> PathBuf {
    let path = std::env::var(XDG_CONFIG_ENV).unwrap_or_else(|_| XDG_CONFIG_DEFAULT.to_string());
    PathBuf::from(shellexpand::tilde(&path).to_string())
}

/// Retrieve XDG-DATA Directories
fn data_dirs(dir: &str) -> Vec<PathBuf> {
    let home = std::env::var(XDG_HOME_ENV).unwrap_or_else(|_| XDG_HOME_DEFAULT.to_string());
    let dirs = std::env::var(XDG_DATA_ENV).unwrap_or_else(|_| XDG_DATA_DEFAULT.to_string());
    format!("{home}:{dirs}")
        .split(":")
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        .map(|p| p.join(dir.to_owned()))
        .filter(|p| p.exists())
        .collect()
}

/// Modify Exec Statements to Remove %u/%f/etc...
#[inline(always)]
fn fix_exec(exec: &str) -> String {
    EXEC_RGX.replace_all(exec, "").trim().to_string()
}

/// Parse XDG Desktop Entry into RMenu Entry
fn parse_desktop(path: &PathBuf, locale: Option<&str>) -> Option<Entry> {
    let bytes = read_to_string(path).ok()?;
    let entry = DesktopEntry::decode(&path, &bytes).ok()?;
    let name = entry.name(locale)?.to_string();
    let icon = entry.icon().map(|i| i.to_string());
    let comment = entry.comment(locale).map(|s| s.to_string());
    let terminal = entry.terminal();
    let mut actions = match entry.exec() {
        Some(exec) => vec![Action {
            name: "main".to_string(),
            exec: Method::new(fix_exec(exec), terminal),
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
                    exec: Method::new(fix_exec(exec), terminal),
                    comment: None,
                })
            }),
    );
    Some(Entry {
        name,
        actions,
        comment,
        icon,
        icon_alt: None,
    })
}

/// Assign XDG Icon based on Desktop-Entry
fn assign_icon(icon: String, map: &icons::IconMap) -> Option<String> {
    if !icon.contains("/") {
        if let Some(icon) = map.get(&icon) {
            if let Some(path) = icon.to_str() {
                return Some(path.to_owned());
            }
        }
    }
    Some(icon)
}

fn main() {
    let locale = Some("en");
    let sizes = vec![64, 32, 96, 22, 128];

    // collect icons
    let cfg = config_dir();
    let spec = icons::IconSpec::standard(&cfg, sizes);
    let icons = icons::collect_icons(spec);

    // collect applications
    let app_paths = data_dirs("applications");
    let mut desktops: Vec<Entry> = Iter::new(app_paths)
        .into_iter()
        .filter_map(|f| parse_desktop(&f, locale))
        .map(|mut e| {
            e.icon = e.icon.and_then(|s| assign_icon(s, &icons));
            e
        })
        .collect();

    desktops.sort_by_cached_key(|e| e.name.to_owned());
    desktops
        .into_iter()
        .filter_map(|e| serde_json::to_string(&e).ok())
        .map(|s| println!("{}", s))
        .last();
}
