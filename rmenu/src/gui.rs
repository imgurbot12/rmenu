/// Gui Implementation
use std::str::FromStr;

use askama::Template;
use keyboard_types::{Code, Modifiers};
use rmenu_plugin::Entry;
use serde::Deserialize;
use web_view::*;

use crate::config::{Config, Keybind};
use crate::exec::execute;
use crate::search::build_searchfn;
use crate::AppData;

static INDEX_JS: &'static str = include_str!("../web/index.js");
static INDEX_CSS: &'static str = include_str!("../web/index.css");

#[derive(Debug, Deserialize)]
struct KeyEvent {
    key: String,
    ctrl: bool,
    shift: bool,
}

impl KeyEvent {
    /// Convert Message into Keyboard Modifiers Object
    fn modifiers(&self) -> Modifiers {
        let mut modifiers = Modifiers::default();
        if self.ctrl {
            modifiers |= Modifiers::CONTROL;
        }
        if self.shift {
            modifiers |= Modifiers::SHIFT;
        }
        modifiers
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "click_type")]
#[serde(rename_all = "lowercase")]
enum ClickEvent {
    Single { id: String },
    Double { id: String },
}

#[derive(Debug, Deserialize)]
struct ScrollEvent {
    y: usize,
    maxy: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum Message {
    Search { value: String },
    KeyDown(KeyEvent),
    Click(ClickEvent),
    Scroll(ScrollEvent),
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    css: &'a str,
    search: &'a str,
    results: &'a str,
    config: &'a Config,
    script: &'a str,
}

#[derive(Template)]
#[template(path = "results.html")]
struct ResultsTemplate<'a> {
    start: usize,
    end: usize,
    results: &'a Vec<&'a Entry>,
    config: &'a Config,
}

#[derive(Debug)]
struct AppState<'a> {
    pos: usize,
    subpos: usize,
    page: usize,
    search: String,
    results: Vec<&'a Entry>,
    data: &'a AppData,
}

/// check if the current inputs match any of the given keybindings
#[inline]
fn matches(bind: &Vec<Keybind>, mods: &Modifiers, key: &Code) -> bool {
    bind.iter().any(|b| mods.contains(b.mods) && &b.key == key)
}

impl<'a> AppState<'a> {
    fn new(data: &'a AppData) -> Self {
        Self {
            pos: 0,
            subpos: 0,
            page: 0,
            search: "".to_owned(),
            results: vec![],
            data,
        }
    }

    /// Render Current Page of Results
    fn render_results_page(&self) -> String {
        let size = self.data.config.page_size;
        let start = self.page * size;
        let max = (self.page + 1) * size;
        let nresults = std::cmp::max(self.results.len(), 1);
        let end = std::cmp::min(max, nresults - 1);
        // generate results html from template
        let template = ResultsTemplate {
            start,
            end,
            config: &self.data.config,
            results: &self.results,
        };
        template.render().unwrap()
    }

    /// Update AppState w/ new Search and Render HTML Results
    fn search(&mut self, search: String) -> String {
        let sfn = build_searchfn(&self.data.config, &search);
        self.pos = 0;
        self.page = 0;
        self.search = search;
        self.results = self.data.entries.iter().filter(|e| sfn(e)).collect();
        self.render_results_page()
    }

    /// Execute Action associated w/ Current Position/Subposition
    fn execute(&self) {
        log::debug!("execute {} {}", self.pos, self.subpos);
        let Some(result) = self.results.get(self.pos) else {
            return;
        };
        log::debug!("result: {result:?}");
        let Some(action) = result.actions.get(self.subpos) else {
            return;
        };
        log::debug!("action: {action:?}");
        execute(action, self.data.config.terminal.clone());
    }

    /// Return Additional Page Results to Load (when nessesary)
    fn append_results(&mut self, smooth: bool) -> Option<String> {
        let pos = self.pos as f64;
        let size = self.data.config.page_size as f64;
        let pages = pos / size;
        let ratio = (pos % size) / size;
        if pages > self.page as f64 && ratio > self.data.config.page_load {
            println!("loading next page!");
            self.page += 1;
            let results = self.render_results_page();
            return Some(format!("append({}, {results:?}, {smooth})", self.pos));
        }
        None
    }

    #[inline]
    fn move_up(&mut self, up: usize) -> Option<String> {
        self.pos = std::cmp::max(self.pos, up) - up;
        Some(format!("setpos({})", self.pos))
    }

    #[inline]
    fn move_down(&mut self, down: usize) -> Option<String> {
        let max = (self.page + 1) * self.data.config.page_size;
        let end = std::cmp::min(max, self.results.len()) - 1;
        self.pos = std::cmp::min(self.pos + down, end);
        match self.append_results(false) {
            Some(operation) => Some(operation),
            None => Some(format!("setpos({})", self.pos)),
        }
    }

