use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

mod entry;
mod state;

pub use state::ContextBuilder;
use state::{Context, Position};

const DEFAULT_CSS_CONTENT: &'static str = include_str!("../../public/default.css");

type Ctx = Rc<RefCell<Context>>;

pub fn run(ctx: Context) {
    let window = dioxus_desktop::WindowBuilder::default()
        .with_title(ctx.config.window.title.clone())
        .with_focused(ctx.config.window.focus)
        .with_decorations(ctx.config.window.decorate)
        .with_transparent(ctx.config.window.transparent)
        .with_always_on_top(ctx.config.window.always_top)
        .with_inner_size(ctx.config.window.logical_size())
        .with_fullscreen(ctx.config.window.get_fullscreen())
        .with_theme(ctx.config.window.get_theme());
    let config = dioxus_desktop::Config::default().with_window(window);
    LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(Rc::new(RefCell::new(ctx)))
        .launch(gui_main);
    println!("hello world!");
}

#[derive(Clone, Props)]
struct Row {
    position: Signal<Position>,
    search_index: usize,
    entry_index: usize,
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.entry_index == other.entry_index
    }
}

fn gui_entry(mut row: Row) -> Element {
    // retrieve entry information based on index
    let ctx = use_context::<Ctx>();
    let context = ctx.borrow();
    let entry = context.get_entry(row.entry_index);
    let hover_select = context.config.hover_select;
    let (pos, subpos) = row.position.with(|p| (p.pos, p.subpos));
    // build element from entry
    let aclass = (pos == row.search_index && subpos > 0)
        .then_some("active")
        .unwrap_or_default();
    let rclass = (pos == row.search_index && subpos == 0)
        .then_some("selected")
        .unwrap_or_default();
    rsx! {
        div {
            class: "result-entry",
            div {
                id: "result-{row.search_index}",
                class: "result {rclass}",
                // actions
                onmouseenter: move |_| {
                    if hover_select {
                        row.position.with_mut(|p| p.set(row.search_index, 0));
                    }
                },
                onclick: move |_| {
                    row.position.with_mut(|p| p.set(row.search_index, 0));
                },
                ondoubleclick: move |_| {
                    // row.position.with_mut(|p| p.set(row.search_index, 0));
                },
                // content
                if context.config.use_comments {
                    {rsx! {
                        div {
                            class: "name",
                            dangerous_inner_html: "{entry.name}"
                        }
                        div {
                            class: "comment",
                            dangerous_inner_html: entry.comment.as_ref().map(|s| s.as_str()).unwrap_or(""),
                        }
                    }}
                } else {
                    {rsx! {
                        div {
                            class: "entry",
                            dangerous_inner_html: "{entry.name}"
                        }
                    }}
                }
            }
        }
    }
}

fn gui_main() -> Element {
    // build context and signals for state
    let ctx = use_context::<Ctx>();
    let window = dioxus_desktop::use_window();
    let mut search = use_signal(String::new);
    let mut position = use_signal(Position::default);
    let mut results = use_signal(|| ctx.borrow().all_results());

    // refocus on input
    let js = format!("setTimeout(() => {{ document.getElementById('search').focus() }}, 100)");
    eval(&js);

    // configure exit cleanup function
    use_drop(move || {
        let ctx = consume_context::<Ctx>();
        ctx.borrow_mut().cleanup();
    });

    // update search results on search
    let effect_ctx = use_context::<Ctx>();
    use_effect(move || {
        let search = search();
        results.set(effect_ctx.borrow_mut().set_search(&search, &mut position));
    });

    // declare keyboard handler
    let key_ctx = use_context::<Ctx>();
    let keydown = move |e: KeyboardEvent| {
        let context = key_ctx.borrow();
        // calculate current entry index
        let pos = position.with(|p| p.pos);
        let index = results.with(|r| r.get(pos).cloned().unwrap_or(0));
        // handle events
        let quit = context.handle_keybinds(e, index, &mut position);
        // handle quit event
        if quit {
            window.set_visible(false);
            spawn(async move {
                // wait for window to vanish
                let time = std::time::Duration::from_millis(50);
                let window = dioxus_desktop::use_window();
                while window.is_visible() {
                    tokio::time::sleep(time).await;
                }
                // actually close app after it becomes invisible
                window.close();
            });
        }
    };

    let context = ctx.borrow();
    let pattern = context.config.search.restrict.clone();
    let maxlength = context.config.search.max_length as i64;
    let max_result = context.calc_limit(&position);
    rsx! {
        style { "{DEFAULT_CSS_CONTENT}" }
        style { "{context.theme}" }
        style { "{context.css}" }
        div {
            id: "content",
            class: "content",
            onkeydown: keydown,
            div {
                id: "navbar",
                class: "navbar",
                input {
                    id: "search",
                    value: "{search}",
                    pattern: pattern,
                    maxlength: maxlength,
                    oninput: move |e| search.set(e.value()),
                }
            }
            div {
                id: "results",
                class: "results",
                for (pos, index) in results().iter().take(max_result).enumerate() {
                    gui_entry {
                        key: "{pos}-{index}",
                        position,
                        search_index: pos,
                        entry_index: *index,
                    }
                }
            }
        }
    }
}
