//! Windows Desktop Application Discovery

use std::path::PathBuf;

use image::{imageops::FilterType, DynamicImage, ImageFormat};
use itertools::Itertools;
use lnk::encoding::WINDOWS_1252;
use rayon::prelude::*;
use rmenu_plugin::{Action, Entry};

mod icons;
use crate::Cli;

const START_PROGRAMS: &'static str = "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs";
const USER_START_PROGRAMS: &'static str =
    "AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs";

//TODO: check lnk encoding to know which to use when parsing lnks
// Computer\HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Nls\CodePage\ACP

/// Check if windows path ends with the specified extension
#[inline]
fn ends_with(path: &PathBuf, ends: &str) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };
    let ext = ext.to_string_lossy().to_string();
    &ext == ends
}

/// Check if Name/Comment contains word "Uninstall"
#[inline]
fn name_contains(entry: &Entry, contains: &str) -> bool {
    if entry.name.to_lowercase().contains(contains) {
        return true;
    }
    if let Some(comment) = entry.comment.as_ref() {
        if comment.to_lowercase().contains(contains) {
            return true;
        }
    }
    false
}

/// Check if windows filepaths are on the same drive
#[inline]
fn match_drive(cache: &PathBuf, path: &str) -> bool {
    let cache = cache.to_string_lossy().to_string();
    let (d1, _) = cache.split_once(":").unwrap_or(("", ""));
    let (d2, _) = path.split_once(":").unwrap_or((d1, ""));
    d1 == d2
}

/// Extract images from exe and save to the rmenu cache directory
fn extract_ico(name: &str, icon_dir: &PathBuf, exe: &str) -> Option<PathBuf> {
    let cache = icon_dir.join(format!("{name}.png"));
    if cache.exists() {
        return Some(cache);
    }

    // extract icons and get first instance
    log::info!("extracting images from {exe:?}");
    let mut icons = match icons::get_images_from_exe(exe) {
        Ok(icons) => icons,
        Err(err) => {
            log::warn!("failed to extract images from exe {exe:?}: {err:?}");
            return None;
        }
    };
    if icons.is_empty() {
        log::warn!("no images in exe {exe:?}");
        return None;
    }
    let mut best = icons.swap_remove(0);
    let (width, height) = best.dimensions();

    // check if all pixels are completely transparent
    let is_hidden = best
        .as_flat_samples()
        .samples
        .par_chunks(4)
        .all(|c| c[3] == 0);
    if is_hidden {
        best.as_flat_samples_mut()
            .samples
            .par_chunks_mut(4)
            .for_each(|c| c[3] = 255);
    }

    // convert to dynamic image and resize before saving to cache
    let mut best = DynamicImage::ImageRgba8(best);
    if width < 64 || height < 64 {
        best = best.resize(64, 64, FilterType::Lanczos3);
    }
    if let Err(err) = best.save_with_format(&cache, ImageFormat::Png) {
        log::warn!("failed to save extracted image: {err:?}");
    };
    Some(cache)
}

/// Expand windows env paths
fn expand_env_vars(s: &str) -> String {
    regex::Regex::new(r"%([[:word:]]*)%")
        .expect("invalid regex exp")
        .replace_all(s, |captures: &regex::Captures| {
            match captures[1].to_string().as_str() {
                "" => String::from("%"),
                varname => std::env::var(varname).expect("bad env var"),
            }
        })
        .to_string()
}

/// Build and Prepare Desktop Icon for RMenu Entry
fn build_icon(name: &str, icon: &str, icon_dir: &PathBuf) -> Option<PathBuf> {
    let path = PathBuf::from(&icon);
    if icon.ends_with("chm") || icon.ends_with("url") {
        return None;
    }
    // extract icons from portal executables
    if icon.ends_with("exe") || icon.ends_with("dll") {
        let path = expand_env_vars(&icon);
        return extract_ico(&name, &icon_dir, &path);
    }
    // copy icons from different drive to cache folder
    if path.exists() && !match_drive(&icon_dir, &icon) {
        let ext = path
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "png".to_owned());
        let cache = icon_dir.join(&format!("{name}.{ext}"));
        if !cache.exists() {
            std::fs::copy(&path, &cache).expect("failed to copy icon");
        }
        return Some(cache);
    }
    Some(path)
}

/// Convert lnk file into rmenu entry
fn get_entry(entry: &PathBuf, icon_dir: &PathBuf) -> Option<Entry> {
    let fname = entry.file_stem().expect("invalid link filepath");
    let name = fname.to_string_lossy().to_string();

    let lnk = match lnk::ShellLink::open(&entry, WINDOWS_1252) {
        Ok(link) => link,
        Err(err) => {
            log::error!("failed to read {entry:?}: {err:?}");
            return None;
        }
    };

    // collect comment from lnk (filters out weird windows builtin comments)
    let sdata = lnk.string_data();
    let comment = sdata.name_string().clone().filter(|c| !c.starts_with("@"));

    // collect icons from exe files when required
    let path = lnk
        .link_info()
        .as_ref()
        .and_then(|i| i.local_base_path())
        .map(|s| s.to_string());
    let icon = sdata
        .icon_location()
        .clone()
        .or(path)
        .and_then(|i| build_icon(&name, &i, icon_dir));

    let icon = icon.map(|s| s.to_string_lossy().to_string());
    let action = Action::exec(&format!("cmd /c {entry:?}"));
    Some(Entry {
        name,
        icon,
        actions: vec![action],
        comment,
        icon_alt: None,
    })
}

/// Generate rmenu entries for all desktop shortcut links
pub fn get_entries(cli: &Cli) -> Vec<Entry> {
    // make temp-dir when extracting images from exe's
    let icon_dir = crate::image::make_temp();

    // collect paths to iterate
    let mut paths = vec![START_PROGRAMS.to_owned()];
    if let Some(path) = dirs::home_dir().map(|h| h.join(USER_START_PROGRAMS)) {
        let s = path.to_string_lossy();
        paths.insert(0, s.to_string());
    };

    // collect links from filesystem
    let links: Vec<PathBuf> = paths
        .into_iter()
        .map(|p| walkdir::WalkDir::new(p))
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| ends_with(&p, "lnk"))
        .unique_by(|f| match cli.non_unique {
            true => f.to_str().map(|s| s.to_owned()),
            false => f
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
        })
        .collect();

    // parse links into entries in parrallel
    links
        .into_par_iter()
        .filter_map(|p| get_entry(&p, &icon_dir))
        .filter(|e| cli.show_uninstall || !name_contains(&e, "uninstall"))
        .filter(|e| cli.show_docs || !name_contains(&e, "documentation"))
        .collect()
}