    /// Handle Search Event sent by UI
    fn search_event(&mut self, search: String) -> Option<String> {
        let results = self.search(search);
        Some(format!("update({results:?})"))
    }

    //TODO: add submenu access and selection
    //TODO: put back main to reference actual config
    //TODO: update sway config to make borderless

    /// Handle Keyboard Events sent by UI
    fn key_event(&mut self, event: KeyEvent) -> Option<String> {
        let code = Code::from_str(&event.key).ok()?;
        let mods = event.modifiers();
        let keybinds = &self.data.config.keybinds;
        if matches(&keybinds.exec, &mods, &code) {
            self.execute();
            None
        } else if matches(&keybinds.exit, &mods, &code) {
            std::process::exit(0);
        } else if matches(&keybinds.move_next, &mods, &code) {
            self.move_down(1)
        } else if matches(&keybinds.move_prev, &mods, &code) {
            self.move_up(1)
        } else if matches(&keybinds.open_menu, &mods, &code) {
            // k_updater.set_event(KeyEvent::OpenMenu);
            None
        } else if matches(&keybinds.close_menu, &mods, &code) {
            // k_updater.set_event(KeyEvent::CloseMenu);
            None
        } else if matches(&keybinds.jump_next, &mods, &code) {
            self.move_down(self.data.config.jump_dist)
        } else if matches(&keybinds.jump_prev, &mods, &code) {
            self.move_up(self.data.config.jump_dist)
        } else {
            None
        }
    }

    /// Parse Position/Subposition from HTML Element Id
    fn parse_id_pos(&self, id: &str) -> Option<(usize, usize)> {
        let mut chunks = id.split("-");
        chunks.next()?;
        let pos = chunks.next()?.parse().ok()?;
        let mut subpos = 0;
        if chunks.next().is_some() {
            subpos = chunks.next()?.parse().ok()?;
        }
        Some((pos, subpos))
    }

    /// Handle Single/Doubleclicks Events sent by UI
    fn click_event(&mut self, event: ClickEvent) -> Option<String> {
        match event {
            ClickEvent::Single { id } => {
                if let Some((pos, subpos)) = self.parse_id_pos(&id) {
                    self.pos = pos;
                    self.subpos = subpos;
                    return match self.append_results(true) {
                        Some(op) => Some(op),
                        None => Some(format!("setpos({pos}, true)")),
                    };
                }
            }
            ClickEvent::Double { id } => {
                if let Some((pos, subpos)) = self.parse_id_pos(&id) {
                    self.pos = pos;
                    self.subpos = subpos;
                    self.execute();
                }
            }
        };
        None
    }

    /// Handle Scrolling Events sent by UI
    fn scroll_event(&mut self, event: ScrollEvent) -> Option<String> {
        // load additonal results when scrolled near bottom
        let ratio = event.y as f64 / event.maxy as f64;
        if ratio >= self.data.config.page_load {
            self.page += 1;
            let results = self.render_results_page();
            return Some(format!("append(null, {results:?})"));
        }
        None
    }

    /// Parse and Process Raw UI Messages
    fn handle_event(&mut self, event: &str) -> Option<String> {
        let message: Message = serde_json::from_str(&event).unwrap();
        match message {
            Message::Search { value } => self.search_event(value),
            Message::KeyDown(event) => self.key_event(event),
            Message::Click(event) => self.click_event(event),
            Message::Scroll(event) => self.scroll_event(event),
        }
    }

    /// Render Initial Index HTML w/ AppState
    fn render_index(&mut self) -> String {
        let results = self.search("".to_owned());
        let index = IndexTemplate {
            css: &format!("{INDEX_CSS}\n{}\n{}", self.data.css, self.data.theme),
            search: &self.search,
            results: &results,
            config: &self.data.config,
            script: &INDEX_JS,
        };
        index.render().unwrap()
    }
}

/// Run GUI Applcation via WebView
pub fn run(data: AppData) {
    // build app-state
    let mut state = AppState::new(&data);
    let html = state.render_index();
    // spawn webview instance
    let size = &state.data.config.window.size;
    web_view::builder()
        .title(&state.data.config.window.title)
        .content(Content::Html(html))
        .frameless(!state.data.config.window.decorate)
        .size(size.width as i32, size.height as i32)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, msg| {
            if let Some(js) = state.handle_event(msg) {
                webview.eval(&js)?;
            };
            Ok(())
        })
        .run()
        .unwrap();
}
