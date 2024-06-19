use dioxus::prelude::*;
use rmenu_plugin::Entry;

/// Dioxus Search Entry Component
#[derive(PartialEq, Clone, Props)]
pub struct SearchEntry {
    pub index: usize,
    pub entry: MappedSignal<Entry>,
    pub pos: usize,
    pub subpos: usize,
}
