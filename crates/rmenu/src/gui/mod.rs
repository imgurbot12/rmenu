/*!
 *  Rmenu - Egui implementation
 */
use std::process::exit;

use eframe::egui;

mod icons;
mod page;
use icons::{load_images, IconCache};
use page::Paginator;

use crate::{config::Config, plugins::Plugins};

// v1:
//TODO: fix grid so items expand entire length of window
//TODO: remove prefix and name specification from module definition
//TODO: allow specifying prefix in search to limit enabled plugins
//TODO: build in the actual execute and close part
//TODO: build compilation and install script for easy setup
//TODO: allow for close-on-defocus option in config?

// v2:
//TODO: look into dynamic rendering w/ a custom style config - maybe even css?
//TODO: add additonal plugins: file-browser, browser-url, etc...

/* Function */

// spawn gui application and run it
pub fn launch_gui(cfg: Config, plugins: Plugins) -> Result<(), eframe::Error> {
    let pos = match cfg.rmenu.window_pos {
        Some(pos) => Some(egui::pos2(pos[0], pos[1])),
        None => None,
    };
    let size = cfg.rmenu.window_size.unwrap_or([550.0, 350.0]);
    let options = eframe::NativeOptions {
        transparent: true,
        always_on_top: true,
        decorated: cfg.rmenu.decorate_window,
        centered: cfg.rmenu.centered.unwrap_or(false),
        initial_window_pos: pos,
        initial_window_size: Some(egui::vec2(size[0], size[1])),
        ..Default::default()
    };
    let gui = GUI::new(cfg, plugins);
    eframe::run_native("rmenu", options, Box::new(|_cc| Box::new(gui)))
}

/* Implementation */

struct GUI {
    plugins: Plugins,
    search: String,
    images: IconCache,
    config: Config,
    page: Paginator,
}

impl GUI {
    pub fn new(config: Config, plugins: Plugins) -> Self {
        let mut gui = Self {
            plugins,
            search: "".to_owned(),
            images: IconCache::new(),
            page: Paginator::new(config.rmenu.result_size.clone().unwrap_or(15)),
            config,
        };
        // pre-run empty search to generate cache
        gui.search();
        gui
    }

    // complete search based on current internal search variable
    fn search(&mut self) {
        let results = self.plugins.search(&self.search);
        if results.len() > 0 {
            load_images(&mut self.images, 20, &results);
        }
        self.page.reset(results);
    }

    #[inline]
    fn keyboard(&mut self, ctx: &egui::Context) {
        // tab / ctrl+tab controls
        if ctx.input().key_pressed(egui::Key::Tab) {
            match ctx.input().modifiers.ctrl {
                true => self.page.focus_up(1),
                false => self.page.focus_down(1),
            };
        }
        // arrow-key controls
        if ctx.input().key_pressed(egui::Key::ArrowUp) {
            self.page.focus_up(1);
        }
        if ctx.input().key_pressed(egui::Key::ArrowDown) {
            self.page.focus_down(1)
        }
        // pageup / pagedown controls
        if ctx.input().key_pressed(egui::Key::PageUp) {
            self.page.focus_up(5);
        }
        if ctx.input().key_pressed(egui::Key::PageDown) {
            self.page.focus_down(5);
        }
        // exit controls
        if ctx.input().key_pressed(egui::Key::Escape) {
            exit(1);
        }
    }
}

impl GUI {
    // implement simple topbar searchbar
    #[inline]
    fn simple_search(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        ui.horizontal(|ui| {
            ui.spacing_mut().text_edit_width = size.x;
            let search = egui::TextEdit::singleline(&mut self.search).frame(false);
            let object = ui.add(search);
            if object.changed() {
                self.search();
            }
        });
    }

    // check if results contain any icons at all
    #[inline]
    fn has_icons(&self) -> bool {
        self.page
            .iter()
            .filter(|r| r.icon.is_some())
            .peekable()
            .peek()
            .is_some()
    }

    #[inline]
    fn grid_highlight(&self) -> Box<dyn Fn(usize, &egui::Style) -> Option<egui::Rgba>> {
        let focus = self.page.row_focus();
        Box::new(move |row, style| {
            if row == focus {
                return Some(egui::Rgba::from(style.visuals.faint_bg_color));
            }
            None
        })
    }

    // implement simple scrolling grid-based results pannel
    #[inline]
    fn simple_results(&mut self, ui: &mut egui::Ui) {
        let results = self.page.iter();
        let has_icons = self.has_icons();
        egui::Grid::new("results")
            .with_row_color(self.grid_highlight())
            .show(ui, |ui| {
                for record in results {
                    // render icons (if any were present in set)
                    if has_icons {
                        ui.horizontal(|ui| {
                            if let Some(icon) = record.icon.as_ref().into_option() {
                                if let Ok(image) = self.images.load(&icon) {
                                    let xy = self.config.rmenu.icon_size;
                                    image.show_size(ui, egui::vec2(xy, xy));
                                }
                            }
                        });
                    }
                    // render content
                    ui.label(record.name.as_str());
                    if let Some(comment) = record.comment.as_ref().into_option() {
                        ui.label(comment.as_str());
                    }
                    ui.end_row();
                }
            });
    }
}

impl eframe::App for GUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.keyboard(ctx);
            self.simple_search(ui);
            self.simple_results(ui);
        });
    }
}
