use dioxus::prelude::*;
use rmenu_plugin::Entry;

use crate::config::{Config, Keybind};
use crate::search::new_searchfn;

/// Global Position Tracker
#[derive(Default)]
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

/// Contain and Track Search Results
pub struct Context {
    pub css: String,
    pub theme: String,
    pub config: Config,
    // search results and controls
    entries: Vec<Entry>,
    num_results: usize,
}

impl Context {
    pub fn new(css: String, theme: String, config: Config, entries: Vec<Entry>) -> Self {
        println!(
            "page_size: {}, threshold: {}",
            config.page_size, config.page_load
        );
        Self {
            num_results: entries.len(),
            entries,
            config,
            theme,
            css,
        }
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
        let limit = (page + md) * page_size;
        println!("pos: {pos}, page: {page}, ratio: {ratio}, limit: {limit}");
        limit
    }

    #[inline]
    pub fn get_entry(&self, index: usize) -> &Entry {
        &self.entries[index]
    }

    // ** Keybind Management **

    #[inline]
    fn matches(&self, bind: &Vec<Keybind>, mods: &Modifiers, key: &Code) -> bool {
        bind.iter().any(|b| mods.contains(b.mods) && &b.key == key)
    }

    pub fn handle_keybinds(&self, event: KeyboardEvent, index: usize, pos: &mut Pos) {
        let code = event.code();
        let modifiers = event.modifiers();
        let keybinds = &self.config.keybinds;
        if self.matches(&keybinds.exec, &modifiers, &code) {
            println!("exec!");
        } else if self.matches(&keybinds.exit, &modifiers, &code) {
            std::process::exit(0);
        } else if self.matches(&keybinds.move_next, &modifiers, &code) {
            self.move_next(index, pos);
        } else if self.matches(&keybinds.move_prev, &modifiers, &code) {
            self.move_prev(pos);
        } else if self.matches(&keybinds.open_menu, &modifiers, &code) {
        } else if self.matches(&keybinds.close_menu, &modifiers, &code) {
        } else if self.matches(&keybinds.jump_next, &modifiers, &code) {
            self.move_down(self.config.jump_dist, pos);
        } else if self.matches(&keybinds.jump_prev, &modifiers, &code) {
            self.move_up(self.config.jump_dist, pos);
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
        let max_pos = self.num_results;
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
        println!("moving down 1");
        self.move_down(1, pos);
    }
}
