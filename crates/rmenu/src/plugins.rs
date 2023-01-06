use abi_stable::std_types::RString;
use rmenu_plugin::internal::{load_plugin, Plugin};
use rmenu_plugin::Entry;

use super::config::PluginConfig;

/// Convenient wrapper used to execute configured plugins
pub struct Plugins {
    plugins: Vec<Plugin>,
}

impl Plugins {
    pub fn new(enable: Vec<String>, plugins: Vec<PluginConfig>) -> Self {
        Self {
            plugins: plugins
                .into_iter()
                .map(|p| unsafe { load_plugin(&p.path, &p.config) }.expect("failed to load plugin"))
                .filter(|plugin| enable.contains(&plugin.module.name().as_str().to_owned()))
                .collect(),
        }
    }

    /// complete search w/ the configured plugins
    pub fn search(&mut self, search: &str) -> Vec<Entry> {
        let mut entries = vec![];
        for plugin in self.plugins.iter_mut() {
            let found = plugin.module.search(RString::from(search));
            entries.append(&mut found.into());
            continue;
        }
        entries
    }
}
