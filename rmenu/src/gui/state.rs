use dioxus::html::geometry::euclid::Point2D;
use dioxus::prelude::*;
use rmenu_plugin::Entry;

use crate::config::{Config, Keybind};
use crate::search::new_searchfn;

type Threads = Vec<std::thread::JoinHandle<()>>;

pub type Entries = Vec<(String, Vec<Entry>)>;

/// Builder Object for Constructing Context
#[derive(Debug, Default)]
pub struct ContextBuilder {
    css: String,
    theme: String,
    config: Option<Config>,
    modes: Vec<String>,
    entries: Entries,
    threads: Threads,
}

impl ContextBuilder {
    pub fn with_css(mut self, css: String) -> Self {
        self.css = css;
        self
    }
    pub fn with_theme(mut self, theme: String) -> Self {
        self.theme = theme;
        self
    }
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    pub fn with_modes(mut self, modes: Vec<String>) -> Self {
        self.modes = modes;
        self
    }
    pub fn with_entries(mut self, entries: Entries) -> Self {
        self.entries = entries;
        self
    }
    pub fn with_bg_threads(mut self, threads: Threads) -> Self {
        self.threads = threads;
        self
    }
    pub fn build(self) -> Context {
        let cfg = self.config.unwrap_or_default();
        let mut ctx = Context {
            quit: false,
            css: self.css,
            theme: self.theme,
            use_icons: cfg.use_icons,
            use_comments: cfg.use_comments,
            config: cfg,

            search: String::new(),
            modes: self.modes,
            entries: vec![],
            all_entries: self.entries,
            num_results: 0,
            threads: self.threads,
        };
        ctx.update_entries();
        ctx
    }
}

/// Custom ContextMenu Tracker
#[derive(Debug, Default)]
pub struct ContextMenu {
    pub entry: Option<usize>,
    pub x: f64,
    pub y: f64,
}

impl ContextMenu {
    #[inline]
    pub fn is_active(&self) -> bool {
        self.entry.is_some()
    }
    pub fn style(&self) -> String {
        if self.entry.is_none() {
            return "display: hidden".to_owned();
        }
        return format!("display: block; left: {}px; top: {}px", self.x, self.y);
    }
    pub fn set<T>(&mut self, index: usize, coords: Point2D<f64, T>) {
        self.entry = Some(index);
        self.x = coords.x;
        self.y = coords.y;
    }
    pub fn reset(&mut self) {
        self.entry = None;
    }
}

/// Global Position Tracker
#[derive(Debug, Default)]
pub struct Position {
    pub pos: usize,
    pub subpos: usize,
}

impl Position {
    pub fn set(&mut self, pos: usize, subpos: usize) {
        self.pos = pos;
        self.subpos = subpos;
    }
    pub fn reset(&mut self) {
        self.set(0, 0)
    }
}

/// Alias for Signal wrapped Position
type Pos = Signal<Position>;
type Results = Signal<Vec<usize>>;

/// Contain and Track Search Results
pub struct Context {
    pub quit: bool,
    pub css: String,
    pub theme: String,
    pub config: Config,
    pub use_icons: bool,
    pub use_comments: bool,
    // search results and controls
    search: String,
    modes: Vec<String>,
    entries: Vec<Entry>,
    all_entries: Entries,
    num_results: usize,
    threads: Threads,
}

impl Context {
    // ** Mode Management **

    fn update_entries(&mut self) {
        self.entries = vec![];
        if self.modes.is_empty() {
            let mode = self
                .all_entries
                .iter()
                .map(|(k, _)| k)
                .take(1)
                .next()
                .expect("empty entries");
            log::warn!("no mode specified. defaulting to {mode:?}");
            self.modes = vec![mode.to_owned()];
        }
        for mode in self.modes.iter() {
            let entries = self
                .all_entries
                .iter()
                .find(|(k, _)| k == mode)
                .map(|(_, v)| v)
                .expect("invalid mode");
            let entries = entries.into_iter().cloned();
            self.entries.extend(entries);
        }
        self.num_results = self.entries.len();
        self.use_icons = self.config.use_icons
            && self
                .entries
                .iter()
                .any(|e| e.icon.is_some() || e.icon_alt.is_some());
        self.use_comments =
            self.config.use_comments && self.entries.iter().any(|e| e.comment.is_some());
    }

