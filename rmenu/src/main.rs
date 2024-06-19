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
    let s = std::fs::read_to_string("/home/andrew/.cache/rmenu/run.cache").unwrap();
    let entries: Vec<Entry> = serde_json::from_str(&s).unwrap();
    let mut config = config::Config::default();
    config.search.max_length = 5;

    let app = App {
        css: String::new(),
        theme: String::new(),
        config,
        entries,
    };
    // run gui
    gui::run(app);
}
