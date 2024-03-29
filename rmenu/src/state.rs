use dioxus::prelude::{use_eval, use_ref, Scope, UseRef};
use regex::Regex;
use rmenu_plugin::Entry;

use crate::config::Config;
use crate::exec::execute;
use crate::search::new_searchfn;
use crate::App;

#[inline]
fn scroll<T>(cx: Scope<T>, pos: usize) {
    let eval = use_eval(cx);
    let js = format!("document.getElementById(`result-{pos}`).scrollIntoView(false)");
    let _ = eval(&js);
}

#[derive(Debug, PartialEq, Clone)]
pub enum KeyEvent {
    Exec,
    Exit,
    MovePrev,
    MoveNext,
    OpenMenu,
    CloseMenu,
    JumpNext,
    JumpPrev,
}

pub struct InnerState {
    pos: usize,
    subpos: usize,
    page: usize,
    search: String,
    event: Option<KeyEvent>,
    search_regex: Option<Regex>,
}

impl InnerState {
    /// Move X Primary Results Upwards
    pub fn move_up(&mut self, x: usize) {
        self.subpos = 0;
        self.pos = std::cmp::max(self.pos, x) - x;
    }

    /// Move X Primary Results Downwards
    pub fn move_down(&mut self, x: usize, max: usize) {
        self.subpos = 0;
        self.pos = std::cmp::min(self.pos + x, max - 1)
    }

    /// Jump a spefified number of results upwards
    #[inline]
    pub fn jump_up(&mut self, jump: usize) {
        self.move_up(jump)
    }

    /// Jump a specified number of results downwards
    pub fn jump_down(&mut self, jump: usize, results: &Vec<&Entry>) {
        let max = std::cmp::max(results.len(), 1);
        self.move_down(jump, max);
    }

    /// Move Up Once With Context of SubMenu
    pub fn move_prev(&mut self) {
        if self.subpos > 0 {
            self.subpos -= 1;
            return;
        }
        self.move_up(1);
    }

    /// Move Down Once With Context of SubMenu
    pub fn move_next(&mut self, results: &Vec<&Entry>) {
        if let Some(result) = results.get(self.pos) {
            if self.subpos > 0 && self.subpos < result.actions.len() - 1 {
                self.subpos += 1;
                return;
            }
        }
        self.jump_down(1, results)
    }
}

#[derive(PartialEq)]
pub struct AppState<'a> {
    state: &'a UseRef<InnerState>,
    app: &'a App,
    results: Vec<&'a Entry>,
}

impl<'a> AppState<'a> {
    /// Spawn new Application State Tracker
    pub fn new<T>(cx: Scope<'a, T>, app: &'a App) -> Self {
        Self {
            state: use_ref(cx, || InnerState {
                pos: 0,
                subpos: 0,
                page: 0,
                search: "".to_string(),
                event: None,
                search_regex: app.config.search.restrict.clone().and_then(|mut r| {
                    if !r.starts_with('^') {
                        r = format!("^{r}")
                    };
                    if !r.ends_with('$') {
                        r = format!("{r}$")
                    };
                    match Regex::new(&r) {
                        Ok(regex) => Some(regex),
                        Err(err) => {
                            log::error!("Invalid Regex Expression: {:?}", err);
                            None
                        }
                    }
                }),
            }),
            app,
            results: vec![],
        }
    }

    /// Create Partial Copy of Self (Not Including Results)
    pub fn partial_copy(&self) -> Self {
        Self {
            state: self.state,
            app: self.app,
            results: vec![],
        }
    }

    /// Retrieve Configuration
    #[inline]
    pub fn config(&self) -> &Config {
        &self.app.config
    }

    /// Retrieve Current Position State
    #[inline]
    pub fn position(&self) -> (usize, usize) {
        self.state.with(|s| (s.pos, s.subpos))
    }

    /// Retrieve Current Search String
    #[inline]
    pub fn search(&self) -> String {
        self.state.with(|s| s.search.clone())
    }

    /// Execute the Current Action
    pub fn execute(&self) {
        let (pos, subpos) = self.position();
        log::debug!("execute {pos} {subpos}");
        let Some(result) = self.results.get(pos) else {
            return;
        };
        log::debug!("result: {result:?}");
        let Some(action) = result.actions.get(subpos) else {
            return;
        };
        log::debug!("action: {action:?}");
        execute(action, self.app.config.terminal.clone());
    }

    /// Set Current Key/Action for Later Evaluation
    #[inline]
    pub fn set_event(&self, event: KeyEvent) {
        self.state.with_mut(|s| s.event = Some(event));
    }

