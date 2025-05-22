use std::sync::{Arc, RwLock};

use dioxus::prelude::*;

mod entry;
mod image;
mod state;

pub use state::ContextBuilder;
use state::{Context, ContextMenu, Position};

const DEFAULT_CSS_CONTENT: &'static str = include_str!("../../public/default.css");

type Ctx = Arc<RwLock<Context>>;

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
    let config = dioxus_desktop::Config::default()
        .with_window(window)
        .with_menu(None)
        .with_disable_context_menu(true);

    let context = Arc::new(RwLock::new(ctx));
    LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(context)
        .launch(gui_main);
}

fn gui_main() -> Element {
    // build context and signals for state
    let ctx = use_context::<Ctx>();

    let mut search = use_signal(String::new);
    let mut position = use_signal(Position::default);
    let mut results = use_signal(|| ctx.read().expect("failed to read ctx").all_results());
    let mut ctx_menu = use_signal(ContextMenu::default);

    // refocus on input
    let js = format!("setTimeout(() => {{ document.getElementById('search').focus() }}, 100)");
    document::eval(&js);

    // configure exit cleanup function
    use_drop(move || {
        let ctx = consume_context::<Ctx>();
        ctx.write().expect("failed to write ctx").cleanup();
    });

    // update search results on search
    let effect_ctx = use_context::<Ctx>();
    use_effect(move || {
        let search = search();
        results.set(
            effect_ctx
                .write()
                .expect("failed to write ctx")
                .set_search(&search, &mut position),
        );
    });

    // declare keyboard handler
    #[cfg(debug_assertions)]
    let window = dioxus_desktop::use_window();
    let key_ctx = use_context::<Ctx>();
    let keydown = move |e: KeyboardEvent| {
        let mut context = key_ctx.write().expect("failed to write ctx");
        // suport console key
        #[cfg(debug_assertions)]
        if e.code() == Code::Backquote {
            window.devtool();
            return;
        }
        // calculate current entry index
        let pos = position.with(|p| p.pos);
        let index = results.with(|r| r.get(pos).cloned().unwrap_or(0));
        // handle events
        context.handle_keybinds(e, index, &mut position, &mut results);
    };

    // handle quit event
    let window = dioxus_desktop::use_window();
    let context = ctx.read().expect("failed to read ctx");
    if context.quit {
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

    // prevent cursor from jumping within input on arrow up/down
    let disable_arrows = |e: KeyboardEvent| {
        let code = e.code();
        if code == Code::ArrowUp || code == Code::ArrowDown {
            e.prevent_default();
        }
    };

    let pattern = context.config.search.restrict.clone();
    let maxlength = context.config.search.max_length as i64;
    let max_result = context.calc_limit(&position);
    rsx! {
        style { "{DEFAULT_CSS_CONTENT}" }
        style { "{context.theme}" }
        style { "{context.css}" }
        // menu content
        div {
            id: "body",
            class: "body",
            div {
                id: "content",
                class: "content",
                onclick: move |_| {
                    ctx_menu.with_mut(|m| m.reset());
                },
                onkeydown: keydown,
                div {
                    id: "navbar",
                    class: "navbar",
                    input {
                        id: "search",
                        value: "{search}",
                        pattern: pattern,
                        maxlength: maxlength,
                        placeholder: "{context.placeholder}",
                        oninput: move |e| search.set(e.value()),
                        onkeydown: disable_arrows,
                    }
                }
                div {
                    id: "results",
                    class: "results",
                    for (pos, index) in results().iter().take(max_result).enumerate() {
                        gui_entry {
                            key: "{pos}-{index}",
                            ctx_menu,
                            position,
                            search_index: pos,
                            entry_index: *index,
                        }
                    }
                }
            }
        }
        // custom context menu
        context_menu { ctx_menu, position }
    }
}

#[derive(Clone, Props)]
struct Row {
    ctx_menu: Signal<ContextMenu>,
    position: Signal<Position>,
    search_index: usize,
    entry_index: usize,
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.entry_index == other.entry_index
    }
}

#[inline]
fn render_image(image: Option<&String>, alt: Option<&String>) -> Element {
    if let Some(img) = image {
        if img.ends_with(".svg") {
            if let Some(content) = image::convert_svg(img.to_owned()) {
                return rsx! { img { class: "image", src: "{content}" } };
            }
        }
        if image::image_exists(img.to_owned()) {
            return rsx! { img { class: "image", src: "{img}" } };
        }
    }
    let alt = alt.map(|s| s.as_str()).unwrap_or_else(|| "?");
    return rsx! { div { class: "icon_alt", dangerous_inner_html: "{alt}" } };
}

