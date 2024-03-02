//! RMENU GUI Implementation using Dioxus
#![allow(non_snake_case)]
use std::fmt::Display;

use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::{Code, Modifiers};
use rmenu_plugin::Entry;

use crate::config::Keybind;
use crate::state::{AppState, KeyEvent};
use crate::{App, DEFAULT_CSS_CONTENT};

/// spawn and run the app on the configured platform
pub fn run(app: App) {
    let theme = match app.config.window.dark_mode {
        Some(dark) => match dark {
            true => Some(dioxus_desktop::tao::window::Theme::Dark),
            false => Some(dioxus_desktop::tao::window::Theme::Light),
        },
        None => None,
    };
    let builder = dioxus_desktop::WindowBuilder::new()
        .with_title(app.config.window.title.clone())
        .with_inner_size(app.config.window.size)
        .with_position(app.config.window.position)
        .with_focused(app.config.window.focus)
        .with_decorations(app.config.window.decorate)
        .with_transparent(app.config.window.transparent)
        .with_always_on_top(app.config.window.always_top)
        .with_fullscreen(app.config.window.get_fullscreen())
        .with_theme(theme);
    let config = dioxus_desktop::Config::new().with_window(builder);
    dioxus_desktop::launch_with_props(App, app, config);
}

#[derive(PartialEq, Props)]
struct GEntry<'a> {
    pos: usize,
    subpos: usize,
    index: usize,
    entry: &'a Entry,
    state: AppState<'a>,
}

#[inline]
fn render_comment(comment: Option<&String>) -> &str {
    comment.map(|s| s.as_str()).unwrap_or("")
}

#[inline]
fn render_image<'a, T>(
    cx: Scope<'a, T>,
    image: Option<&String>,
    alt: Option<&String>,
) -> Element<'a> {
    if let Some(img) = image {
        if img.ends_with(".svg") {
            if let Some(content) = crate::image::convert_svg(img.to_owned()) {
                return cx.render(rsx! { img { class: "image", src: "{content}" } });
            }
        }
        if crate::image::image_exists(img.to_owned()) {
            return cx.render(rsx! { img { class: "image", src: "{img}" } });
        }
    }
    let alt = alt.map(|s| s.as_str()).unwrap_or_else(|| "?");
    return cx.render(rsx! { div { class: "icon_alt", dangerous_inner_html: "{alt}" } });
}

/// render a single result entry w/ the given information
fn TableEntry<'a>(cx: Scope<'a, GEntry<'a>>) -> Element<'a> {
    // build css classes for result and actions (if nessesary)
    let main_select = cx.props.index == cx.props.pos;
    let action_select = main_select && cx.props.subpos > 0;
    let action_classes = match action_select {
        true => "active",
        false => "",
    };
    let multi_classes = match cx.props.entry.actions.len() > 1 {
        true => "submenu",
        false => "",
    };
    let result_classes = match main_select && !action_select {
        true => "selected",
        false => "",
    };
    // build sub-actions if present
    let actions = cx
        .props
        .entry
        .actions
        .iter()
        .skip(1)
        .enumerate()
        .map(|(idx, action)| {
            let act_class = match action_select && idx + 1 == cx.props.subpos {
                true => "selected",
                false => "",
            };
            cx.render(rsx! {
                div {
                    class: "action {act_class}",
                    onclick: move |_| cx.props.state.set_position(cx.props.index, idx + 1),
                    ondblclick: |_| cx.props.state.set_event(KeyEvent::Exec),
                    div {
                        class: "action-name",
                        dangerous_inner_html: "{action.name}"
                    }
                    div {
                        class: "action-comment",
                        render_comment(action.comment.as_ref())
                    }
                }
            })
        });
    cx.render(rsx! {
        div {
            class: "result-entry",
            div {
                id: "result-{cx.props.index}",
                class: "result {result_classes} {multi_classes}",
                // onmouseenter: |_| cx.props.state.set_position(cx.props.index, 0),
                onclick: |_| cx.props.state.set_position(cx.props.index, 0),
                ondblclick: |_| cx.props.state.set_event(KeyEvent::Exec),
                if cx.props.state.config().use_icons {
                    cx.render(rsx! {
                        div {
                            class: "icon",
                            render_image(cx, cx.props.entry.icon.as_ref(), cx.props.entry.icon_alt.as_ref())
                        }
                    })
                }
                match cx.props.state.config().use_comments {
                    true => cx.render(rsx! {
                        div {
                            class: "name",
                            dangerous_inner_html: "{cx.props.entry.name}"
                        }
                        div {
                            class: "comment",
                            dangerous_inner_html: render_comment(cx.props.entry.comment.as_ref())
                        }
                    }),
                    false => cx.render(rsx! {
                        div {
                            class: "entry",
                            dangerous_inner_html: "{cx.props.entry.name}"
                        }
                    })
                }
            }
            div {
                id: "result-{cx.props.index}-actions",
                class: "actions {action_classes}",
                actions.into_iter()
            }
        }
    })
}