    fn get_mode_index(&self) -> usize {
        self.all_entries
            .iter()
            .map(|(k, _)| k)
            .enumerate()
            .find(|(_, k)| self.modes.last().expect("no active modes") == *k)
            .map(|(i, _)| i)
            .expect("invalid mode present")
    }

    pub fn next_mode(&mut self, pos: &mut Pos, results: &mut Results) {
        let _ = pos.with_mut(|p| p.reset());
        let index = self.get_mode_index();
        let mode = match index == self.all_entries.len() - 1 {
            true => self.all_entries.first().expect("empty entries").0.clone(),
            false => self
                .all_entries
                .iter()
                .map(|(k, _)| k)
                .skip(index + 1)
                .take(1)
                .next()
                .expect("cannot find next mode")
                .to_owned(),
        };
        log::info!("switching to next mode: {mode:?}");
        self.modes = vec![mode];
        self.update_entries();
        results.set(self.set_search(&self.search.clone(), pos));
    }

    pub fn prev_mode(&mut self, pos: &mut Pos, results: &mut Results) {
        let _ = pos.with_mut(|p| p.reset());
        let index = self.get_mode_index();
        let mode = match index == 0 {
            true => self.all_entries.last().expect("empty entries").0.clone(),
            false => self
                .all_entries
                .iter()
                .map(|(k, _)| k)
                .skip(index - 1)
                .take(1)
                .next()
                .expect("cannot find prev mode")
                .to_owned(),
        };
        log::info!("switching to prev mode: {mode:?}");
        self.modes = vec![mode];
        self.update_entries();
        results.set(self.set_search(&self.search.clone(), pos));
    }

    // ** Search Results Management  **

    pub fn all_results(&self) -> Vec<usize> {
        (0..self.entries.len()).collect()
    }

