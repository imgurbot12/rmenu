//! Execution Implementation for Entry Actions
use std::process::Command;
use std::{collections::HashMap, process::Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(not(target_os = "windows"))]
use std::os::unix::process::CommandExt;

use rmenu_plugin::{Action, Method};
use shell_words::split;
use strfmt::strfmt;
use which::which;

const DETACHED_PROCESS: u32 = 8u32;
const CREATE_NEW_PROCESS_GROUP: u32 = 512u32;

/// Find Best Terminal To Execute
fn find_terminal() -> String {
    vec![
        ("wezterm", "-e {cmd}"),
        ("alacritty", "-e {cmd}"),
        ("kitty", "{cmd}"),
        ("gnome-terminal", "-x {cmd}"),
        ("foot", "-e {cmd}"),
        ("xterm", "-C {cmd}"),
    ]
    .into_iter()
    .map(|(t, v)| (which(t), v))
    .filter(|(c, _)| c.is_ok())
    .map(|(c, v)| (c.unwrap(), v))
    .map(|(p, v)| {
        (
            p.to_str()
                .expect("Failed to Parse Terminal Path")
                .to_owned(),
            v,
        )
    })
    .find_map(|(p, v)| Some(format!("{p} {v}")))
    .expect("Failed to Find Terminal Executable!")
}

#[inline]
fn parse_args(exec: &str) -> Vec<String> {
    match split(exec) {
        Ok(args) => args,
        Err(err) => panic!("{:?} invalid command {err}", exec),
    }
}

/// Execute the Entry Action as Specified
pub fn execute(action: &Action, term: Option<String>) {
    log::info!("executing: {:?} {:?}", action.name, action.exec);
    let args = match &action.exec {
        Method::Run(exec) => parse_args(&exec),
        Method::Terminal(exec) => {
            let mut args = HashMap::new();
            let terminal = term.unwrap_or_else(find_terminal);
            args.insert("cmd".to_string(), exec.to_owned());
            let command = strfmt(&terminal, &args).expect("Failed String Format");
            parse_args(&command)
        }
        Method::Echo(echo) => {
            println!("{echo}");
            return;
        }
    };

    #[cfg(target_os = "windows")]
    {
        let child = Command::new(&args[0])
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            .args(&args[1..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();
        if let Err(err) = child {
            panic!("Command Error: {err:?}");
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let err = Command::new(&args[0]).args(&args[1..]).exec();
        panic!("Command Error: {err:?}");
    }
}
