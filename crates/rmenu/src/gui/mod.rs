/*!
 *  Rmenu - Egui implementation
 */
use std::cmp::min;

use eframe::egui;
use rmenu_plugin::Entry;

mod icons;
use icons::{load_images, IconCache};

use crate::{config::Config, plugins::Plugins};

/* Function */

// spawn gui application and run it
pub fn launch_gui(cfg: Config, plugins: Plugins) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(cfg.rmenu.window_width, cfg.rmenu.window_height)),
        ..Default::default()
    };
    let gui = GUI::new(cfg, plugins);
    eframe::run_native("rmenu", options, Box::new(|_cc| Box::new(gui)))
}

/* Implementation */

struct GUI {
    plugins: Plugins,
    search: String,
    results: Vec<Entry>,
    focus: usize,
    focus_updated: bool,
    images: IconCache,
    config: Config,
}

impl GUI {
    pub fn new(config: Config, plugins: Plugins) -> Self {
        let mut gui = Self {
            config,
            plugins,
            search: "".to_owned(),
            results: vec![],
            focus: 0,
            focus_updated: false,
            images: IconCache::new(),
        };
        // pre-run empty search to generate cache
        gui.search();
        gui
    }

    // complete search based on current internal search variable
    fn search(&mut self) {
        // update variables and complete search
        self.focus = 0;
        self.results = self.plugins.search(&self.search);
        self.focus_updated = true;
        // load icons in background
        if self.results.len() > 0 {
            load_images(&mut self.images, 20, &self.results);
        }
    }

    #[inline]
    fn set_focus(&mut self, focus: usize) {
        self.focus = focus;
        self.focus_updated = true;
    }

    // shift focus up a certain number of rows
    #[inline]
    fn focus_up(&mut self, shift: usize) {
        self.set_focus(self.focus - min(shift, self.focus));
    }

    // shift focus down a certain number of rows
    fn focus_down(&mut self, shift: usize) {
        let results = self.results.len();
        let max_pos = if results > 0 { results - 1 } else { 0 };
        self.set_focus(min(self.focus + shift, max_pos));
    }

    #[inline]
    fn keyboard(&mut self, ctx: &egui::Context) {
        // tab / ctrl+tab controls
        if ctx.input().key_pressed(egui::Key::Tab) {
            match ctx.input().modifiers.ctrl {
                true => self.focus_down(1),
                false => self.focus_up(1),
            };
        }
        // arrow-key controls
        if ctx.input().key_pressed(egui::Key::ArrowUp) {
            self.focus_up(1);
        }
        if ctx.input().key_pressed(egui::Key::ArrowDown) {
            self.focus_down(1)
        }
        // pageup / pagedown controls
        if ctx.input().key_pressed(egui::Key::PageUp) {
            self.focus_up(5);
        }
        if ctx.input().key_pressed(egui::Key::PageDown) {
            self.focus_down(5);
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
            let search = ui.text_edit_singleline(&mut self.search);
            if search.changed() {
                self.search();
            }
        });
    }

    // check if results contain any icons at all
    #[inline]
    fn has_icons(&self) -> bool {
        self.results
            .iter()
            .filter(|r| r.icon.is_some())
            .peekable()
            .peek()
            .is_some()
    }

    #[inline]
    fn grid_highlight(&self) -> Box<dyn Fn(usize, &egui::Style) -> Option<egui::Rgba>> {
        let focus = self.focus;
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
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui, viewport| {
                // calculate top/bottom positions and size of rows
                let spacing = ui.spacing();
                let top_y = viewport.min.y;
                let bot_y = viewport.max.y;
                let row_h = spacing.interact_size.y + spacing.item_spacing.y;
                // render results and their related fields
                let results = &self.results;
                let has_icons = self.has_icons();
                egui::Grid::new("results")
                    .with_row_color(self.grid_highlight())
                    .show(ui, |ui| {
                        for (n, record) in results.iter().enumerate() {
                            // render icon if enabled and within visible bounds
                            if has_icons {
                                ui.horizontal(|ui| {
                                    let y = n as f32 * row_h;
                                    if n == 0 || (y < bot_y && y > top_y) {
                                        if let Some(icon) = record.icon.as_ref().into_option() {
                                            if let Ok(image) = self.images.load(&icon) {
                                                let xy = self.config.rmenu.icon_size;
                                                image.show_size(ui, egui::vec2(xy, xy));
                                            }
                                        }
                                    }
                                });
                            }
                            // render main label
                            let label = ui.label(record.name.as_str());
                            //   scroll to laebl when focus shifts
                            if n == self.focus && self.focus_updated {
                                label.scroll_to_me(None);
                                self.focus_updated = false;
                            }
                            // render comment (if any)
                            if let Some(comment) = record.comment.as_ref().into_option() {
                                ui.label(comment.as_str());
                            }
                            ui.end_row();
                        }
                    });
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