#[inline]
fn focus<T>(cx: Scope<T>) {
    let eval = use_eval(cx);
    let js = "document.getElementById(`search`).focus()";
    let _ = eval(js);
}

/// check if the current inputs match any of the given keybindings
#[inline]
fn matches(bind: &Vec<Keybind>, mods: &Modifiers, key: &Code) -> bool {
    bind.iter().any(|b| mods.contains(b.mods) && &b.key == key)
}

/// retrieve string value for display-capable enum
#[inline]
fn get_str<T: Display>(item: Option<T>) -> String {
    item.map(|i| i.to_string()).unwrap_or_else(String::new)
}

/// main application function/loop
fn App<'a>(cx: Scope<App>) -> Element {
    let mut state = AppState::new(cx, cx.props);

    // always ensure focus
    focus(cx);

    // log current position
    let search = state.search();
    let (pos, subpos) = state.position();
    log::debug!("search: {search:?}, pos: {pos}, {subpos}");

    // generate state tracker instances
    let results = state.results(&cx.props.entries);
    let k_updater = state.partial_copy();
    let s_updater = state.partial_copy();

    // build keyboard actions event handler
    let keybinds = &cx.props.config.keybinds;
    let keyboard_controls = move |e: KeyboardEvent| {
        let code = e.code();
        let mods = e.modifiers();
        if matches(&keybinds.exec, &mods, &code) {
            k_updater.set_event(KeyEvent::Exec);
        } else if matches(&keybinds.exit, &mods, &code) {
            k_updater.set_event(KeyEvent::Exit);
        } else if matches(&keybinds.move_next, &mods, &code) {
            k_updater.set_event(KeyEvent::MoveNext);
        } else if matches(&keybinds.move_prev, &mods, &code) {
            k_updater.set_event(KeyEvent::MovePrev);
        } else if matches(&keybinds.open_menu, &mods, &code) {
            k_updater.set_event(KeyEvent::OpenMenu);
        } else if matches(&keybinds.close_menu, &mods, &code) {
            k_updater.set_event(KeyEvent::CloseMenu);
        } else if matches(&keybinds.jump_next, &mods, &code) {
            k_updater.set_event(KeyEvent::JumpNext)
        } else if matches(&keybinds.jump_prev, &mods, &code) {
            k_updater.set_event(KeyEvent::JumpPrev)
        }
    };

    // handle keyboard events
    state.handle_events(cx);

    // render results objects
    let rendered_results = results.iter().enumerate().map(|(i, e)| {
        let state = state.partial_copy();
        cx.render(rsx! {
            TableEntry{
                pos:    pos,
                subpos: subpos,
                index:  i,
                entry:  e,
                state: state,
            }
        })
    });

    // get input settings
    let minlen = get_str(cx.props.config.search.min_length.as_ref());
    let maxlen = get_str(cx.props.config.search.max_length.as_ref());
    let placeholder = get_str(cx.props.config.search.placeholder.as_ref());

    // complete final rendering
    cx.render(rsx! {
        style { DEFAULT_CSS_CONTENT }
        style { "{cx.props.theme}" }
        style { "{cx.props.css}" }
        div {
            id: "content",
            class: "content",
            div {
                id: "navbar",
                class: "navbar",
                match cx.props.config.search.restrict.as_ref() {
                    Some(pattern) => cx.render(rsx! {
                        input {
                            id: "search",
                            value: "{search}",
                            pattern: "{pattern}",
                            minlength: "{minlen}",
                            maxlength: "{maxlen}",
                            placeholder: "{placeholder}",
                            oninput: move |e| s_updater.set_search(cx, e.value.clone()),
                            onkeydown: keyboard_controls,
                        }
                    }),
                    None => cx.render(rsx! {
                        input {
                            id: "search",
                            value: "{search}",
                            minlength: "{minlen}",
                            maxlength: "{maxlen}",
                            placeholder: "{placeholder}",
                            oninput: move |e| s_updater.set_search(cx, e.value.clone()),
                            onkeydown: keyboard_controls,
                        }
                    })
                }
            }
            div {
                id: "results",
                class: "results",
                rendered_results.into_iter()
            }
        }
    })
}
