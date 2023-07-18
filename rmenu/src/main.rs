use std::fs::{read_to_string, File};
use std::io::{prelude::*, BufReader, Error};

mod config;
mod gui;
mod search;
mod state;

use clap::*;
use rmenu_plugin::Entry;

/// Rofi Clone (Built with Rust)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[arg(short, long, default_value_t=String::from("-"))]
    input: String,
    #[arg(short, long)]
    json: bool,
    #[arg(short, long)]
    msgpack: bool,
    #[arg(short, long)]
    run: Vec<String>,
    #[arg(long)]
    css: Option<String>,
}

//TODO: improve search w/ options for regex/case-insensivity/modes?
//TODO: add secondary menu for sub-actions aside from the main action
//TODO: improve looks and css

//TODO: config
//  - default and cli accessable modules (instead of piped in)
//  - allow/disable icons (also available via CLI)
//  - custom keybindings (some available via CLI?)

/// Application State for GUI
#[derive(Debug, PartialEq)]
pub struct App {
    css: String,
    name: String,
    entries: Vec<Entry>,
    config: config::Config,
}

fn default(args: &Args) -> Result<App, Error> {
    // read entries from specified input
    let fpath = match args.input.as_str() {
        "-" => "/dev/stdin",
        _ => &args.input,
    };
    let file = File::open(fpath)?;
    let reader = BufReader::new(file);
    let mut entries = vec![];
    for line in reader.lines() {
        let entry = serde_json::from_str::<Entry>(&line?)?;
        entries.push(entry);
    }
    // generate app object based on configured args
    let css = args
        .css
        .clone()
        .unwrap_or("rmenu/public/default.css".to_owned());
    let args = App {
        name: "default".to_string(),
        css: read_to_string(css)?,
        entries,
        config: Default::default(),
    };
    Ok(args)
}

fn main() {
    let cli = Args::parse();
    let app = default(&cli).unwrap();
    println!("{:?}", app);
    gui::run(app);
}