    /// React to Previously Activated KeyEvents
    pub fn handle_events(&self, cx: Scope<'a, App>) {
        match self.state.with(|s| s.event.clone()) {
            None => {}
            Some(event) => {
                match event {
                    KeyEvent::Exit => std::process::exit(0),
                    KeyEvent::Exec => self.execute(),
                    KeyEvent::OpenMenu => self.open_menu(),
                    KeyEvent::CloseMenu => self.close_menu(),
                    KeyEvent::MovePrev => {
                        self.move_prev();
                        let pos = self.position().0;
                        scroll(cx, if pos <= 3 { pos } else { pos + 3 })
                    }
                    KeyEvent::MoveNext => {
                        self.move_next();
                        scroll(cx, self.position().0 + 3)
                    }
                    KeyEvent::JumpPrev => {
                        self.jump_prev();
                        let pos = self.position().0;
                        scroll(cx, if pos <= 3 { pos } else { pos + 3 })
                    }
                    KeyEvent::JumpNext => {
                        self.jump_next();
                        scroll(cx, self.position().0 + 3)
                    }
                };
                self.state.with_mut(|s| s.event = None);
            }
        }
    }

    /// Generate and return Results PTR
    pub fn results(&mut self, entries: &'a Vec<Entry>) -> Vec<&'a Entry> {
        let ratio = self.app.config.page_load;
        let page_size = self.app.config.page_size;
        let (pos, page, search) = self.state.with(|s| (s.pos, s.page, s.search.clone()));
        // determine current page based on position and configuration
        let next = (pos % page_size) as f64 / page_size as f64 > ratio;
        let pos_page = (pos + 1) / page_size + 1 + next as usize;
        let new_page = std::cmp::max(pos_page, page);
        let index = page_size * new_page;
        // update page counter if higher than before
        if new_page > page {
            self.state.with_mut(|s| s.page = new_page);
        }
        // render results and stop at page-limit
        let sfn = new_searchfn(&self.app.config, &search);
        self.results = entries.iter().filter(|e| sfn(e)).take(index).collect();
        self.results.clone()
    }

    /// Update Search and Reset Position
    pub fn set_search(&self, cx: Scope<'_, App>, search: String) {
        // confirm search meets required criteria
        if let Some(min) = self.app.config.search.min_length.as_ref() {
            if search.len() < *min {
                return;
            }
        }
        if let Some(min) = self.app.config.search.min_length.as_ref() {
            if search.len() < *min {
                return;
            }
        }
        let is_match = self.state.with(|s| {
            s.search_regex
                .as_ref()
                .map(|r| r.is_match(&search))
                .unwrap_or(true)
        });
        if !is_match {
            return;
        }
        // update search w/ new content
        self.state.with_mut(|s| {
            s.pos = 0;
            s.subpos = 0;
            s.search = search;
        });
        scroll(cx, 0);
    }

    /// Manually Set Position/SubPosition (with Click)
    pub fn set_position(&self, pos: usize, subpos: usize) {
        self.state.with_mut(|s| {
            s.pos = pos;
            s.subpos = subpos;
        })
    }

    /// Automatically Increase PageCount When Nearing Bottom
    // pub fn scroll_down(&self) {
    //     self.state.with_mut(|s| {
    //         if self.app.config.page_size * s.page < self.app.entries.len() {
    //             s.page += 1;
    //         }
    //     });
    // }

    /// Move Position To SubMenu if it Exists
    pub fn open_menu(&self) {
        let pos = self.state.with(|s| s.pos);
        if let Some(result) = self.results.get(pos) {
            if result.actions.len() > 1 {
                self.state.with_mut(|s| s.subpos += 1);
            }
        }
    }

    // Reset and Close SubMenu Position
    #[inline]
    pub fn close_menu(&self) {
        self.state.with_mut(|s| s.subpos = 0);
    }

    /// Move Up Once With Context of SubMenu
    #[inline]
    pub fn move_prev(&self) {
        self.state.with_mut(|s| s.move_prev());
    }

    /// Move Down Once With Context of SubMenu
    #[inline]
    pub fn move_next(&self) {
        self.state.with_mut(|s| s.move_next(&self.results))
    }

    /// Jump a Configured Distance Up the Results
    #[inline]
    pub fn jump_prev(&self) {
        let distance = self.app.config.jump_dist;
        self.state.with_mut(|s| s.jump_up(distance))
    }

    /// Jump a Configured Distance Down the Results
    #[inline]
    pub fn jump_next(&self) {
        let distance = self.app.config.jump_dist;
        self.state
            .with_mut(|s| s.jump_down(distance, &self.results))
    }
}
