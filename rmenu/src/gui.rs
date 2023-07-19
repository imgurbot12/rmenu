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
    index: usize,
    entry: &'a Entry,
    pos: usize,
    subpos: usize,
}

fn TableEntry<'a>(cx: Scope<'a, GEntry<'a>>) -> Element<'a> {
    // build css classes for result and actions (if nessesary)
    let main_select = cx.props.index == cx.props.pos;
    let action_select = main_select && cx.props.subpos > 0;
    let action_classes = match action_select {
        true => "active",
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
            class: "result {result_classes}",
            div {
                class: "icon",
                if let Some(icon) = cx.props.entry.icon.as_ref() {
                    cx.render(rsx! { img { src: "{icon}" } })
                }
            }
            div {
                class: "name",
                "{cx.props.entry.name}"
            }
            div {
                class: "comment",
                if let Some(comment) = cx.props.entry.comment.as_ref() {
                    format!("- {comment}")
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

fn App(cx: Scope<App>) -> Element {
    let search = use_state(cx, || "".to_string());

    // retrieve build results tracker
    let results = &cx.props.entries;
    let tracker = PosTracker::new(cx, results);
    let (pos, subpos) = tracker.position();
    println!("pos: {pos}, {subpos}");

    // keyboard events
    let eval = dioxus_desktop::use_eval(cx);
    let change_evt = move |evt: KeyboardEvent| {
        match evt.code() {
            // modify position
            Code::ArrowUp => tracker.shift_up(),
            Code::ArrowDown => tracker.shift_down(),
            Code::Tab => match evt.modifiers().contains(Modifiers::SHIFT) {
                true => {
                    println!("close menu");
                    tracker.close_menu()
                }
                false => {
                    println!("open menu!");
                    tracker.open_menu()
                }
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
        .map(|(index, entry)| {
            cx.render(rsx! {
                TableEntry{
                    index: index,
                    entry: entry,
                    pos:   pos,
                    subpos: subpos,
                }
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
