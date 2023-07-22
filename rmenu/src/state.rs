//! GUI Application State Trackers and Utilities
use dioxus::prelude::{use_state, Scope, UseState};
use rmenu_plugin::{Action, Entry};

use crate::App;

#[derive(PartialEq)]
pub struct PosTracker<'a> {
    pos: &'a UseState<usize>,
    subpos: &'a UseState<usize>,
    results: Vec<&'a Entry>,
}

impl<'a> PosTracker<'a> {
    pub fn new(cx: Scope<'a, App>, results: Vec<&'a Entry>) -> Self {
        let pos = use_state(cx, || 0);
        let subpos = use_state(cx, || 0);
        Self {
            pos,
            subpos,
            results,
        }
    }
    /// Move X Primary Results Upwards
    pub fn move_up(&self, x: usize) {
        self.subpos.set(0);
        self.pos.modify(|v| if v >= &x { v - x } else { 0 })
    }
    /// Move X Primary Results Downwards
    pub fn move_down(&self, x: usize) {
        let max = std::cmp::max(self.results.len(), 1);
        self.subpos.set(0);
        self.pos.modify(|v| std::cmp::min(v + x, max - 1))
    }
    /// Get Current Position/SubPosition
    pub fn position(&self) -> (usize, usize) {
        (self.pos.get().clone(), self.subpos.get().clone())
    }
    /// Get Action Linked To The Current Position
    pub fn action(&self) -> Option<&Action> {
        let (pos, subpos) = self.position();
        self.results[pos].actions.get(subpos)
    }
    /// Move Position To SubMenu if it Exists
    pub fn open_menu(&self) {
        let index = *self.pos.get();
        let result = &self.results[index];
        if result.actions.len() > 0 {
            self.subpos.set(1);
        }
    }
    // Reset and Close SubMenu Position
    pub fn close_menu(&self) {
        self.subpos.set(0);
    }
    /// Move Up Once With Context of SubMenu
    pub fn shift_up(&self) {
        if self.subpos.get() > &0 {
            self.subpos.modify(|v| v - 1);
            return;
        }
        self.move_up(1)
    }
    /// Move Down Once With Context of SubMenu
    pub fn shift_down(&self) {
        let index = *self.pos.get();
        if let Some(result) = &self.results.get(index) {
            let subpos = *self.subpos.get();
            if subpos > 0 && subpos < result.actions.len() - 1 {
                self.subpos.modify(|v| v + 1);
                return;
            }
        }
        self.move_down(1)
    }
}
