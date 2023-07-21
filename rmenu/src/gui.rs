//! RMENU GUI Implementation using Dioxus
#![allow(non_snake_case)]
use dioxus::prelude::*;
use keyboard_types::{Code, Modifiers};
use rmenu_plugin::Entry;

use crate::config::{Config, Keybind};
use crate::exec::execute;
use crate::search::new_searchfn;
use crate::state::PosTracker;
use crate::App;

/// spawn and run the app on the configured platform
pub fn run(app: App) {
    // customize window
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
        .with_theme(theme);
    let config = dioxus_desktop::Config::new().with_window(builder);
    dioxus_desktop::launch_with_props(App, app, config);
}

#[derive(PartialEq, Props)]
struct GEntry<'a> {
    index: usize,
    entry: &'a Entry,
    config: &'a Config,
    pos: usize,
    subpos: usize,
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
                    div {
                        class: "action-name",
                        "{action.name}"
                    }
                    div {
                        class: "action-comment",
                        if let Some(comment) = action.comment.as_ref() {
                            format!("- {comment}")
                        }
                    }
                }
            })
        });
    cx.render(rsx! {
        div {
            id: "result-{cx.props.index}",
            class: "result {result_classes} {multi_classes}",
            ondblclick: |_| {
                let action = match cx.props.entry.actions.get(0) {
                    Some(action) => action,
                    None => {
                        let name = &cx.props.entry.name;
                        log::warn!("no action to execute on {:?}", name);
                        return;
                    }
                };
                log::info!("executing: {:?}", action.exec);
                execute(action);
            },
            if cx.props.config.use_icons {
                cx.render(rsx! {
                    div {
                        class: "icon",
                        if let Some(icon) = cx.props.entry.icon.as_ref() {
                            cx.render(rsx! { img { src: "{icon}" } })
                        }
                    }
                })
            }
            div {
                class: "name",
                "{cx.props.entry.name}"
            }
            div {
                class: "comment",
                if let Some(comment) = cx.props.entry.comment.as_ref() {
                    comment.to_string()
                }
            }
        }
        div {
            id: "result-{cx.props.index}-actions",
            class: "actions {action_classes}",
            actions.into_iter()
        }
    })
}

#[inline]
fn focus<T>(cx: Scope<T>) {
    let eval = dioxus_desktop::use_eval(cx);
    let js = "document.getElementById(`search`).focus()";
    eval(js.to_owned());
}

/// check if the current inputs match any of the given keybindings
#[inline]
fn matches(bind: &Vec<Keybind>, mods: &Modifiers, key: &Code) -> bool {
    bind.iter().any(|b| mods.contains(b.mods) && &b.key == key)
}

/// main application function/loop
fn App(cx: Scope<App>) -> Element {
    let quit = use_state(cx, || false);
    let search = use_state(cx, || "".to_string());

    // handle exit check
    if *quit.get() {
        std::process::exit(0);
    }

    // retrieve results and filter based on search
    let searchfn = new_searchfn(&cx.props.config, &search);
    let results: Vec<&Entry> = cx
        .props
        .entries
        .iter()
        .filter(|entry| searchfn(entry))
        .collect();

    // retrieve results build and build position-tracker
    let tracker = PosTracker::new(cx, results.clone());
    let (pos, subpos) = tracker.position();
    log::debug!("pos: {pos}, {subpos}");

    // keyboard events
    let keybinds = &cx.props.config.keybinds;
    let keyboard_evt = move |evt: KeyboardEvent| {
        let key = &evt.code();
        let mods = &evt.modifiers();
        log::debug!("key: {key:?} mods: {mods:?}");
        if matches(&keybinds.exec, mods, key) {
            match tracker.action() {
                Some(action) => execute(action),
                None => panic!("No Action Configured"),
            }
        } else if matches(&keybinds.exit, mods, key) {
            quit.set(true);
        } else if matches(&keybinds.move_up, mods, key) {
            tracker.shift_up();
        } else if matches(&keybinds.move_down, mods, key) {
            tracker.shift_down();
        } else if matches(&keybinds.open_menu, mods, key) {
            tracker.open_menu();
        } else if matches(&keybinds.close_menu, mods, key) {
            tracker.close_menu();
        }
        // always set focus back on input
        focus(cx);
    };

    // pre-render results into elements
    let results_rendered: Vec<Element> = results
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            cx.render(rsx! {
                TableEntry{
                    index: index,
                    entry: entry,
                    config: &cx.props.config,
                    pos:   pos,
                    subpos: subpos,
                }
            })
        })
        .collect();

    cx.render(rsx! {
        style { "{cx.props.css}" }
        div {
            onkeydown: keyboard_evt,
            onclick: |_| focus(cx),
            input {
                id: "search",
                value: "{search}",
                oninput: move |evt| search.set(evt.value.clone()),

            }
            results_rendered.into_iter()
        }
    })
}
