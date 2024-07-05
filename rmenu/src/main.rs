mod config;
mod gui;
mod search;

use rmenu_plugin::Entry;

#[derive(Debug)]
pub struct App {
    css: String,
    theme: String,
    config: config::Config,
    entries: Vec<Entry>,
}

fn main() {
    // temp building of app
    let s = std::fs::read_to_string("/home/andrew/.cache/rmenu/drun.cache").unwrap();
    let entries: Vec<Entry> = serde_json::from_str(&s).unwrap();
    let mut config = config::Config::default();
    config.search.max_length = 5;

    let test = std::thread::spawn(move || {
        println!("running thread!");
        std::thread::sleep(std::time::Duration::from_secs(3));
        println!("exiting!");
    });

    // run gui
    let context = gui::ContextBuilder::default()
        .with_config(config)
        .with_entries(entries)
        .with_bg_threads(vec![test])
        .build();
    gui::run(context)
}