    pub fn set_search(&mut self, search: &str, pos: &mut Pos) -> Vec<usize> {
        let _ = pos.with_mut(|p| p.reset());
        let filter = new_searchfn(&self.config, &search);
        let results: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| filter(e))
            .map(|(i, _)| i)
            .collect();
        self.search = search.to_owned();
        self.num_results = results.len();
        results
    }

    pub fn calc_limit(&self, pos: &Pos) -> usize {
        let pos = pos.with(|p| p.pos);
        let page_size = self.config.page_size;
        // calc current page number
        let partial = pos % page_size;
        let page = (pos - partial) / page_size;
        // calc ratio of completion for current page
        let ratio = partial as f64 / page_size as f64;
        let threshold = self.config.page_load;
        // increment total results by 1 page if beyond threshold
        let md = if ratio < threshold { 1 } else { 2 };
        let limit = std::cmp::min((page + md) * page_size, self.entries.len() - 1);
        log::debug!("pos: {pos}, page: {page}, ratio: {ratio}, limit: {limit}");
        limit
    }

    #[inline]
    pub fn get_entry(&self, index: usize) -> &Entry {
        &self.entries[index]
    }

    // ** Keybind Management **

    #[inline]
    fn matches(&self, bind: &Vec<Keybind>, mods: &Modifiers, key: &Code) -> bool {
        bind.iter().any(|b| &b.mods == mods && &b.key == key)
    }

    fn scroll(&self, pos: usize) {
        let js = format!("document.getElementById('result-{pos}').scrollIntoView(false)");
        document::eval(&js);
    }

    fn scroll_up(&self, pos: &Pos) {
        let pos = pos.with(|p| p.pos);
        self.scroll(if pos <= 3 { pos } else { pos + 3 });
    }

    fn scroll_down(&self, pos: &Pos) {
        self.scroll(pos.with(|p| p.pos) + 3);
    }

    //NOTE: using with_mut to trigger rendering update
    pub fn execute(&mut self, index: usize, pos: &mut Pos) {
        let entry = self.get_entry(index);
        let (pos, subpos) = pos.with_mut(|p| (p.pos, p.subpos));
        log::debug!("execute-pos {pos} {subpos}");
        let Some(action) = entry.actions.get(subpos) else {
            return;
        };
        log::debug!("execute-entry {entry:?}");
        log::debug!("execute-action: {action:?}");
        crate::exec::execute(action, self.config.terminal.clone());
        self.quit = true;
    }

    pub fn handle_keybinds(
        &mut self,
        event: KeyboardEvent,
        index: usize,
        pos: &mut Pos,
        results: &mut Results,
    ) {
        let code = event.code();
        let modifiers = event.modifiers();
        let keybinds = &self.config.keybinds;
        if self.matches(&keybinds.exec, &modifiers, &code) {
            self.execute(index, pos);
        } else if self.matches(&keybinds.exit, &modifiers, &code) {
            self.quit = true;
            pos.with_mut(|_| {});
        } else if self.matches(&keybinds.move_next, &modifiers, &code) {
            self.move_next(index, pos);
            self.scroll_down(pos);
        } else if self.matches(&keybinds.move_prev, &modifiers, &code) {
            self.move_prev(pos);
            self.scroll_up(pos);
        } else if self.matches(&keybinds.open_menu, &modifiers, &code) {
            self.open_menu(index, pos);
        } else if self.matches(&keybinds.close_menu, &modifiers, &code) {
            self.close_menu(pos);
        } else if self.matches(&keybinds.jump_next, &modifiers, &code) {
            self.move_down(self.config.jump_dist, pos);
            self.scroll_down(pos);
        } else if self.matches(&keybinds.jump_prev, &modifiers, &code) {
            self.move_up(self.config.jump_dist, pos);
            self.scroll_up(pos);
        } else if self.matches(&keybinds.mode_next, &modifiers, &code) {
            self.next_mode(pos, results);
        } else if self.matches(&keybinds.mode_prev, &modifiers, &code) {
            self.prev_mode(pos, results);
        }
    }

    // ** Position Management **

    pub fn move_up(&self, dist: usize, pos: &mut Pos) {
        pos.with_mut(|p| {
            p.subpos = 0;
            p.pos = std::cmp::max(p.pos, dist) - dist;
        })
    }

    pub fn move_down(&self, dist: usize, pos: &mut Pos) {
        let max_pos = std::cmp::max(self.num_results, 1) - 1;
        pos.with_mut(move |p| {
            p.subpos = 0;
            p.pos = std::cmp::min(p.pos + dist, max_pos);
        })
    }

    pub fn move_prev(&self, pos: &mut Pos) {
        let subpos = pos.with(|p| p.subpos);
        match subpos > 0 {
            true => pos.with_mut(|p| p.subpos -= 1),
            false => self.move_up(1, pos),
        }
    }

    pub fn move_next(&self, index: usize, pos: &mut Pos) {
        let entry = self.get_entry(index);
        let subpos = pos.with(|p| p.subpos);
        if subpos > 0 && subpos < entry.actions.len() - 1 {
            return pos.with_mut(|p| p.subpos += 1);
        }
        self.move_down(1, pos);
    }

    pub fn open_menu(&self, index: usize, pos: &mut Pos) {
        let entry = self.get_entry(index);
        if entry.actions.len() > 1 {
            pos.with_mut(|s| s.subpos += 1);
        }
    }

    #[inline]
    pub fn close_menu(&self, pos: &mut Pos) {
        pos.with_mut(|s| s.subpos = 0);
    }

    //** Cleanup  **

    pub fn cleanup(&mut self) {
        log::debug!("cleaning up {} threads", self.threads.len());
        while !self.threads.is_empty() {
            let thread = self.threads.pop().unwrap();
            let _ = thread.join();
        }
    }
}
