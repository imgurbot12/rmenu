#![allow(non_snake_case)]
use dioxus::prelude::*;
use keyboard_types::{Code, Modifiers};
use rmenu_plugin::Entry;

use crate::search::new_searchfn;
use crate::state::PosTracker;
use crate::App;

pub fn run(app: App) {
    dioxus_desktop::launch_with_props(App, app, dioxus_desktop::Config::default());
}

#[derive(PartialEq, Props)]
struct GEntry<'a> {
    i: usize,
    o: &'a Entry,
    selected: bool,
}

fn TableEntry<'a>(cx: Scope<'a, GEntry<'a>>) -> Element<'a> {
    let classes = if cx.props.selected { "selected" } else { "" };
    cx.render(rsx! {
        div {
            id: "result-{cx.props.i}",
            class: "result {classes}",
            div {
                class: "icon",
                if let Some(icon) = cx.props.o.icon.as_ref() {
                    cx.render(rsx! { img { src: "{icon}" } })
                }
            }
            div {
                class: "name",
                "{cx.props.o.name}"
            }
            div {
                class: "comment",
                if let Some(comment) = cx.props.o.comment.as_ref() {
                    format!("- {comment}")
                }
            }
        }
    })
}

fn App(cx: Scope<App>) -> Element {
    let search = use_state(cx, || "".to_string());
    let position = use_state(cx, || 0);

    // retrieve build results tracker
    let results = &cx.props.entries;
    let mut tracker = PosTracker::new(position, results);

    // keyboard events
    let eval = dioxus_desktop::use_eval(cx);
    let change_evt = move |evt: KeyboardEvent| {
        match evt.code() {
            // modify position
            Code::ArrowUp => tracker.shift_up(),
            Code::ArrowDown => tracker.shift_down(),
            Code::Tab => match evt.modifiers().contains(Modifiers::SHIFT) {
                true => tracker.close_menu(),
                false => tracker.open_menu(),
            },
            _ => println!("key: {:?}", evt.key()),
        }
        // always set focus back on input
        let js = "document.getElementById(`search`).focus()";
        eval(js.to_owned());
    };

    // pre-render results into elements
    let searchfn = new_searchfn(&cx.props.config, &search);
    let results_rendered: Vec<Element> = results
        .iter()
        .filter(|entry| searchfn(entry))
        .enumerate()
        .map(|(i, entry)| {
            cx.render(rsx! {
                TableEntry{ i: i, o: entry, selected: (i + 1) == active }
            })
        })
        .collect();

    cx.render(rsx! {
        style { "{cx.props.css}" }
        div {
            onkeydown: change_evt,
            input {
                id: "search",
                value: "{search}",
                oninput: move |evt| search.set(evt.value.clone()),

            }
            results_rendered.into_iter()
        }
    })
}
