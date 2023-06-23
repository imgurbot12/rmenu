
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

use rmenu_plugin::Entry;

use crate::{config, plugins::Plugins};

/// Spawn GUI instance with the specified config and plugins
pub fn launch_gui(cfg: config::Config, plugins: Plugins) -> Result<(), String> {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    let size = cfg.rmenu.window_size.unwrap_or([550.0, 350.0]);
    let gui  = GUI::new(cfg, plugins);
    dioxus_desktop::launch_cfg(
        App,
        Config::default().with_window(WindowBuilder::new().with_resizable(true).with_inner_size(
            dioxus_desktop::wry::application::dpi::LogicalSize::new(size[0], size[1]),
        )),
    );
    Ok(())
}

struct GUI {
    plugins: Plugins,
    search: String,
    config: config::Config,
}

impl GUI {
    pub fn new(config: config::Config, plugins: Plugins) -> Self {
        Self {
            config,
            plugins,
            search: "".to_owned(),
        }
    }
    
    fn search(&mut self, search: &str) -> Vec<Entry> {
        self.plugins.search(search)
    }

    pub fn app(&mut self, cx: Scope) -> Element {
        let results = self.search("");
        cx.render(rsx! { 
            div {  
                h1 { "Hello World!" }
                result.iter().map(|entry| {
                    div {
                        div { entry.name }
                        div { entry.comment }
                    }
                })
            }
        })
    }

}
