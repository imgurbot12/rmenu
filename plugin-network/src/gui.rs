#![allow(non_snake_case)]

///! NetworkManager Authenticator GUI
use anyhow::{anyhow, Result};
use dioxus::prelude::*;
use dioxus_desktop::LogicalSize;
use dioxus_free_icons::icons::fa_regular_icons::{FaEye, FaEyeSlash};
use dioxus_free_icons::Icon;
use dioxus_html::input_data::keyboard_types::Code;

use crate::network::Manager;

static SPINNER_CSS: &'static str = include_str!("../public/spinner.css");
static SPINNER_HTML: &'static str = include_str!("../public/spinner.html");
static DEFAULT_CSS_CONTENT: &'static str = include_str!("../public/default.css");

/// Run GUI Application
pub fn run(ssid: String, timeout: u32) {
    let window = dioxus_desktop::WindowBuilder::new()
        .with_title("RMenu - Network Login")
        .with_inner_size(LogicalSize {
            width: 400,
            height: 150,
        })
        .with_focused(true)
        .with_decorations(false)
        .with_always_on_top(true);
    let config = dioxus_desktop::Config::new()
        .with_menu(None)
        .with_window(window);
    let context = Context { ssid, timeout };
    LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(context)
        .launch(gui_main);
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
    fn css_class(&self) -> String {
        match self {
            Self::Error(_) => "error",
            Self::Success(_) => "success",
            Self::Nothing => "none",
        }
        .to_owned()
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

#[derive(Debug, Clone)]
struct Context {
    ssid: String,
    timeout: u32,
}

//TODO: add auth timeout

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

fn gui_main() -> Element {
    let ctx = use_context::<Context>();
    let mut message = use_signal(|| UserMessage::Nothing);
    let mut loading = use_signal(|| false);
    let mut secret = use_signal(String::new);
    let mut show_secret = use_signal(|| false);

    // refocus on input
    let js = format!("setTimeout(() => {{ document.getElementById('secret').focus() }}, 100)");
    eval(&js);

    // build keyboard actions event handler
    let keyboard_controls = move |e: KeyboardEvent| match e.code() {
        Code::Escape => std::process::exit(0),
        Code::Enter => {
            let timeout = ctx.timeout.to_owned();
            loading.set(true);
            message.set(UserMessage::Nothing);

            spawn(async move {
                let ctx = use_context::<Context>();
                let ssid = ctx.ssid.to_owned();
                log::info!("connecting to ssid: {ssid:?}");
                let result = connect_async(ssid, timeout, &secret()).await;
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
    let msg_css = message.with(|msg| msg.css_class());
    let msg_str = message.with(|msg| msg.message());
    let input_type = match show_secret() {
        true => "text",
        false => "password",
    };
    let blackout_css = match loading() {
        true => "active",
        false => "",
    };

    // complete final rendering
    rsx! {
        style { "{DEFAULT_CSS_CONTENT}" }
        style { "{SPINNER_CSS}" }
        div {
            onkeydown: keyboard_controls,
            label {
                id: "header",
                "for": "secret",
                "Wi-Fi Network {ctx.ssid:?} Requires a Password"
            }
            div {
                id: "controls",
                input {
                    id: "secret",
                    value: "{secret}",
                    placeholder: "Password",
                    oninput: move |e| secret.set(e.value()),
                    "type": "{input_type}"
                }
                button {
                    id: "icon",
                    onclick: move |_| show_secret.toggle(),
                    {match show_secret() {
                        true => rsx! {
                            Icon {
                                fill: "black",
                                icon: FaEye,
                            }
                        },
                        false => rsx! {
                            Icon {
                                fill: "black",
                                icon: FaEyeSlash,
                            }
                        }
                    }}
                }
            }
            if let Some(msg) = msg_str {
                {rsx! {
                    div {
                        id: "message",
                        class: "{msg_css}",
                        "{msg}"
                    }
                }}
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
    }
}
