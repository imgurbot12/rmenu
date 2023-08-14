#![allow(non_snake_case)]

///! NetworkManager Authenticator GUI
use anyhow::{anyhow, Result};
use dioxus::prelude::*;
use dioxus_desktop::LogicalSize;
use dioxus_free_icons::icons::fa_regular_icons::{FaEye, FaEyeSlash};
use dioxus_free_icons::Icon;
use keyboard_types::Code;

use crate::network::Manager;

static SPINNER_CSS: &'static str = include_str!("../public/spinner.css");
static SPINNER_HTML: &'static str = include_str!("../public/spinner.html");
static DEFAULT_CSS_CONTENT: &'static str = include_str!("../public/default.css");

/// Run GUI Application
pub fn run_app(ssid: &str, timeout: u32) {
    let builder = dioxus_desktop::WindowBuilder::new()
        .with_title("RMenu - Network Login")
        .with_inner_size(LogicalSize {
            width: 400,
            height: 150,
        })
        .with_focused(true)
        .with_decorations(false)
        .with_always_on_top(true);
    dioxus_desktop::launch_with_props(
        App,
        AppProp {
            ssid: ssid.to_owned(),
            timeout,
        },
        dioxus_desktop::Config::new().with_window(builder),
    );
}

/// Message to send to GUI
#[derive(Debug, Clone, PartialEq)]
pub enum UserMessage {
    Nothing,
    Error(String),
    Success(String),
}

impl UserMessage {
    /// Retrieve CSS-Class to use on UserMessage
    fn css_class(&self) -> &str {
        match self {
            Self::Error(_) => "error",
            Self::Success(_) => "success",
            Self::Nothing => "none",
        }
    }
    /// Retrieve Message Value
    fn message(&self) -> Option<String> {
        match self {
            Self::Nothing => None,
            Self::Error(msg) => Some(msg.to_owned()),
            Self::Success(msg) => Some(msg.to_owned()),
        }
    }
}

#[derive(Debug, PartialEq, Props)]
struct AppProp {
    ssid: String,
    timeout: u32,
}

//TODO: add auth timeout

/// Set Cursor/Keyboard Focus onto Input
#[inline]
fn focus<T>(cx: Scope<T>) {
    let eval = use_eval(cx);
    let js = "document.getElementById(`secret`).focus()";
    let _ = eval(&js);
}

/// Complete Network Connection/Authentication Attempt
async fn connect_async(ssid: String, timeout: u32, secret: &str) -> Result<()> {
    // retrieve access-point from manager
    let manager = Manager::new().await?.with_timeout(timeout);
    let access_point = manager
        .access_points()
        .into_iter()
        .find(|a| a.ssid == ssid)
        .ok_or_else(|| anyhow!("Unable to find Access Point: {ssid:?}"))?;
    // attempt to establish connection w/ secret
    log::debug!("Found AccessPoint: {ssid:?} | {:?}", access_point.security);
    manager.connect(&access_point, Some(secret)).await
}

/// Simple Login GUI Application
fn App(cx: Scope<AppProp>) -> Element {
    let secret = use_state(cx, || String::new());
    let message = use_state(cx, || UserMessage::Nothing);
    let show_secret = use_state(cx, || false);
    let loading = use_state(cx, || false);

    // always ensure focus
    focus(cx);

    // build keyboard actions event handler
    let keyboard_controls = move |e: KeyboardEvent| match e.code() {
        Code::Escape => std::process::exit(0),
        Code::Enter => {
            let ssid = cx.props.ssid.to_owned();
            let timeout = cx.props.timeout.to_owned();
            let secret = secret.get().to_owned();
            let message = message.to_owned();
            let loading = loading.to_owned();
            loading.set(true);
            message.set(UserMessage::Nothing);

            cx.spawn(async move {
                log::info!("connecting to ssid: {ssid:?}");
                let result = connect_async(ssid, timeout, &secret).await;
                log::info!("connection result: {result:?}");
                match result {
                    Ok(_) => {
                        // update message and unlock gui
                        message.set(UserMessage::Success("Connection Complete!".to_owned()));
                        loading.set(false);
                        // exit program after timeout
                        let wait = std::time::Duration::from_secs(1);
                        async_std::task::sleep(wait).await;
                        std::process::exit(0);
                    }
                    Err(err) => {
                        message.set(UserMessage::Error(format!("Connection Failed: {err:?}")));
                        loading.set(false);
                    }
                }
            });
        }
        _ => {}
    };

    // retrieve message details / set input-type / get loading css-class
    let msg_css = message.css_class();
    let msg_str = message.message();
    let input_type = match show_secret.get() {
        true => "text",
        false => "password",
    };
    let blackout_css = match loading.get() {
        true => "active",
        false => "",
    };

    // complete final rendering
    cx.render(rsx! {
        style { DEFAULT_CSS_CONTENT }
        style { SPINNER_CSS }
        div {
            onkeydown: keyboard_controls,
            label {
                id: "header",
                "for": "secret",
                "Wi-Fi Network {cx.props.ssid:?} Requires a Password"
            }
            div {
                id: "controls",
                input {
                    id: "secret",
                    value: "{secret}",
                    placeholder: "Password",
                    oninput: move |e| secret.set(e.value.clone()),
                    "type": "{input_type}"
                }
                button {
                    id: "icon",
                    onclick: |_| show_secret.modify(|v| !v),
                    match show_secret.get() {
                        true => cx.render(rsx! {
                            Icon {
                                fill: "black",
                                icon: FaEye,
                            }
                        }),
                        false => cx.render(rsx! {
                            Icon {
                                fill: "black",
                                icon: FaEyeSlash,
                            }
                        })
                    }
                }
            }
            if let Some(msg) = msg_str {
                cx.render(rsx! {
                    div {
                        id: "message",
                        class: "{msg_css}",
                        "{msg}"
                    }
                })
            }
            div {
                id: "blackout",
                class: "{blackout_css}",
                div {
                    id: "spinner",
                    dangerous_inner_html: "{SPINNER_HTML}"
                }
            }
        }
    })
}