fn gui_entry(mut row: Row) -> Element {
    // retrieve entry information based on index
    let ctx = use_context::<Ctx>();
    let context = ctx.read().expect("failed to read ctx");
    let entry = context.get_entry(row.entry_index);
    let hover_select = context.config.hover_select;
    let (pos, subpos) = row.position.with(|p| (p.pos, p.subpos));

    // build element from entry
    let single_click = context.config.single_click;
    let context_menu = context.config.context_menu;
    let action_select = pos == row.search_index && subpos > 0;
    let aclass = action_select.then_some("active").unwrap_or_default();
    let rclass = (pos == row.search_index && subpos == 0)
        .then_some("selected")
        .unwrap_or_default();

    // context menu event handler
    let contextmenu = move |e: Event<MouseData>| {
        if context_menu {
            let mouse: MouseData = e.downcast::<SerializedMouseData>().cloned().unwrap().into();
            let coords = mouse.page_coordinates();
            row.ctx_menu.with_mut(|c| c.set(row.entry_index, coords));
        }
    };

    // mouse enter event handler
    let menu_active = row.ctx_menu.with(|m| m.is_active());
    let mouseenter = move |_| {
        if hover_select && !menu_active {
            row.position.with_mut(|p| p.set(row.search_index, 0));
        }
    };

    // onclick event handler
    let result_ctx1 = use_context::<Ctx>();
    let onclick = move |_| {
        row.position.with_mut(|p| p.set(row.search_index, 0));
        if single_click && !menu_active {
            let mut pos = row.position.clone();
            result_ctx1
                .write()
                .expect("failed to write ctx")
                .execute(row.entry_index, &mut pos);
        }
    };

    // doubleclick event handler
    let result_ctx2 = use_context::<Ctx>();
    let doubleclick = move |_| {
        if !menu_active {
            let mut pos = row.position.clone();
            result_ctx2
                .write()
                .expect("failed to write ctx")
                .execute(row.entry_index, &mut pos);
        }
    };

    rsx! {
        div {
            class: "result-entry",
            // main-entry
            div {
                id: "result-{row.search_index}",
                class: "result {rclass}",
                // actions
                oncontextmenu: contextmenu,
                onmouseenter: mouseenter,
                onclick: onclick,
                ondoubleclick: doubleclick,
                // content
                if context.use_icons {
                    {rsx! {
                        div {
                            class: "icon",
                            {render_image(entry.icon.as_ref(), entry.icon_alt.as_ref())}
                        }
                    }}
                }
                if context.use_comments {
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
            // actions
            div {
                id: "result-{row.search_index}-actions",
                class: "actions {aclass}",
                for (idx, action, classes, ctx, ctx2) in entry.actions
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(idx, act)| {
                        let ctx = use_context::<Ctx>();
                        let ctx2 = use_context::<Ctx>();
                        let classes = (idx == subpos).then_some("selected").unwrap_or_default();
                        (idx, act, classes, ctx, ctx2)
                    })
                {
                    div {
                        class: "action {classes}",
                        // actions
                        onmouseenter: move |_| {
                            if hover_select {
                                row.position.with_mut(|p| p.set(row.search_index, idx));
                            }
                        },
                        onclick: move |_| {
                            row.position.with_mut(|p| p.set(row.search_index, 0));
                            if single_click {
                                let mut pos = row.position.clone();
                                ctx.write().expect("failed to write ctx").execute(row.entry_index, &mut pos);
                            }
                        },
                        ondoubleclick: move |_| {
                            let mut pos = row.position.clone();
                            ctx2.write().expect("failed to write ctx").execute(row.entry_index, &mut pos);
                        },
                        // content
                        div {
                            class: "action-name",
                            dangerous_inner_html: "{action.name}"
                        }
                        div {
                            class: "action-comment",
                            dangerous_inner_html: action.comment.as_ref().map(|s| s.as_str()).unwrap_or(""),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn context_menu(ctx_menu: Signal<ContextMenu>, position: Signal<Position>) -> Element {
    let ctx = use_context::<Ctx>();
    let context = ctx.read().expect("failed to read ctx");
    let index = ctx_menu.with(|c| c.entry);
    if index.is_none() {
        return rsx! {};
    }
    let index = index.unwrap_or(0);
    let entry = context.get_entry(index);
    rsx! {
        div {
            id: "context-menu",
            class: "context-menu",
            style: ctx_menu.with(|m| m.style()),
            ul {
                class: "menu",
                for (idx, name, ctx) in entry.actions
                    .iter()
                    .enumerate()
                    .map(|(idx, action)| {
                        let name = match idx == 0 {
                            true => format!("Launch {:?}", entry.name),
                            false => action.name.to_owned(),
                        };
                        (idx, name, use_context::<Ctx>())
                    }) {
                    li {
                        class: "menu-action",
                        a {
                            href: "#",
                            onclick: move |_| {
                                position.with_mut(|p| p.subpos = idx);
                                let mut pos = position.clone();
                                ctx.write().expect("failed to write ctx").execute(index, &mut pos);
                            },
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}
