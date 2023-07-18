#![allow(non_snake_case)]
use dioxus::prelude::*;
use rmenu_plugin::Entry;

use crate::App;

pub fn run(app: App) {
    #[cfg(target_family = "wasm")]
    dioxus_web::launch(App, app, dioxus_web::Config::default());

    #[cfg(any(windows, unix))]
    dioxus_desktop::launch_with_props(App, app, dioxus_desktop::Config::default());
}

#[derive(PartialEq, Props)]
struct GEntry<'a> {
    o: &'a Entry,
}

fn TableEntry<'a>(cx: Scope<'a, GEntry<'a>>) -> Element<'a> {
    cx.render(rsx! {
        div {
            class: "result",
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
    let results = &cx.props.entries;
    let searchstr = search.as_str();
    let results_rendered = results
        .iter()
        .filter(|entry| {
            if entry.name.contains(searchstr) {
                return true;
            }
            if let Some(comment) = entry.comment.as_ref() {
                return comment.contains(searchstr);
            }
            false
        })
        .map(|entry| cx.render(rsx! { TableEntry{ o: entry } }));

    cx.render(rsx! {
        style { "{cx.props.css}" }
        input {
            value: "{search}",
            oninput: move |evt| search.set(evt.value.clone()),
        }
        results_rendered
    })
}
