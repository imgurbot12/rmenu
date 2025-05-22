#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::path::PathBuf;

use clap::Parser;
use rayon::prelude::*;
use rmenu_plugin::Entry;

mod image;

#[cfg(target_family = "unix")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Parser)]
pub struct Cli {
    /// Allow non-unique desktop entries
    #[clap(long)]
    non_unique: bool,
    /// Show Uninstall App Shortcuts
    #[clap(long)]
    show_uninstall: bool,
    /// Show Documentation App Shortcuts
    #[clap(long)]
    show_docs: bool,
    /// Specify preferred desktop entry locale
    #[clap(short, long, default_value = "en")]
    locale: String,
}

#[cfg(target_os = "windows")]
fn get_entries(cli: &Cli) -> Vec<Entry> {
    windows::get_entries(cli)
}

#[cfg(target_family = "unix")]
fn get_entries(cli: &Cli) -> Vec<Entry> {
    linux::get_entries(cli)
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let mut desktops = get_entries(&cli);

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
