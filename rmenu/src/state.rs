/// Application State Trackers and Utilities
use dioxus::prelude::UseState;
use rmenu_plugin::Entry;

pub struct PosTracker<'a> {
    pos: &'a UseState<usize>,
    subpos: usize,
    results: &'a Vec<Entry>,
}

impl<'a> PosTracker<'a> {
    pub fn new(pos: &UseState<usize>, results: &Vec<Entry>) -> Self {
        Self {
            pos,
            results,
            subpos: 0,
        }
    }
    /// Move X Primary Results Upwards
    pub fn move_up(&mut self, x: usize) {
        self.subpos = 0;
        self.pos.modify(|v| if v >= &x { v - x } else { 0 })
    }
    /// Move X Primary Results Downwards
    pub fn move_down(&mut self, x: usize) {
        self.subpos = 0;
        self.pos
            .modify(|v| std::cmp::min(v + x, self.results.len()))
    }
    /// Get Current Position/SubPosition
    pub fn position(&self) -> (usize, usize) {
        (*self.pos.get(), self.subpos)
    }
    /// Move Position To SubMenu if it Exists
    pub fn open_menu(&mut self) {
        self.subpos = 1;
    }
    // Reset and Close SubMenu Position
    pub fn close_menu(&mut self) {
        self.subpos = 0;
    }
    /// Move Up Once With Context of SubMenu
    pub fn shift_up(&mut self) {
        let index = *self.pos.get();
        if index == 0 {
            return;
        }
        let result = self.results[index];
        if self.subpos > 0 {
            self.subpos -= 1;
            return;
        }
        self.move_up(1)
    }
    /// Move Down Once With Context of SubMenu
    pub fn shift_down(&mut self) {
        let index = *self.pos.get();
        if index == 0 {
            return self.move_down(1);
        }
        let result = self.results[index];
        if self.subpos > 0 && self.subpos < result.actions.len() {
            self.subpos += 1;
            return;
        }
        self.move_down(1)
    }
}
