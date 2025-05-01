use std::path::PathBuf;

use clap::Parser;
use freedesktop_desktop_entry::{DesktopEntry, Iter};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use rmenu_plugin::{Action, Entry, Method};

mod icons;
mod image;

static XDG_HOME_ENV: &'static str = "XDG_DATA_HOME";
static XDG_DATA_ENV: &'static str = "XDG_DATA_DIRS";
static XDG_CONFIG_ENV: &'static str = "XDG_CONFIG_HOME";
static XDG_CURRENT_DESKTOP_ENV: &'static str = "XDG_CURRENT_DESKTOP";

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
    let mut data_dirs: Vec<PathBuf> = format!("{home}:{dirs}")
        .split(":")
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        .map(|p| p.join(dir.to_owned()))
        .filter(|p| p.exists())
        .collect();
    if dir == "icons" {
        let home = shellexpand::tilde("~").to_string();
        let path = PathBuf::from(home).join(".icons");
        if path.exists() {
            data_dirs.insert(0, path)
        };
    }
    data_dirs
}

/// Modify Exec Statements to Remove %u/%f/etc...
#[inline(always)]
fn fix_exec(exec: &str) -> String {
    EXEC_RGX.replace_all(exec, "").trim().to_string()
}

/// Parse XDG Desktop Entry into RMenu Entry
fn parse_desktop(path: PathBuf, locales: &[&str]) -> Option<Entry> {
    let entry = DesktopEntry::from_path(path, Some(locales)).ok()?;
    // hide `NoDisplay` entries
    if entry.no_display() {
        return None;
    }
    // hide entries restricted by `OnlyShowIn`
    if let Ok(de) = std::env::var(XDG_CURRENT_DESKTOP_ENV) {
        if entry
            .only_show_in()
            .is_some_and(|only| only.contains(&de.as_str()))
        {
            return None;
        };
    }
    // parse desktop entry into rmenu entry
    let name = entry.name(locales)?.to_string();
    let icon = entry.icon().map(|i| i.to_string());
    let comment = entry.comment(locales).map(|s| s.to_string());
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
            .unwrap_or_default()
            .into_iter()
            .filter(|a| a.len() > 0)
            .filter_map(|a| {
                let name = entry.action_name(a, locales)?;
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

#[derive(Debug, Parser)]
struct Cli {
    /// Allow non-unique desktop entries
    #[clap(short, long)]
    non_unique: bool,
    /// Specify preferred desktop entry locale
    #[clap(short, long, default_value = "en")]
    locale: String,
}

fn main() {
    let cli = Cli::parse();
    let locales = &[cli.locale.as_str()];
    let sizes = vec![64, 32, 96, 22, 128];

    // collect icons
    let cfg = config_dir();
    let spec = icons::IconSpec::standard(&cfg, sizes, locales);
    let icons = icons::collect_icons(spec, locales);

    // collect applications
    let app_paths = data_dirs("applications");
    let mut desktops: Vec<Entry> = Iter::new(app_paths.into_iter())
        .into_iter()
        .unique_by(|f| match cli.non_unique {
            true => f.to_str().map(|s| s.to_owned()),
            false => f
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
        })
        .filter_map(|f| parse_desktop(f, locales))
        .map(|mut e| {
            e.icon = e.icon.and_then(|s| assign_icon(s, &icons));
            e
        })
        .collect();

    // convert desktop icon svgs to pngs
    let images = image::make_temp();
    let mut svgs: Vec<(&mut Entry, String, PathBuf)> = desktops
        .iter_mut()
        .filter(|e| {
            e.icon
                .as_ref()
                .map(|i| i.ends_with(".svg"))
                .unwrap_or_default()
        })
        .filter_map(|e| {
            let icon = e.icon.clone().expect("icon missing");
            let path = image::svg_path(&images, &icon)?;
            match path.exists() {
                true => None,
                false => Some((e, icon, path)),
            }
        })
        .collect();

    if !svgs.is_empty() {
        let opt = image::svg_options();
        svgs.par_iter_mut().for_each(|(entry, svg, png)| {
            image::convert_svg(svg, png, &opt);
            entry.icon = png.to_str().map(|s| s.to_owned());
        });
    }

    // sort entries and display
    desktops.par_sort_by_cached_key(|e| e.name.to_owned());
    let _: Vec<()> = desktops
        .into_iter()
        .filter_map(|e| serde_json::to_string(&e).ok())
        .map(|s| println!("{}", s))
        .collect();
}
